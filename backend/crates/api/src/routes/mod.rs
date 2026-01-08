//! API route definitions.

use axum::Router;

use crate::AppState;

pub mod health;

// Route modules will be added as they are implemented
// pub mod auth;
// pub mod accounts;
// pub mod transactions;
// pub mod budgets;
// pub mod reports;
// pub mod simulation;

/// Creates the API router with all routes.
pub fn api_routes() -> Router<AppState> {
    Router::new()
        .merge(health::routes())
    // Add more routes as they are implemented
    // .merge(auth::routes())
    // .merge(accounts::routes())
    // .merge(transactions::routes())
}
