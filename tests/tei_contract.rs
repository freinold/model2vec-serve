#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Contract tests for TEI-compatible endpoints.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{test_app, test_app_with_state};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

#[tokio::test]
async fn tei_embed_returns_vector_list() {
    let app = test_app(None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/embed")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"inputs": "hello"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert!(value.is_array());
    assert_eq!(value.as_array().unwrap().len(), 1);
    assert!(value[0].is_array());
}

#[tokio::test]
async fn tei_info_returns_model_metadata() {
    let app = test_app(None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/info")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert!(value["model_id"].is_string());
    assert!(value["max_input_length"].is_u64());
    assert!(value["embedding_dimension"].is_u64());
}

#[tokio::test]
async fn tei_embed_without_model_uses_default() {
    let (app, state) = test_app_with_state(None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/embed")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"inputs": "hello"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert!(value.is_array());
    assert_eq!(value.as_array().unwrap().len(), 1);
    assert!(value[0].is_array());

    let default_model = state.registry.resolve(None).unwrap();
    assert_eq!(
        value[0].as_array().unwrap().len(),
        default_model.embedding_dimension
    );
}

#[tokio::test]
async fn tei_embed_with_model_query_param() {
    let (app, state) = test_app_with_state(None).await;
    let model_id = state.registry.default_model_id().to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/embed?model={model_id}"))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"inputs": "hello"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert!(value.is_array());
    assert_eq!(value.as_array().unwrap().len(), 1);
    assert!(value[0].is_array());
}

#[tokio::test]
async fn tei_info_without_model_uses_default() {
    let (app, state) = test_app_with_state(None).await;
    let default_model_id = state.registry.default_model_id().to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/info")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["model_id"].as_str().unwrap(), default_model_id);
}

#[tokio::test]
async fn tei_info_with_model_query_param() {
    let (app, state) = test_app_with_state(None).await;
    let model_id = state.registry.default_model_id().to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/info?model={model_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["model_id"].as_str().unwrap(), model_id);
}

#[tokio::test]
async fn tei_embed_with_unknown_model_returns_error() {
    let app = test_app(None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/embed?model=unknown-model")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"inputs": "hello"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["error"], "model_not_found");
    assert!(value["message"].as_str().unwrap().contains("unknown-model"));
}
