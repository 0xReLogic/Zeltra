//! HTTP API layer with Axum routes and middleware.
//!
//! This crate provides:
//! - REST API routes
//! - Authentication middleware
//! - Request extractors
//! - Response types

pub mod extractors;
pub mod middleware;
pub mod routes;

use axum::Router;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool.
    pub db: Arc<DatabaseConnection>,
    /// JWT configuration.
    pub jwt_secret: String,
}

/// Creates the main application router.
#[must_use]
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1", routes::api_routes())
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}
