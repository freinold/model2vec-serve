#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Contract tests for health endpoints.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::test_app;
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_healthy_status() {
    let app = test_app(None).await;

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

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["status"], "healthy");
    assert_eq!(value["ready"], true);

    let models = value["models"]
        .as_array()
        .expect("models should be an array");
    assert!(
        !models.is_empty(),
        "health response should include at least one model status"
    );
    let first = &models[0];
    assert!(first["model_id"].is_string());
    assert_eq!(first["status"], "ready");
    assert_eq!(first["message"], "model loaded");
}

#[tokio::test]
async fn ready_alias_returns_healthy_status() {
    let app = test_app(None).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
