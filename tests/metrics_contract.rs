#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Contract tests for the Prometheus metrics endpoint.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::test_app;
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn metrics_returns_prometheus_text() {
    let app = test_app(None).await;

    // Generate some traffic so the counters exist in the scrape output.
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8(body.to_vec()).unwrap();

    assert!(text.contains("http_requests_total"));
    assert!(text.contains("http_request_duration_seconds"));
}

#[tokio::test]
async fn metrics_include_model_label_for_embeddings() {
    let app = test_app(None).await;

    // Discover the loaded model identifier from the health endpoint.
    let health_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let health_body = health_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let health: serde_json::Value = serde_json::from_slice(&health_body).unwrap();
    let model_id = health["models"][0]["model_id"].as_str().unwrap();

    // Generate traffic on the embedding endpoint with the resolved model.
    let embed_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/embeddings")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({"input": "hello", "model": model_id}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(embed_response.status(), StatusCode::OK);

    let metrics_response = app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(metrics_response.status(), StatusCode::OK);

    let body = metrics_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let text = String::from_utf8(body.to_vec()).unwrap();

    // Verify the embedding endpoint metrics carry a `model` label with the
    // resolved identifier.
    let request_lines: Vec<&str> = text
        .lines()
        .filter(|line| {
            line.contains("http_requests_total{") && line.contains("path=\"/v1/embeddings\"")
        })
        .collect();
    assert!(
        !request_lines.is_empty(),
        "expected http_requests_total line for /v1/embeddings"
    );
    assert!(
        request_lines
            .iter()
            .any(|line| line.contains(&format!("model=\"{model_id}\""))),
        "expected http_requests_total to include a model label for {model_id}"
    );

    let duration_lines: Vec<&str> = text
        .lines()
        .filter(|line| {
            line.contains("http_request_duration_seconds{")
                && line.contains("path=\"/v1/embeddings\"")
        })
        .collect();
    assert!(
        !duration_lines.is_empty(),
        "expected http_request_duration_seconds line for /v1/embeddings"
    );
    assert!(
        duration_lines
            .iter()
            .any(|line| line.contains(&format!("model=\"{model_id}\""))),
        "expected http_request_duration_seconds to include a model label for {model_id}"
    );
}
