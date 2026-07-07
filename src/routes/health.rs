//! Health and readiness endpoints.

use crate::routes::dto::{ErrorResponse, HealthStatus};
use axum::Json;

/// Health / readiness endpoint.
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy and ready", body = HealthStatus),
        (status = 503, description = "Service is not ready", body = ErrorResponse)
    )
)]
pub async fn health() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "healthy",
        ready: true,
        message: "model loaded and serving requests".to_string(),
    })
}

/// Readiness alias for Kubernetes probes.
#[utoipa::path(
    get,
    path = "/ready",
    tag = "health",
    responses(
        (status = 200, description = "Service is ready", body = HealthStatus),
        (status = 503, description = "Service is not ready", body = ErrorResponse)
    )
)]
pub async fn ready() -> Json<HealthStatus> {
    health().await
}
