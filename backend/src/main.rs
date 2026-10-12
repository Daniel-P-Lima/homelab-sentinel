mod docker;
mod pihole;
mod routes;
mod state;

use axum::{routing::get, Router};
use state::AppState;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let state = AppState::new().await;

    // Sem CORS: o navegador sempre fala com a mesma origem — o Nginx em
    // produção e o proxy do Vite em dev repassam /api pra cá.
    let app = Router::new()
        .route("/api/health", get(routes::health))
        .route("/api/containers", get(routes::list_containers))
        .route("/api/pihole/summary", get(routes::pihole_summary))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = "0.0.0.0:8087";
    tracing::info!("homelab-sentinel backend listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
