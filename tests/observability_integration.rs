#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Integration tests for observability features.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::test_app;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn response_includes_request_id_header() {
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
    assert!(response.headers().contains_key("x-request-id"));
}
