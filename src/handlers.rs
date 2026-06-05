//! HTTP request handlers.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use crate::state::AppState;

/// Body returned by the status endpoint.
#[derive(Serialize)]
struct StatusBody {
    version: String,
    uptime_seconds: u64,
    region: String,
}

/// Body returned when the service cannot serve a request.
#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

/// Liveness and readiness check. Returns `200` only when the database
/// is reachable; otherwise `503`.
pub(crate) async fn healthcheck(State(state): State<AppState>) -> Response {
    match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => (StatusCode::OK, "OK").into_response(),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "database unavailable").into_response(),
    }
}

/// Reports version, uptime, and a value read from the database.
pub(crate) async fn status(State(state): State<AppState>) -> Response {
    let uptime_seconds = state.started_at.elapsed().as_secs();
    let region =
        sqlx::query_scalar::<_, String>("SELECT value FROM service_metadata WHERE key = 'region'")
            .fetch_one(&state.pool)
            .await;
    match region {
        Ok(region) => Json(StatusBody {
            version: state.version.to_owned(),
            uptime_seconds,
            region,
        })
        .into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorBody {
                error: "database unavailable".to_owned(),
            }),
        )
            .into_response(),
    }
}
