//! API route definitions.

use axum::{Router, middleware};

use crate::{AppState, middleware::auth::auth_middleware};

pub mod accounts;
pub mod auth;
pub mod currencies;
pub mod dimensions;
pub mod exchange_rates;
pub mod fiscal;
pub mod health;
pub mod organizations;
pub mod transactions;

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
            .merge(fiscal::routes())
            .merge(accounts::routes())
            .merge(dimensions::routes())
            .merge(exchange_rates::routes())
            .merge(currencies::routes())
            .merge(transactions::routes())
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
