//! Zeltra API Server
//!
//! Main entry point for the Zeltra backend service.

use std::sync::Arc;

use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use zeltra_api::{AppState, create_router};
use zeltra_db::connect;
use zeltra_shared::{AppConfig, EmailService, JwtConfig, JwtService};

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

    // Create JWT service
    let jwt_config = JwtConfig {
        secret: config.jwt.secret.clone(),
        #[allow(clippy::cast_possible_wrap)]
        access_token_expires_minutes: (config.jwt.access_token_expiry_secs / 60) as i64,
        #[allow(clippy::cast_possible_wrap)]
        refresh_token_expires_days: (config.jwt.refresh_token_expiry_secs / 86400) as i64,
    };
    let jwt_service = JwtService::new(jwt_config);

    // Create email service
    let email_service = EmailService::new(config.email.clone());
    info!(
        smtp_host = %config.email.smtp_host,
        smtp_port = %config.email.smtp_port,
        "Email service configured"
    );

    // Create application state
    let state = AppState {
        db: Arc::new(db),
        jwt_service: Arc::new(jwt_service),
        email_service: Arc::new(email_service),
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
