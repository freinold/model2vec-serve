#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Contract tests for TEI-compatible endpoints.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::test_app;
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
