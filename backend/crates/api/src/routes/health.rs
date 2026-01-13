//! Health check endpoints.

use axum::{Json, Router, routing::get};
use serde::Serialize;

use crate::AppState;

/// Health check response.
#[derive(Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    /// Service status.
    #[schema(example = "healthy")]
    pub status: &'static str,
    /// Service version.
    #[schema(example = "0.1.0")]
    pub version: &'static str,
}

/// Health check handler.
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    ),
    tag = "Health"
)]
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// Creates health check routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}
