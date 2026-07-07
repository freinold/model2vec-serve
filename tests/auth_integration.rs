#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Integration tests for API key authentication.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::test_app;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn invalid_api_key_is_rejected() {
    let app = test_app(Some("secret".to_string())).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/embeddings")
                .header("Content-Type", "application/json")
                .header("Authorization", "Bearer wrong")
                .body(Body::from(json!({"input": "hello"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_can_be_disabled() {
    let app = test_app(None).await;

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

    assert_eq!(response.status(), StatusCode::OK);
}
