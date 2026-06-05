//! status-service binary entry point.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Context;
use clap::Parser;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use status_service::app::build_router;
use status_service::config::load_config;
use status_service::state::AppState;
use tokio::net::TcpListener;
use tokio::select;
use tokio::signal;

/// Command-line arguments.
#[derive(Parser)]
#[command(version, about = "A small status service")]
struct Cli {
    /// Path to the configuration file.
    #[arg(long, default_value = "config.yaml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = load_config(&cli.config).context("failed to load configuration")?;
    init_tracing(&config.monitoring.log_level);

    // An empty URL means the secret was not injected: start anyway and
    // report unhealthy. A non-empty but malformed URL is a real
    // misconfiguration, so fail fast.
    let db_options = if config.database.url.is_empty() {
        tracing::warn!("database url not configured; service will report unhealthy");
        PgConnectOptions::new()
    } else {
        config
            .database
            .url
            .parse::<PgConnectOptions>()
            .context("invalid database connection string")?
    };
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect_lazy_with(db_options);

    let state = AppState {
        pool,
        started_at: Instant::now(),
        version: env!("CARGO_PKG_VERSION"),
    };
    let router = build_router(state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;
    tracing::info!(
        "status-service ({:?}) listening on {addr}",
        config.server.environment
    );

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;
    Ok(())
}

/// Initializes tracing from the configured log level, falling back to
/// `info` when the level cannot be parsed.
fn init_tracing(level: &str) {
    let filter = tracing_subscriber::EnvFilter::try_new(level)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

/// Resolves when the process receives SIGINT or SIGTERM.
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut term) = signal::unix::signal(signal::unix::SignalKind::terminate()) {
            term.recv().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
