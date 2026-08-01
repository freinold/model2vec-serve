#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Contract tests for health endpoints.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{model_dir, test_app, test_app_with_models};
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

#[tokio::test]
async fn health_reports_failed_models() {
    let valid = model_dir();
    let missing = "/tmp/definitely-not-a-real-model-path-12345".to_string();
    let app = test_app_with_models(vec![missing.clone(), valid], None, None).await;

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

    let ready: Vec<_> = models.iter().filter(|m| m["status"] == "ready").collect();
    assert_eq!(ready.len(), 1, "exactly one ready model should be reported");

    let failed: Vec<_> = models.iter().filter(|m| m["status"] == "failed").collect();
    assert_eq!(
        failed.len(),
        1,
        "exactly one failed model should be reported"
    );
    assert_eq!(failed[0]["model_id"], missing);
    assert_eq!(failed[0]["message"], "model failed to load");
}
