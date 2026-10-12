use std::sync::Arc;

use bollard::Docker;

use crate::pihole::PiholeClient;

/// Estado compartilhado entre handlers.
#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pub started_at: std::time::Instant,
    /// None quando o Docker daemon não respondeu ao ping no boot (ex: rodando
    /// fora de um ambiente com Docker disponível). Os handlers tratam esse
    /// caso retornando 503 em vez de dar panic.
    pub docker: Option<Docker>,
    /// None só quando `PIHOLE_URL`/`PIHOLE_PASSWORD` não estão configurados.
    pub pihole: Option<PiholeClient>,
}

impl AppState {
    pub async fn new() -> Self {
        let docker = match crate::docker::connect().await {
            Ok(d) => {
                tracing::info!("conectado ao Docker daemon");
                Some(d)
            }
            Err(e) => {
                tracing::warn!("não foi possível conectar ao Docker daemon: {e}. Endpoint /api/containers vai retornar 503.");
                None
            }
        };

        // Diferente do Docker, um login que falha no boot não desliga o
        // Pi-hole: o cliente reloga sozinho a cada request, então se o Pi-hole
        // subir depois do Sentinel (ou a senha for corrigida no Pi-hole), o
        // card volta sem reiniciar nada. Enquanto isso o endpoint dá 502.
        let pihole = PiholeClient::from_env();
        match &pihole {
            Some(client) => match client.check().await {
                Ok(()) => tracing::info!("conectado ao Pi-hole em {}", client.base_url()),
                Err(e) => tracing::warn!("Pi-hole em {} não respondeu no boot: {e}", client.base_url()),
            },
            None => tracing::warn!(
                "PIHOLE_URL/PIHOLE_PASSWORD não configurados. Endpoint /api/pihole/summary vai retornar 503."
            ),
        }

        Self {
            inner: Arc::new(AppStateInner {
                started_at: std::time::Instant::now(),
                docker,
                pihole,
            }),
        }
    }
}
