#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Contract tests for the OpenAI-compatible embeddings endpoint.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::test_app;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

/// Query `/v1/models` and return the id of the first (and usually only) loaded model.
async fn fixture_model_id(app: axum::Router) -> String {
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/models")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();

    value["data"][0]["id"]
        .as_str()
        .expect("models list should contain a model id")
        .to_string()
}

#[tokio::test]
async fn openai_models_lists_loaded_models() {
    let app = test_app(None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/models")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["object"], "list");
    assert!(value["data"].is_array());

    let data = value["data"].as_array().unwrap();
    assert!(!data.is_empty());

    let model = &data[0];
    assert!(model["id"].is_string());
    assert_eq!(model["object"], "model");
    assert!(model["created"].is_number());
    assert!(model["owned_by"].is_string());
}

#[tokio::test]
async fn openai_embeddings_with_model_returns_requested_model() {
    let app = test_app(None).await;
    let model_id = fixture_model_id(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/embeddings")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({"input": "Hello world", "model": model_id}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["model"], model_id);
}

#[tokio::test]
async fn openai_embeddings_without_model_uses_default() {
    let app = test_app(None).await;
    let default_model_id = fixture_model_id(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/embeddings")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"input": "Hello world"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["model"], default_model_id);
}

#[tokio::test]
async fn openai_embeddings_with_unknown_model_returns_error() {
    let app = test_app(None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/embeddings")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({"input": "Hello world", "model": "definitely-not-loaded"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["error"], "model_not_found");
}

#[tokio::test]
async fn openai_embeddings_supports_batch_input() {
    let app = test_app(None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/embeddings")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"input": ["a", "b"]}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn openai_embeddings_rejects_empty_input() {
    let app = test_app(None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/embeddings")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"input": []}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
