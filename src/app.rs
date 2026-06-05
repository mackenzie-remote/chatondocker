//! Router construction.

use axum::routing::get;
use axum::Router;

use crate::handlers;
use crate::state::AppState;

/// Builds the application router with all routes and shared state.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthcheck", get(handlers::healthcheck))
        .route("/api/status", get(handlers::status))
        .with_state(state)
}
