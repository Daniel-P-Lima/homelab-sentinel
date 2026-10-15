//! Métricas do servidor (host): CPU, memória, disco, temperatura e bateria.
//!
//! Dentro de um container, `/proc/stat`, `/proc/meminfo`, `/proc/uptime` e
//! `/proc/loadavg` mostram o host inteiro — o kernel não isola esses arquivos
//! por namespace — e o Docker monta o `/sys` do host só-leitura. Então nada
//! aqui exige container privilegiado; só o disco precisa de um caminho do host
//! montado (`HOST_DISK_PATH`, ver docker-compose.yml).

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Serialize;
use thiserror::Error;

/// Intervalo entre as duas leituras do `/proc/stat`. CPU% é um delta: uma
/// leitura só daria a média desde o boot (o mesmo problema do `one_shot` em
/// `docker.rs`).
const CPU_SAMPLE_INTERVAL: Duration = Duration::from_millis(500);

/// Zona térmica do pacote da CPU em Intel; sem ela, vale a mais quente.
const CPU_THERMAL_ZONE: &str = "x86_pkg_temp";

#[derive(Debug, Error)]
pub enum HostError {
    #[error("falha ao ler {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("formato inesperado em {0}")]
    Parse(&'static str),
}

#[derive(Debug, Serialize)]
pub struct HostSummary {
    pub uptime_seconds: u64,
    pub cpu_count: usize,
    /// None se as duas amostras vierem iguais (não deveria acontecer em 500ms).
    pub cpu_percent: Option<f64>,
    /// Carga média de 1, 5 e 15 minutos.
    pub load_avg: [f64; 3],
    pub mem_total_mb: f64,
    /// Total menos `MemAvailable`: cache que o kernel pode liberar não conta.
    pub mem_used_mb: f64,
    pub swap_total_mb: f64,
    pub swap_used_mb: f64,
    /// None sem `HOST_DISK_PATH` ou se o `statvfs` falhar.
    pub disk: Option<DiskUsage>,
    pub temperature_c: Option<f64>,
    /// None em máquina sem bateria.
    pub battery: Option<Battery>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct DiskUsage {
    pub total_gb: f64,
    pub used_gb: f64,
    pub available_gb: f64,
    /// Mesma conta do `df`: usado / (usado + disponível). O ext4 reserva ~5%
    /// para o root, que não entra em nenhum dos dois.
    pub used_percent: f64,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Battery {
    pub capacity_percent: u8,
    /// Como o kernel devolve: "Charging", "Discharging", "Full", "Not charging".
    pub status: String,
    /// Capacidade máxima atual em relação à de fábrica, em %.
    pub health_percent: Option<f64>,
}

pub struct HostCollector {
    disk_path: Option<PathBuf>,
}

impl HostCollector {
    /// None fora do Linux (ex: `cargo run` no Mac): sem `/proc` não há o que ler.
    pub fn from_env() -> Option<Self> {
        if !Path::new("/proc/stat").exists() {
            return None;
        }
        let disk_path = std::env::var_os("HOST_DISK_PATH")
            .filter(|p| !p.is_empty())
            .map(PathBuf::from);
        Some(Self { disk_path })
    }

    pub fn disk_path(&self) -> Option<&Path> {
        self.disk_path.as_deref()
    }

    pub async fn summary(&self) -> Result<HostSummary, HostError> {
        let before = read("/proc/stat").await?;
        tokio::time::sleep(CPU_SAMPLE_INTERVAL).await;
        let after = read("/proc/stat").await?;

        let cpu_before = parse_cpu_times(&before).ok_or(HostError::Parse("/proc/stat"))?;
        let cpu_after = parse_cpu_times(&after).ok_or(HostError::Parse("/proc/stat"))?;

        let mem = parse_meminfo(&read("/proc/meminfo").await?).ok_or(HostError::Parse("/proc/meminfo"))?;
        let uptime = parse_uptime(&read("/proc/uptime").await?).ok_or(HostError::Parse("/proc/uptime"))?;
        let load_avg = parse_loadavg(&read("/proc/loadavg").await?).ok_or(HostError::Parse("/proc/loadavg"))?;

        // Disco, temperatura e bateria são extras: se faltarem, o card mostra
        // o resto em vez de virar erro.
        let disk = self.disk_path.as_deref().and_then(|path| match disk_usage(path) {
            Ok(d) => Some(d),
            Err(e) => {
                tracing::warn!("statvfs em {} falhou: {e}", path.display());
                None
            }
        });

        Ok(HostSummary {
            uptime_seconds: uptime as u64,
            cpu_count: count_cpus(&after),
            cpu_percent: cpu_percent(&cpu_before, &cpu_after),
            load_avg,
            mem_total_mb: kb_to_mb(mem.total_kb),
            mem_used_mb: kb_to_mb(mem.total_kb.saturating_sub(mem.available_kb)),
            swap_total_mb: kb_to_mb(mem.swap_total_kb),
            swap_used_mb: kb_to_mb(mem.swap_total_kb.saturating_sub(mem.swap_free_kb)),
            disk,
            temperature_c: read_temperature().await,
            battery: read_battery().await,
        })
    }
}

async fn read(path: &str) -> Result<String, HostError> {
    tokio::fs::read_to_string(path).await.map_err(|source| HostError::Read {
        path: path.to_string(),
        source,
    })
}

/// Lê um arquivo de `/sys`, sem a quebra de linha do fim. None se não existir.
async fn read_sys(path: impl AsRef<Path>) -> Option<String> {
    tokio::fs::read_to_string(path).await.ok().map(|s| s.trim().to_string())
}

// --- CPU ---

#[derive(Debug, PartialEq)]
struct CpuTimes {
    /// idle + iowait: tempo em que a CPU não estava trabalhando.
    idle: u64,
    total: u64,
}

/// Primeira linha do `/proc/stat`, em ticks desde o boot:
/// `cpu  user nice system idle iowait irq softirq steal guest guest_nice`.
/// guest e guest_nice já estão somados em user/nice, então ficam de fora do
/// total para não contar duas vezes.
fn parse_cpu_times(stat: &str) -> Option<CpuTimes> {
    let line = stat.lines().next()?.strip_prefix("cpu ")?;
    let fields: Vec<u64> = line
        .split_whitespace()
        .take(8)
        .map(|f| f.parse().ok())
        .collect::<Option<_>>()?;
    if fields.len() < 5 {
        return None;
    }
    Some(CpuTimes {
        idle: fields[3] + fields[4],
        total: fields.iter().sum(),
    })
}

fn cpu_percent(before: &CpuTimes, after: &CpuTimes) -> Option<f64> {
    let total = after.total.checked_sub(before.total)?;
    let idle = after.idle.checked_sub(before.idle)?;
    if total == 0 {
        return None;
    }
    Some((1.0 - idle as f64 / total as f64) * 100.0)
}

/// Uma linha `cpuN` por CPU lógica, depois da linha agregada `cpu`.
fn count_cpus(stat: &str) -> usize {
    stat.lines()
        .filter(|l| l.strip_prefix("cpu").is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_digit())))
        .count()
}

// --- memória, uptime, carga ---

#[derive(Debug, PartialEq)]
struct Meminfo {
    total_kb: u64,
    available_kb: u64,
    swap_total_kb: u64,
    swap_free_kb: u64,
}

/// Linhas no formato `MemTotal:        7465136 kB`.
fn parse_meminfo(meminfo: &str) -> Option<Meminfo> {
    let field = |name: &str| {
        meminfo.lines().find_map(|l| {
            let (key, value) = l.split_once(':')?;
            (key == name).then(|| value.trim().trim_end_matches(" kB").parse().ok())?
        })
    };
    Some(Meminfo {
        total_kb: field("MemTotal")?,
        available_kb: field("MemAvailable")?,
        swap_total_kb: field("SwapTotal").unwrap_or(0),
        swap_free_kb: field("SwapFree").unwrap_or(0),
    })
}

/// `7589.20 27302.25`: segundos desde o boot e tempo ocioso somado das CPUs.
fn parse_uptime(uptime: &str) -> Option<f64> {
    uptime.split_whitespace().next()?.parse().ok()
}

/// `0.98 0.82 0.81 4/691 9`: só as três primeiras interessam.
fn parse_loadavg(loadavg: &str) -> Option<[f64; 3]> {
    let mut it = loadavg.split_whitespace().map(|f| f.parse::<f64>().ok());
    Some([it.next()??, it.next()??, it.next()??])
}

fn kb_to_mb(kb: u64) -> f64 {
    kb as f64 / 1024.0
}

// --- disco ---

/// `statvfs(3)`, a mesma chamada que o `df` usa. A std do Rust não a expõe,
/// então ela vem do crate `libc` — uma chamada direta à libc do sistema, por
/// isso o `unsafe`.
#[cfg(unix)]
// `as u64` nos campos abaixo: o tamanho deles muda entre Linux e macOS, então
// num dos dois o cast é redundante.
#[allow(clippy::unnecessary_cast)]
fn disk_usage(path: &Path) -> std::io::Result<DiskUsage> {
    use std::ffi::CString;
    use std::mem::MaybeUninit;
    use std::os::unix::ffi::OsStrExt;

    let c_path = CString::new(path.as_os_str().as_bytes())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    let mut stat = MaybeUninit::<libc::statvfs>::uninit();

    // SAFETY: `c_path` é uma string C válida (terminada em \0) que vive até o
    // fim da função, e `stat` aponta para memória do tamanho de `statvfs`.
    let rc = unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: rc == 0 garante que a chamada preencheu a struct inteira.
    let stat = unsafe { stat.assume_init() };

    Ok(disk_from_blocks(
        stat.f_blocks as u64,
        stat.f_bfree as u64,
        stat.f_bavail as u64,
        stat.f_frsize as u64,
    ))
}

#[cfg(not(unix))]
fn disk_usage(_path: &Path) -> std::io::Result<DiskUsage> {
    Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "statvfs só existe em Unix"))
}

/// `bfree` inclui os blocos reservados ao root; `bavail` é o que um usuário
/// comum ainda pode usar. Por isso usado = blocks - bfree, e não blocks - bavail.
fn disk_from_blocks(blocks: u64, bfree: u64, bavail: u64, block_size: u64) -> DiskUsage {
    let gb = |n: u64| (n * block_size) as f64 / 1024.0 / 1024.0 / 1024.0;
    let used = blocks.saturating_sub(bfree);
    let used_percent = if used + bavail == 0 {
        0.0
    } else {
        used as f64 / (used + bavail) as f64 * 100.0
    };
    DiskUsage {
        total_gb: gb(blocks),
        used_gb: gb(used),
        available_gb: gb(bavail),
        used_percent,
    }
}

// --- temperatura e bateria (/sys) ---

async fn read_temperature() -> Option<f64> {
    let mut zones = Vec::new();
    let mut dir = tokio::fs::read_dir("/sys/class/thermal").await.ok()?;
    while let Ok(Some(entry)) = dir.next_entry().await {
        if !entry.file_name().to_string_lossy().starts_with("thermal_zone") {
            continue;
        }
        let path = entry.path();
        let (Some(kind), Some(temp)) = (read_sys(path.join("type")).await, read_sys(path.join("temp")).await)
        else {
            continue;
        };
        if let Ok(millidegrees) = temp.parse::<i64>() {
            zones.push((kind, millidegrees));
        }
    }
    pick_temperature(&zones)
}

/// Zonas em miligraus Celsius. Prefere a da CPU; sem ela, a mais quente.
fn pick_temperature(zones: &[(String, i64)]) -> Option<f64> {
    let millidegrees = zones
        .iter()
        .find(|(kind, _)| kind == CPU_THERMAL_ZONE)
        .or_else(|| zones.iter().max_by_key(|(_, t)| *t))
        .map(|(_, t)| *t)?;
    Some(millidegrees as f64 / 1000.0)
}

/// Primeira fonte de energia do tipo "Battery" em `/sys/class/power_supply`.
async fn read_battery() -> Option<Battery> {
    let mut dir = tokio::fs::read_dir("/sys/class/power_supply").await.ok()?;
    while let Ok(Some(entry)) = dir.next_entry().await {
        let path = entry.path();
        if read_sys(path.join("type")).await.as_deref() != Some("Battery") {
            continue;
        }
        let capacity_percent = read_sys(path.join("capacity")).await?.parse().ok()?;
        let status = read_sys(path.join("status")).await.unwrap_or_else(|| "Unknown".into());
        // Conforme o driver, a bateria informa carga (µAh) ou energia (µWh).
        let mut health_percent = None;
        for (full, design) in [("charge_full", "charge_full_design"), ("energy_full", "energy_full_design")] {
            if let (Some(f), Some(d)) = (read_sys(path.join(full)).await, read_sys(path.join(design)).await) {
                health_percent = battery_health(&f, &d);
                break;
            }
        }
        return Some(Battery {
            capacity_percent,
            status,
            health_percent,
        });
    }
    None
}

fn battery_health(full: &str, design: &str) -> Option<f64> {
    let (full, design): (f64, f64) = (full.parse().ok()?, design.parse().ok()?);
    (design > 0.0).then(|| full / design * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Recortes reais do servidor do homelab (i5 de 4 threads, Ubuntu).
    const STAT: &str = "cpu  203438 5929 53795 2730225 7664 0 5586 0 0 0
cpu0 50000 1000 13000 680000 2000 0 1400 0 0 0
cpu1 50000 1000 13000 680000 2000 0 1400 0 0 0
cpu2 50000 1000 13000 680000 2000 0 1400 0 0 0
cpu3 50000 1000 13000 680000 2000 0 1400 0 0 0
intr 123 0 0
ctxt 456
";
    const MEMINFO: &str = "MemTotal:        7465136 kB
MemFree:          518000 kB
MemAvailable:    5735664 kB
SwapTotal:       4194300 kB
SwapFree:        3866216 kB
";

    #[test]
    fn le_tempos_de_cpu_da_linha_agregada() {
        let t = parse_cpu_times(STAT).unwrap();
        assert_eq!(t.idle, 2_730_225 + 7_664);
        assert_eq!(t.total, 203_438 + 5_929 + 53_795 + 2_730_225 + 7_664 + 5_586);
    }

    #[test]
    fn cpu_percent_e_o_delta_entre_leituras() {
        // 1000 ticks passaram, 750 ociosos → 25% de uso
        let before = CpuTimes { idle: 10_000, total: 20_000 };
        let after = CpuTimes { idle: 10_750, total: 21_000 };
        assert!((cpu_percent(&before, &after).unwrap() - 25.0).abs() < 1e-9);
        // amostras iguais não têm delta
        assert_eq!(cpu_percent(&before, &before), None);
    }

    #[test]
    fn conta_uma_cpu_por_linha_cpun() {
        assert_eq!(count_cpus(STAT), 4);
    }

    #[test]
    fn le_memoria_e_swap() {
        assert_eq!(
            parse_meminfo(MEMINFO).unwrap(),
            Meminfo {
                total_kb: 7_465_136,
                available_kb: 5_735_664,
                swap_total_kb: 4_194_300,
                swap_free_kb: 3_866_216,
            }
        );
        // sem swap configurado as linhas existem com 0, mas não dependemos disso
        let sem_swap = "MemTotal: 1000 kB\nMemAvailable: 400 kB\n";
        assert_eq!(parse_meminfo(sem_swap).unwrap().swap_total_kb, 0);
    }

    #[test]
    fn le_uptime_e_carga() {
        assert_eq!(parse_uptime("7589.20 27302.25\n"), Some(7589.2));
        assert_eq!(parse_loadavg("0.98 0.82 0.81 4/691 9\n"), Some([0.98, 0.82, 0.81]));
        assert_eq!(parse_loadavg("lixo"), None);
    }

    #[test]
    fn disco_segue_a_conta_do_df() {
        // 100 blocos de 1 GiB: 40 livres, dos quais 5 reservados ao root
        let gib = 1024 * 1024 * 1024;
        let d = disk_from_blocks(100, 40, 35, gib);
        assert_eq!(d.total_gb, 100.0);
        assert_eq!(d.used_gb, 60.0);
        assert_eq!(d.available_gb, 35.0);
        assert!((d.used_percent - 60.0 / 95.0 * 100.0).abs() < 1e-9);
    }

    #[test]
    fn temperatura_prefere_a_zona_da_cpu() {
        let zones = vec![
            ("acpitz".to_string(), 43_000),
            ("x86_pkg_temp".to_string(), 50_000),
            ("pch_skylake".to_string(), 61_000),
        ];
        assert_eq!(pick_temperature(&zones), Some(50.0));
        assert_eq!(pick_temperature(&zones[..1]), Some(43.0));
        assert_eq!(pick_temperature(&[]), None);
    }

    #[test]
    fn saude_da_bateria() {
        // valores reais do BAT1 do servidor
        assert!((battery_health("1702000", "2500000").unwrap() - 68.08).abs() < 1e-6);
        assert_eq!(battery_health("100", "0"), None);
    }
}
