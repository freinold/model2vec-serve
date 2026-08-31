#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Integration tests for API key authentication.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{test_app, test_app_with_state};
use http_body_util::BodyExt;
use serde_json::{Value, json};
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

#[tokio::test]
async fn tei_per_model_endpoints_require_api_key() {
    let (app, state) = test_app_with_state(Some("secret".to_string())).await;
    let path_id = state
        .registry
        .path_identifier_for(state.registry.default_model_id())
        .expect("default model should have a path identifier");

    let unauthenticated = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/tei/{path_id}/embed"))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"inputs": "hello"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

    let wrong_key = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/tei/{path_id}/embed"))
                .header("Content-Type", "application/json")
                .header("Authorization", "Bearer wrong")
                .body(Body::from(json!({"inputs": "hello"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(wrong_key.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn tei_per_model_endpoints_accept_valid_api_key() {
    let (app, state) = test_app_with_state(Some("secret".to_string())).await;
    let path_id = state
        .registry
        .path_identifier_for(state.registry.default_model_id())
        .expect("default model should have a path identifier");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/tei/{path_id}/embed"))
                .header("Content-Type", "application/json")
                .header("Authorization", "Bearer secret")
                .body(Body::from(json!({"inputs": "hello"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();
    assert!(
        !value[0].as_array().unwrap().is_empty(),
        "embedding should be non-empty"
    );
}

#[tokio::test]
async fn health_remains_public_with_api_key() {
    let app = test_app(Some("secret".to_string())).await;

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
}
