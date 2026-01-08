//! Zeltra API Server
//!
//! Main entry point for the Zeltra backend service.

use std::sync::Arc;

use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use zeltra_api::{AppState, create_router};
use zeltra_db::connect;
use zeltra_shared::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "zeltra=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = AppConfig::load().expect("Failed to load configuration");

    // Connect to database
    let db = connect(&config.database.url).await?;
    info!("Connected to database");

    // Create application state
    let state = AppState {
        db: Arc::new(db),
        jwt_secret: config.jwt.secret,
    };

    // Create router
    let app = create_router(state);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await?;
    info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
