//! Health check endpoints.

use axum::{Json, Router, routing::get};
use serde::Serialize;

use crate::AppState;

/// Health check response.
#[derive(Serialize)]
pub struct HealthResponse {
    /// Service status.
    pub status: &'static str,
    /// Service version.
    pub version: &'static str,
}

/// Health check handler.
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// Creates health check routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}
