//! Health and readiness endpoints.

use crate::{
    routes::dto::{ErrorResponse, HealthModelStatus, HealthStatus},
    state::AppState,
};
use axum::{Json, extract::State, http::StatusCode};
use std::sync::Arc;

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
pub async fn health(State(state): State<Arc<AppState>>) -> (StatusCode, Json<HealthStatus>) {
    let models: Vec<HealthModelStatus> = state
        .registry
        .model_statuses()
        .into_iter()
        .map(|m| HealthModelStatus {
            model_id: m.model_id,
            status: m.status,
            message: m.message,
        })
        .collect();
    let ready = state.registry.loaded_count() > 0;
    let status_code = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    let status = if ready { "healthy" } else { "unhealthy" };
    let message = if ready {
        format!("{} model(s) ready", state.registry.loaded_count())
    } else {
        "no models loaded".to_string()
    };

    (
        status_code,
        Json(HealthStatus {
            status,
            ready,
            message,
            models,
        }),
    )
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
pub async fn ready(State(state): State<Arc<AppState>>) -> (StatusCode, Json<HealthStatus>) {
    health(State(state)).await
}
