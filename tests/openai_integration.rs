#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Integration tests for the OpenAI embeddings endpoint.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::test_app;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

#[tokio::test]
async fn single_string_request_returns_one_embedding() {
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
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn batch_request_returns_embeddings_in_order() {
    let app = test_app(None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/embeddings")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({"input": ["first sentence", "second sentence"]}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();
    let data = value["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0]["index"], 0);
    assert_eq!(data[1]["index"], 1);
}
