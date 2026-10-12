use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

use crate::docker::{self, ContainerInfo, DockerError};
use crate::state::AppState;

/// Erro de qualquer handler. Cada variante decide seu status HTTP em
/// `into_response`, e o corpo é sempre `{"error": "..."}` — o formato que o
/// `api.ts` do frontend espera.
#[derive(Debug, Error)]
pub enum AppError {
    /// Dependência externa não conectou no boot (ver `AppState::new`).
    #[error("{0} indisponível para este backend")]
    Unavailable(&'static str),
    #[error(transparent)]
    Docker(#[from] DockerError),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Docker(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        if status.is_server_error() {
            tracing::error!("{self}");
        }
        (status, Json(ErrorResponse { error: self.to_string() })).into_response()
    }
}

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    uptime_seconds: u64,
}

pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        uptime_seconds: state.inner.started_at.elapsed().as_secs(),
    })
}

pub async fn list_containers(
    State(state): State<AppState>,
) -> Result<Json<Vec<ContainerInfo>>, AppError> {
    let client = state
        .inner
        .docker
        .as_ref()
        .ok_or(AppError::Unavailable("Docker daemon"))?;

    Ok(Json(docker::list_containers(client).await?))
}
