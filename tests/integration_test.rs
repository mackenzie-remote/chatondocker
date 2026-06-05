//! Integration tests for the HTTP layer.
//!
//! These tests do not require a running database. With no reachable
//! database the endpoints return `503`, which still exercises routing,
//! state wiring, and dynamic (non-hardcoded) port binding.

use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use status_service::app::build_router;
use status_service::state::AppState;
use tokio::net::TcpListener;
use tower::ServiceExt;

/// Builds state whose pool points at a port nothing listens on, so any
/// query fails fast.
fn unreachable_db_state() -> AppState {
    let options = PgConnectOptions::new()
        .host("127.0.0.1")
        .port(1)
        .database("nonexistent");
    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(1))
        .connect_lazy_with(options);
    AppState {
        pool,
        started_at: Instant::now(),
        version: "test",
    }
}

#[tokio::test]
async fn healthcheck_reports_unavailable_without_db() {
    let app = build_router(unreachable_db_state());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthcheck")
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("router responds");
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn status_reports_unavailable_without_db() {
    let app = build_router(unreachable_db_state());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/status")
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("router responds");
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn server_binds_ephemeral_port() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("binds ephemeral port");
    let addr = listener.local_addr().expect("has local addr");
    let app = build_router(unreachable_db_state());
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    let url = format!("http://{addr}/healthcheck");
    let response = reqwest::get(&url).await.expect("request succeeds");
    assert_eq!(response.status(), reqwest::StatusCode::SERVICE_UNAVAILABLE);
}
