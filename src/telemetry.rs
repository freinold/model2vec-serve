//! Observability: structured logging, request IDs, and Prometheus metrics.

use axum::{
    extract::Request,
    http::header::{self, HeaderMap},
    middleware::Next,
    response::Response,
};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{Instrument, info_span};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

/// Initialize the global tracing subscriber.
pub fn init_tracing(log_level: &str) {
    let filter = EnvFilter::try_new(log_level).unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

/// Install the Prometheus metrics recorder and return the recorder handle.
///
/// # Panics
///
/// Panics if the recorder has already been installed or installation fails.
#[must_use]
pub fn init_metrics() -> metrics_exporter_prometheus::PrometheusHandle {
    PrometheusBuilder::new()
        .install_recorder()
        .unwrap_or_else(|_| panic!("failed to install Prometheus recorder"))
}

/// Extract or generate a request correlation id.
fn request_id(headers: &HeaderMap) -> String {
    headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map_or_else(
            || Uuid::new_v4().to_string(),
            std::string::ToString::to_string,
        )
}

/// Per-request model identifier passed through request extensions.
///
/// Embedding handlers set the resolved model id; the request tracing
/// middleware reads it when recording Prometheus metrics. Other endpoints
/// leave it unset, resulting in the `none` label value.
#[derive(Clone, Debug)]
pub struct RequestModelId {
    inner: Arc<Mutex<Option<String>>>,
}

impl RequestModelId {
    /// Create a new empty model identifier holder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    /// Store the resolved model identifier.
    pub fn set(&self, model_id: String) {
        *self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(model_id);
    }

    fn get(&self) -> Option<String> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl Default for RequestModelId {
    fn default() -> Self {
        Self::new()
    }
}

/// Tower/axum middleware that adds a request id, logs requests, and records metrics.
pub async fn request_tracing_middleware(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let id = request_id(request.headers());
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    let model_id = RequestModelId::new();
    let mut request = request;
    request.extensions_mut().insert(model_id.clone());

    let span = info_span!(
        "http_request",
        request_id = %id,
        method = %method,
        path = %path,
    );

    let response = next.run(request).instrument(span).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16();
    let model_label = model_id.get().unwrap_or_else(|| "none".to_string());

    metrics::counter!(
        "http_requests_total",
        "method" => method.clone(),
        "path" => path.clone(),
        "status" => status.to_string(),
        "model" => model_label.clone(),
    )
    .increment(1);
    metrics::histogram!(
        "http_request_duration_seconds",
        "method" => method,
        "path" => path,
        "model" => model_label,
    )
    .record(latency);

    if response.status().is_server_error() {
        metrics::counter!("http_errors_total", "status" => status.to_string()).increment(1);
    }

    let mut response = response;
    response.headers_mut().insert(
        "x-request-id",
        header::HeaderValue::from_str(&id)
            .unwrap_or_else(|_| header::HeaderValue::from_static("unknown")),
    );
    response
}
