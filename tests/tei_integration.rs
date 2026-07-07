#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Integration tests for TEI-compatible endpoints.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::test_app;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

#[tokio::test]
async fn tei_batch_embed_returns_multiple_vectors() {
    let app = test_app(None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/embed")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({"inputs": ["one", "two", "three"]}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value.as_array().unwrap().len(), 3);
}
