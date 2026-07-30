use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

use crate::docker::{self, ContainerInfo};
use crate::state::AppState;

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

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

pub async fn list_containers(
    State(state): State<AppState>,
) -> Result<Json<Vec<ContainerInfo>>, (StatusCode, Json<ErrorResponse>)> {
    let Some(client) = state.inner.docker.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Docker daemon indisponível para este backend".to_string(),
            }),
        ));
    };

    match docker::list_containers(client).await {
        Ok(containers) => Ok(Json(containers)),
        Err(e) => {
            tracing::error!("erro ao listar containers: {e}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            ))
        }
    }
}

