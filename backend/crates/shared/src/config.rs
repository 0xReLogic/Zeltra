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
    /// Email configuration.
    #[serde(default)]
    pub email: EmailConfig,
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
    604_800 // 7 days
}

/// Email configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct EmailConfig {
    /// SMTP host.
    #[serde(default = "default_smtp_host")]
    pub smtp_host: String,
    /// SMTP port.
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,
    /// SMTP username.
    #[serde(default)]
    pub smtp_username: String,
    /// SMTP password.
    #[serde(default)]
    pub smtp_password: String,
    /// From email address.
    #[serde(default = "default_from_email")]
    pub from_email: String,
    /// From name.
    #[serde(default = "default_from_name")]
    pub from_name: String,
    /// Frontend URL for verification links.
    #[serde(default = "default_frontend_url")]
    pub frontend_url: String,
}

fn default_smtp_host() -> String {
    "localhost".to_string()
}

fn default_smtp_port() -> u16 {
    1025
}

fn default_from_email() -> String {
    "noreply@zeltra.app".to_string()
}

fn default_from_name() -> String {
    "Zeltra".to_string()
}

fn default_frontend_url() -> String {
    "http://localhost:3000".to_string()
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_host: default_smtp_host(),
            smtp_port: default_smtp_port(),
            smtp_username: String::new(),
            smtp_password: String::new(),
            from_email: default_from_email(),
            from_name: default_from_name(),
            frontend_url: default_frontend_url(),
        }
    }
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
