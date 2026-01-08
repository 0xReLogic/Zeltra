//! Application configuration management.

use serde::Deserialize;

/// Application configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Server configuration.
    pub server: ServerConfig,
    /// Database configuration.
    pub database: DatabaseConfig,
    /// JWT configuration.
    pub jwt: JwtConfig,
}

/// Server configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to.
    #[serde(default = "default_host")]
    pub host: String,
    /// Port to listen on.
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

/// Database configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL.
    pub url: String,
    /// Maximum number of connections in the pool.
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    /// Minimum number of connections in the pool.
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
}

fn default_max_connections() -> u32 {
    10
}

fn default_min_connections() -> u32 {
    1
}

/// JWT configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    /// Secret key for signing tokens.
    pub secret: String,
    /// Access token expiration in seconds.
    #[serde(default = "default_access_token_expiry")]
    pub access_token_expiry_secs: u64,
    /// Refresh token expiration in seconds.
    #[serde(default = "default_refresh_token_expiry")]
    pub refresh_token_expiry_secs: u64,
}

fn default_access_token_expiry() -> u64 {
    900 // 15 minutes
}

fn default_refresh_token_expiry() -> u64 {
    604800 // 7 days
}

impl AppConfig {
    /// Loads configuration from environment and config files.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration cannot be loaded.
    pub fn load() -> Result<Self, config::ConfigError> {
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".to_string());

        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::File::with_name(&format!("config/{run_mode}")).required(false))
            .add_source(config::Environment::with_prefix("ZELTRA").separator("__"))
            .build()?;

        config.try_deserialize()
    }
}
