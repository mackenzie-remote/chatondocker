//! Shared application state passed to every request handler.

use std::time::Instant;

use sqlx::PgPool;

/// State shared across handlers.
#[derive(Clone)]
pub struct AppState {
    /// Connection pool for the backing Postgres database.
    pub pool: PgPool,
    /// Instant the process started, used to compute uptime.
    pub started_at: Instant,
    /// Service version from `CARGO_PKG_VERSION`.
    pub version: &'static str,
}
