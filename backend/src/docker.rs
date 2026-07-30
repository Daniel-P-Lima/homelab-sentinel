use bollard::container::{ListContainersOptions, Stats, StatsOptions};
use bollard::Docker;
use futures_util::stream::StreamExt;
use serde::Serialize;
use thiserror::Error;

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

/// Conecta no Docker via socket local (`/var/run/docker.sock`).
/// Em produção, esse socket é montado read-only no container do backend
/// (ver docker-compose.snippet.yml).
pub fn connect() -> Result<Docker, DockerError> {
    Docker::connect_with_socket_defaults().map_err(DockerError::Connect)
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

    let mut result = Vec::with_capacity(containers.len());

    for c in containers {
        let id = c.id.clone().unwrap_or_default();
        let name = c
            .names
            .as_ref()
            .and_then(|names| names.first())
            .map(|n| n.trim_start_matches('/').to_string())
            .unwrap_or_else(|| id.clone());
        let image = c.image.clone().unwrap_or_default();
        let state = c.state.clone().unwrap_or_default();
        let status = c.status.clone().unwrap_or_default();

        // Stats só fazem sentido pra containers rodando; puxar stats de um
        // container parado retorna erro/vazio no daemon.
        let (cpu_percent, mem_usage_mb, mem_limit_mb) = if state == "running" && !id.is_empty() {
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

        result.push(ContainerInfo {
            id,
            name,
            image,
            state,
            status,
            cpu_percent,
            mem_usage_mb,
            mem_limit_mb,
        });
    }

    // Ordena por nome pra UI ficar estável entre polls
    result.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(result)
}

/// Pega uma única amostra de stats (stream=false) e calcula CPU% da mesma
/// forma que o `docker stats` faz: delta de uso / delta do sistema * nº de
/// CPUs * 100.
async fn fetch_stats(
    docker: &Docker,
    id: &str,
) -> Result<(Option<f64>, Option<f64>, Option<f64>), bollard::errors::Error> {
    let options = StatsOptions {
        stream: false,
        one_shot: true,
    };

    let mut stream = docker.stats(id, Some(options));
    let stats: Stats = match stream.next().await {
        Some(Ok(s)) => s,
        Some(Err(e)) => return Err(e),
        None => return Ok((None, None, None)),
    };

    let cpu_percent = calculate_cpu_percent(&stats);
    let (mem_usage_mb, mem_limit_mb) = calculate_memory_mb(&stats);

    Ok((cpu_percent, mem_usage_mb, mem_limit_mb))
}

fn calculate_cpu_percent(stats: &Stats) -> Option<f64> {
    let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64
        - stats.precpu_stats.cpu_usage.total_usage as f64;
    let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64
        - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;

    if system_delta <= 0.0 || cpu_delta < 0.0 {
        return None;
    }

    let cpu_count = stats
        .cpu_stats
        .online_cpus
        .or_else(|| {
            stats
                .cpu_stats
                .cpu_usage
                .percpu_usage
                .as_ref()
                .map(|v| v.len() as u64)
        })
        .unwrap_or(1) as f64;

    Some((cpu_delta / system_delta) * cpu_count * 100.0)
}

fn calculate_memory_mb(stats: &Stats) -> (Option<f64>, Option<f64>) {
    let usage = stats.memory_stats.usage;
    let limit = stats.memory_stats.limit;

    let usage_mb = usage.map(|u| u as f64 / 1024.0 / 1024.0);
    let limit_mb = limit.map(|l| l as f64 / 1024.0 / 1024.0);

    (usage_mb, limit_mb)
}
