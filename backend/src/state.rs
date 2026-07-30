use std::sync::Arc;

use bollard::Docker;

/// Estado compartilhado entre handlers.
#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pub started_at: std::time::Instant,
    /// None quando não foi possível conectar no Docker socket (ex: rodando
    /// fora de um ambiente com Docker disponível). Os handlers tratam esse
    /// caso retornando 503 em vez de dar panic.
    pub docker: Option<Docker>,
}

impl AppState {
    pub fn new() -> Self {
        let docker = match crate::docker::connect() {
            Ok(d) => {
                tracing::info!("conectado ao Docker daemon");
                Some(d)
            }
            Err(e) => {
                tracing::warn!("não foi possível conectar ao Docker daemon: {e}. Endpoint /api/containers vai retornar 503.");
                None
            }
        };

        Self {
            inner: Arc::new(AppStateInner {
                started_at: std::time::Instant::now(),
                docker,
            }),
        }
    }
}

