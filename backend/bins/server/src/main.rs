//! Zeltra API Server
//!
//! Main entry point for the Zeltra backend service.

use std::sync::Arc;

use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use zeltra_api::{AppState, create_router};
use zeltra_core::storage::{StorageConfig, StorageProvider, StorageService};
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

    // Create storage service (optional, based on environment)
    let storage = create_storage_service();

    // Create application state
    let state = AppState {
        db: Arc::new(db),
        jwt_service: Arc::new(jwt_service),
        email_service: Arc::new(email_service),
        storage,
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

/// Create storage service from environment variables.
///
/// Supports:
/// - `STORAGE_TYPE=local` with `STORAGE_LOCAL_PATH` (default: ./uploads)
/// - `STORAGE_TYPE=s3` with S3-compatible config (R2, Supabase, AWS)
/// - `STORAGE_TYPE=azure` with Azure Blob config
fn create_storage_service() -> Option<Arc<StorageService>> {
    let storage_type = std::env::var("STORAGE_TYPE").unwrap_or_default();

    let config = match storage_type.as_str() {
        "s3" => {
            let endpoint = std::env::var("STORAGE_S3_ENDPOINT").ok()?;
            let bucket = std::env::var("STORAGE_S3_BUCKET").ok()?;
            let access_key = std::env::var("STORAGE_S3_ACCESS_KEY").ok()?;
            let secret_key = std::env::var("STORAGE_S3_SECRET_KEY").ok()?;
            let region = std::env::var("STORAGE_S3_REGION").unwrap_or_else(|_| "auto".to_string());

            info!(
                endpoint = %endpoint,
                bucket = %bucket,
                region = %region,
                "Configuring S3-compatible storage"
            );

            StorageConfig::new(StorageProvider::s3(
                endpoint, bucket, access_key, secret_key, region,
            ))
        }
        "azure" => {
            let account = std::env::var("STORAGE_AZURE_ACCOUNT").ok()?;
            let access_key = std::env::var("STORAGE_AZURE_ACCESS_KEY").ok()?;
            let container = std::env::var("STORAGE_AZURE_CONTAINER").ok()?;

            info!(
                account = %account,
                container = %container,
                "Configuring Azure Blob storage"
            );

            StorageConfig::new(StorageProvider::azure_blob(account, access_key, container))
        }
        "local" => {
            let path =
                std::env::var("STORAGE_LOCAL_PATH").unwrap_or_else(|_| "./uploads".to_string());

            info!(path = %path, "Configuring local filesystem storage");

            StorageConfig::new(StorageProvider::local_fs(&path))
        }
        _ => {
            info!("No storage configured (STORAGE_TYPE not set)");
            return None;
        }
    };

    match StorageService::from_config(config) {
        Ok(service) => {
            info!("Storage service initialized");
            Some(Arc::new(service))
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to initialize storage service");
            None
        }
    }
}
