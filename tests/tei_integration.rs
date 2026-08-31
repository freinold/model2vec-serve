#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Integration tests for TEI-compatible endpoints.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{alt_model_dir, metrics_handle, model_dir, test_app, test_app_with_models};
use http_body_util::BodyExt;
use model2vec_serve::{config::Config, routes::app, state::AppState};
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

#[tokio::test]
async fn retired_qualifier_on_root_embed_returns_invalid_request() {
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
    assert_eq!(value["error"], "invalid_request");
    assert!(value["message"].as_str().unwrap().contains("/tei/"));
}

#[tokio::test]
async fn per_model_paths_select_non_default_model() {
    let default_model = model_dir();
    let alt_model = alt_model_dir();
    let app = test_app_with_models(vec![default_model, alt_model], Some(model_dir()), None).await;

    let info_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/tei/alt-model/info")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(info_response.status(), StatusCode::OK);
    let info_body = info_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let info: Value = serde_json::from_slice(&info_body).unwrap();
    assert_eq!(info["model_id"].as_str().unwrap(), "alt-model");

    let embed_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tei/alt-model/embed")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"inputs": "hello"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(embed_response.status(), StatusCode::OK);
    let embed_body = embed_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let embed: Value = serde_json::from_slice(&embed_body).unwrap();
    assert!(embed.is_array());
    assert_eq!(embed.as_array().unwrap().len(), 1);
    assert!(embed[0].is_array());
}

#[tokio::test]
async fn root_embed_without_qualifier_serves_default_model() {
    let default_model = model_dir();
    let alt_model = alt_model_dir();

    let config = Config {
        host: "127.0.0.1".to_string(),
        port: 0,
        models: vec![default_model.clone(), alt_model],
        default_model: Some(default_model),
        model_owner: "minishlab".to_string(),
        model_alias: Vec::new(),
        api_key: None,
        max_batch_size: 32,
        max_input_length: 512,
        log_level: "warn".to_string(),
        request_timeout_seconds: 30,
    };

    let state = AppState::new(config, metrics_handle()).expect("failed to load models");
    let expected_dimension = state.registry.resolve(None).unwrap().embedding_dimension;

    let app = app(state);
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
    let vector = value[0].as_array().unwrap();
    assert_eq!(
        vector.len(),
        expected_dimension,
        "root /embed vector length should match the default model's embedding dimension"
    );
}
