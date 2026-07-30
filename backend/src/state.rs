use std::sync::Arc;

/// Estado compartilhado entre handlers. Por enquanto vazio; a partir da
/// Fase 1 vai carregar o cliente Docker (bollard) e, na Fase 3, o pool
/// de conexões SQLite.
#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pub started_at: std::time::Instant,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                started_at: std::time::Instant::now(),
            }),
        }
    }
}
