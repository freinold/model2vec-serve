#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Contract tests for API key authentication.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::test_app;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn missing_api_key_is_rejected() {
    let app = test_app(Some("secret".to_string())).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/embeddings")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"input": "hello"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn valid_api_key_is_accepted() {
    let app = test_app(Some("secret".to_string())).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/embeddings")
                .header("Content-Type", "application/json")
                .header("Authorization", "Bearer secret")
                .body(Body::from(json!({"input": "hello"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
