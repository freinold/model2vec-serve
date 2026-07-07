//! Prometheus metrics scrape endpoint.

use crate::state::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode, header},
};
use std::sync::Arc;

/// Prometheus metrics endpoint.
///
/// # Panics
///
/// Panics only if the axum response builder is misconfigured, which cannot
/// happen for a valid status code and body.
#[utoipa::path(
    get,
    path = "/metrics",
    tag = "observability",
    responses(
        (status = 200, description = "Prometheus metrics", body = String),
        (status = 500, description = "Internal error", body = String)
    )
)]
pub async fn metrics(State(state): State<Arc<AppState>>) -> Response<Body> {
    let body = state.metrics_handle.render();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; version=0.0.4")
        .body(Body::from(body))
        .unwrap_or_else(|_| panic!("failed to build metrics response"))
}
