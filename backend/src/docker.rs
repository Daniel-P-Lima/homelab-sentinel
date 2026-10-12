use bollard::container::{ListContainersOptions, MemoryStatsStats, Stats, StatsOptions};
use bollard::Docker;
use futures_util::stream::{self, StreamExt};
use serde::Serialize;
use thiserror::Error;

/// Quantos containers têm stats coletados ao mesmo tempo. Cada amostra leva
/// ~1-2s no daemon (ver `fetch_stats`), então em série a request inteira
/// passaria do intervalo de poll do frontend.
const STATS_CONCURRENCY: usize = 8;

/// Tentativas de ping no boot, com 2s entre elas (ver `connect`).
const PING_ATTEMPTS: u32 = 5;

#[derive(Debug, Error)]
pub enum DockerError {
    #[error("falha ao conectar no Docker daemon: {0}")]
    Connect(#[source] bollard::errors::Error),
    #[error("falha ao listar containers: {0}")]
    List(#[source] bollard::errors::Error),
}

#[derive(Debug, Serialize, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    /// Estado bruto do Docker: running, exited, paused, etc.
    pub state: String,
    /// Texto legível, ex: "Up 3 days"
    pub status: String,
    /// None quando o container não está rodando (stats não fazem sentido)
    pub cpu_percent: Option<f64>,
    pub mem_usage_mb: Option<f64>,
    pub mem_limit_mb: Option<f64>,
}

/// Conecta no Docker respeitando `DOCKER_HOST` (em produção aponta pro
/// docker-socket-proxy, ver docker-compose.yml); sem a variável, usa o
/// socket local `/var/run/docker.sock`.
///
/// Criar o cliente não abre conexão nenhuma — por isso o `ping`: sem ele, um
/// daemon inexistente só seria descoberto na primeira request.
pub async fn connect() -> Result<Docker, DockerError> {
    // bollard 0.15 não tem um connect que escolha sozinho entre TCP e socket;
    // a escolha fica explícita aqui.
    let docker = if std::env::var_os("DOCKER_HOST").is_some() {
        Docker::connect_with_http_defaults()
    } else {
        Docker::connect_with_socket_defaults()
    }
    .map_err(DockerError::Connect)?;

    // No boot do servidor o docker-proxy pode subir depois do backend
    // (depends_on só espera o container iniciar, não ficar pronto). Sem essas
    // tentativas, o backend ficaria em 503 até ser reiniciado na mão.
    let mut attempt = 1;
    loop {
        match docker.ping().await {
            Ok(_) => return Ok(docker),
            Err(e) if attempt >= PING_ATTEMPTS => return Err(DockerError::Connect(e)),
            Err(e) => {
                tracing::warn!("ping no Docker falhou (tentativa {attempt}/{PING_ATTEMPTS}): {e}");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                attempt += 1;
            }
        }
    }
}

pub async fn list_containers(docker: &Docker) -> Result<Vec<ContainerInfo>, DockerError> {
    let options = ListContainersOptions::<String> {
        all: true, // inclui parados também, não só os "running"
        ..Default::default()
    };

    let containers = docker
        .list_containers(Some(options))
        .await
        .map_err(DockerError::List)?;

    let mut result: Vec<ContainerInfo> = stream::iter(containers)
        .map(|c| async move {
            let id = c.id.unwrap_or_default();
            let name = c
                .names
                .as_ref()
                .and_then(|names| names.first())
                .map(|n| n.trim_start_matches('/').to_string())
                .unwrap_or_else(|| id.clone());
            let image = c.image.unwrap_or_default();
            let state = c.state.unwrap_or_default();
            let status = c.status.unwrap_or_default();

            // Stats só fazem sentido pra containers rodando; puxar stats de um
            // container parado retorna erro/vazio no daemon.
            let (cpu_percent, mem_usage_mb, mem_limit_mb) =
                if state == "running" && !id.is_empty() {
                    match fetch_stats(docker, &id).await {
                        Ok(stats) => stats,
                        Err(e) => {
                            tracing::warn!("falha ao coletar stats de {name}: {e}");
                            (None, None, None)
                        }
                    }
                } else {
                    (None, None, None)
                };

            ContainerInfo {
                id,
                name,
                image,
                state,
                status,
                cpu_percent,
                mem_usage_mb,
                mem_limit_mb,
            }
        })
        .buffer_unordered(STATS_CONCURRENCY)
        .collect()
        .await;

    // buffer_unordered devolve na ordem em que terminam; ordena por nome pra
    // UI ficar estável entre polls
    result.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(result)
}

/// Pega uma única amostra de stats (stream=false).
///
/// `one_shot` precisa ser false: com true o daemon não espera o segundo ciclo
/// de leitura e devolve `precpu_stats` zerado, e o "delta" de CPU vira o uso
/// acumulado desde que o container subiu — uma média da vida inteira, não o
/// uso atual.
async fn fetch_stats(
    docker: &Docker,
    id: &str,
) -> Result<(Option<f64>, Option<f64>, Option<f64>), bollard::errors::Error> {
    let options = StatsOptions {
        stream: false,
        one_shot: false,
    };

    let mut stream = docker.stats(id, Some(options));
    let stats: Stats = match stream.next().await {
        Some(Ok(s)) => s,
        Some(Err(e)) => return Err(e),
        None => return Ok((None, None, None)),
    };

    let cpu_count = stats.cpu_stats.online_cpus.or_else(|| {
        stats
            .cpu_stats
            .cpu_usage
            .percpu_usage
            .as_ref()
            .map(|v| v.len() as u64)
    });
    let cpu = cpu_percent(
        stats.cpu_stats.cpu_usage.total_usage,
        stats.precpu_stats.cpu_usage.total_usage,
        stats.cpu_stats.system_cpu_usage,
        stats.precpu_stats.system_cpu_usage,
        cpu_count,
    );

    // Mesmo desconto que o `docker stats` faz: page cache inativo pode ser
    // reclamado pelo kernel, então não conta como uso. O nome do campo muda
    // entre cgroup v1 e v2.
    let inactive_file = match stats.memory_stats.stats {
        Some(MemoryStatsStats::V1(s)) => Some(s.total_inactive_file),
        Some(MemoryStatsStats::V2(s)) => Some(s.inactive_file),
        None => None,
    };
    let mem_usage = stats
        .memory_stats
        .usage
        .map(|u| bytes_to_mb(memory_used(u, inactive_file)));
    let mem_limit = stats.memory_stats.limit.map(bytes_to_mb);

    Ok((cpu, mem_usage, mem_limit))
}

/// CPU% como o `docker stats` calcula: delta de uso do container / delta do
/// sistema × nº de CPUs × 100. Funções puras em cima de números crus pra dar
/// pra testar sem montar um `Stats` inteiro.
fn cpu_percent(
    total: u64,
    pre_total: u64,
    system: Option<u64>,
    pre_system: Option<u64>,
    online_cpus: Option<u64>,
) -> Option<f64> {
    // Sem amostra anterior não existe delta — devolver algo aqui seria a média
    // desde o boot do container, que é exatamente o bug que o one_shot causava.
    let (system, pre_system) = (system?, pre_system?);
    if pre_system == 0 {
        return None;
    }

    let cpu_delta = total as f64 - pre_total as f64;
    let system_delta = system as f64 - pre_system as f64;
    if system_delta <= 0.0 || cpu_delta < 0.0 {
        return None;
    }

    let cpus = online_cpus.unwrap_or(1) as f64;
    Some((cpu_delta / system_delta) * cpus * 100.0)
}

fn memory_used(usage: u64, inactive_file: Option<u64>) -> u64 {
    usage.saturating_sub(inactive_file.unwrap_or(0))
}

fn bytes_to_mb(bytes: u64) -> f64 {
    bytes as f64 / 1024.0 / 1024.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_percent_usa_delta_entre_amostras() {
        // container usou 50 ticks de 1000 do sistema, em 4 CPUs → 20%
        let v = cpu_percent(1_050, 1_000, Some(11_000), Some(10_000), Some(4)).unwrap();
        assert!((v - 20.0).abs() < 1e-9);
    }

    #[test]
    fn cpu_percent_sem_amostra_anterior_e_none() {
        // precpu zerado (o que o one_shot=true devolve) não pode virar número
        assert_eq!(cpu_percent(5_000, 0, Some(100_000), Some(0), Some(4)), None);
        assert_eq!(cpu_percent(5_000, 0, Some(100_000), None, Some(4)), None);
    }

    #[test]
    fn cpu_percent_delta_invalido_e_none() {
        assert_eq!(cpu_percent(1_000, 1_000, Some(10_000), Some(10_000), Some(1)), None);
        assert_eq!(cpu_percent(900, 1_000, Some(11_000), Some(10_000), Some(1)), None);
    }

    #[test]
    fn cpu_percent_sem_contagem_de_cpus_assume_uma() {
        let v = cpu_percent(1_100, 1_000, Some(2_000), Some(1_000), None).unwrap();
        assert!((v - 10.0).abs() < 1e-9);
    }

    #[test]
    fn memoria_desconta_cache_inativo() {
        assert_eq!(memory_used(1_000, Some(300)), 700);
        assert_eq!(memory_used(1_000, None), 1_000);
        // nunca dá underflow se o daemon devolver algo inconsistente
        assert_eq!(memory_used(100, Some(300)), 0);
    }

    #[test]
    fn bytes_para_mb() {
        assert_eq!(bytes_to_mb(1024 * 1024), 1.0);
    }
}
