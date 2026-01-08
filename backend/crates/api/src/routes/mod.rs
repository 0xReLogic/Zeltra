//! API route definitions.

use axum::{Router, middleware};

use crate::{AppState, middleware::auth::auth_middleware};

pub mod auth;
pub mod health;
pub mod organizations;

/// Creates the API router with all routes.
pub fn api_routes() -> Router<AppState> {
    Router::new().merge(health::routes()).merge(auth::routes())
}

/// Creates the API router with protected routes that need state for middleware.
#[allow(clippy::needless_pass_by_value)]
pub fn api_routes_with_state(state: AppState) -> Router<AppState> {
    // Protected routes that require authentication
    let protected_routes =
        Router::new()
            .merge(organizations::routes())
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ));

    // Combine public and protected routes
    Router::new()
        .merge(health::routes())
        .merge(auth::routes())
        .merge(protected_routes)
}
