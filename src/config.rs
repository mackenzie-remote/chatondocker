//! Configuration loading from a file with environment overrides.

use std::env;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Top-level application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub monitoring: MonitoringConfig,
    pub database: DatabaseConfig,
}

/// HTTP server settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub environment: Environment,
}

/// Logging settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub log_level: String,
}

/// Database connection settings. The connection string is supplied at
/// run time via the `DATABASE_URL` environment variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

/// Deployment environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_owned(),
                port: 3000,
                environment: Environment::Development,
            },
            monitoring: MonitoringConfig {
                log_level: "info".to_owned(),
            },
            database: DatabaseConfig {
                url: String::new(),
                max_connections: 5,
            },
        }
    }
}

/// Loads configuration from `path`, then applies environment overrides.
///
/// # Errors
///
/// Returns [`ConfigError`] if the file cannot be read or parsed, or if
/// an environment override holds an invalid value.
pub fn load_config(path: &Path) -> Result<AppConfig, ConfigError> {
    let raw = fs::read_to_string(path).map_err(|err| ConfigError::Read(err.to_string()))?;
    let mut config: AppConfig =
        serde_yaml::from_str(&raw).map_err(|err| ConfigError::Parse(err.to_string()))?;
    apply_env_overrides(&mut config)?;
    Ok(config)
}

/// Overrides config fields from environment variables when present.
fn apply_env_overrides(config: &mut AppConfig) -> Result<(), ConfigError> {
    if let Ok(host) = env::var("APP_HOST") {
        config.server.host = host;
    }
    if let Ok(port) = env::var("APP_PORT") {
        config.server.port = port
            .parse::<u16>()
            .map_err(|err| ConfigError::Parse(format!("APP_PORT: {err}")))?;
    }
    if let Ok(level) = env::var("APP_LOG_LEVEL") {
        config.monitoring.log_level = level;
    }
    if let Ok(url) = env::var("DATABASE_URL") {
        config.database.url = url;
    }
    Ok(())
}

/// Errors that can occur while loading configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Read(String),
    #[error("invalid config: {0}")]
    Parse(String),
}
