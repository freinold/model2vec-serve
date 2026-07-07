#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Integration tests for health endpoints.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::test_app;
use tower::ServiceExt;

#[tokio::test]
async fn health_responds_within_one_second() {
    let app = test_app(None).await;
    let start = std::time::Instant::now();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(start.elapsed() < std::time::Duration::from_secs(1));
}
