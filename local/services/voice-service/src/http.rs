//! HTTP surface: liveness and readiness for the voice container.
//!
//! `/health` is the container liveness probe; `/ready` reports ready as long
//! as the engine answers (provider state is intentionally not part of it —
//! an STT outage must fail a turn, not restart the pod).

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use tower_http::trace::TraceLayer;

use crate::engine::VoiceEngine;

#[derive(Clone)]
pub struct HttpState {
    pub engine: Arc<VoiceEngine>,
    pub environment: String,
}

pub fn app(state: HttpState) -> Router {
    Router::new()
        .route("/health", get(live))
        .route("/ready", get(ready))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub async fn serve(app: Router, addr: &str) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "http listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn live(State(state): State<HttpState>) -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "voice-service",
        "environment": state.environment,
    }))
}

async fn ready(State(state): State<HttpState>) -> Response {
    match state.engine.ping().await {
        Ok(()) => Json(json!({ "status": "ready", "service": "voice-service" })).into_response(),
        Err(err) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "degraded", "service": "voice-service", "error": err.to_string() })),
        )
            .into_response(),
    }
}
