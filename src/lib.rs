//! status-service library.
//!
//! Configuration loading, shared application state, and the HTTP
//! router. The binary in `main.rs` wires these together.

pub mod app;
pub mod config;
pub mod state;

mod handlers;

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::config::{load_config, AppConfig, Environment};

    #[test]
    fn default_config_has_expected_values() {
        let config = AppConfig::default();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.monitoring.log_level, "info");
        assert_eq!(config.database.max_connections, 5);
    }

    #[test]
    fn config_round_trips_through_json() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).expect("serializes");
        let back: AppConfig = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(config.server.port, back.server.port);
    }

    #[test]
    fn load_config_reads_file_then_applies_env() {
        use std::env;
        use std::fs;
        use std::process;

        let mut path = env::temp_dir();
        path.push(format!("status-service-test-{}.yaml", process::id()));
        let yaml = concat!(
            "server:\n",
            "  host: \"127.0.0.1\"\n",
            "  port: 8080\n",
            "  environment: production\n",
            "monitoring:\n",
            "  log_level: \"debug\"\n",
            "database:\n",
            "  url: \"postgres://localhost/app\"\n",
            "  max_connections: 10\n",
        );
        let mut file = fs::File::create(&path).expect("creates temp file");
        file.write_all(yaml.as_bytes()).expect("writes temp file");

        let config = load_config(&path).expect("loads config");
        assert_eq!(config.server.port, 8080);
        assert!(matches!(config.server.environment, Environment::Production));
        assert_eq!(config.database.url, "postgres://localhost/app");

        env::set_var("APP_PORT", "9090");
        env::set_var("DATABASE_URL", "postgres://override/db");
        let overridden = load_config(&path).expect("loads config with env");
        assert_eq!(overridden.server.port, 9090);
        assert_eq!(overridden.database.url, "postgres://override/db");
        env::remove_var("APP_PORT");
        env::remove_var("DATABASE_URL");

        let _ = fs::remove_file(&path);
    }
}
