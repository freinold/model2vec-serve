#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Contract tests for the Prometheus metrics endpoint.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::test_app;
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn metrics_returns_prometheus_text() {
    let app = test_app(None).await;

    // Generate some traffic so the counters exist in the scrape output.
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8(body.to_vec()).unwrap();

    assert!(text.contains("http_requests_total"));
    assert!(text.contains("http_request_duration_seconds"));
}
