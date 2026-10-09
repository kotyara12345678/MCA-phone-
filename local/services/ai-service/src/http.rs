//! HTTP surface: liveness, readiness and the Prometheus exposition endpoint.
//!
//! `/health` is the container liveness probe; `/ready` verifies the datastore
//! actually answers; `/metrics` exposes the registry when metrics are enabled.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use tower_http::trace::TraceLayer;

use crate::engine::DialogueEngine;

#[derive(Clone)]
pub struct HttpState {
    pub engine: Arc<DialogueEngine>,
    pub environment: String,
}

pub fn app(state: HttpState) -> Router {
    Router::new()
        .route("/health", get(live))
        .route("/ready", get(ready))
        .route("/metrics", get(prometheus_metrics))
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
        "service": "ai-service",
        "environment": state.environment,
    }))
}

async fn ready(State(state): State<HttpState>) -> Response {
    match state.engine.ping().await {
        Ok(()) => Json(json!({ "status": "ready", "service": "ai-service" })).into_response(),
        Err(err) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(
                json!({ "status": "degraded", "service": "ai-service", "error": err.to_string() }),
            ),
        )
            .into_response(),
    }
}

async fn prometheus_metrics(State(state): State<HttpState>) -> Response {
    match state.engine.metrics_text() {
        Some(body) => (StatusCode::OK, body).into_response(),
        None => (
            StatusCode::OK,
            "# ai-service metrics disabled\n".to_string(),
        )
            .into_response(),
    }
}
