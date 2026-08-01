#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Integration tests for loading and serving multiple models.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{metrics_handle, model_dir};
use http_body_util::BodyExt;
use model2vec_serve::{config::Config, routes::app, state::AppState};
use serde_json::{Value, json};
use std::collections::HashSet;
use tempfile::TempDir;
use tower::ServiceExt;

#[tokio::test]
async fn two_models_load_and_embed() {
    let original = model_dir();
    let temp = TempDir::new().expect("failed to create temp dir");

    let model_a = temp.path().join("model-a");
    let model_b = temp.path().join("model-b");
    std::os::unix::fs::symlink(&original, &model_a).expect("failed to create symlink model-a");
    std::os::unix::fs::symlink(&original, &model_b).expect("failed to create symlink model-b");

    let model_a_path = model_a.to_string_lossy().to_string();
    let model_b_path = model_b.to_string_lossy().to_string();

    let config = Config {
        host: "127.0.0.1".to_string(),
        port: 0,
        models: vec![model_a_path.clone(), model_b_path.clone()],
        default_model: Some(model_a_path),
        model_owner: "minishlab".to_string(),
        api_key: None,
        max_batch_size: 32,
        max_input_length: 512,
        log_level: "warn".to_string(),
        request_timeout_seconds: 30,
    };

    let state = AppState::new(config, metrics_handle()).expect("failed to load models");
    assert_eq!(
        state.registry.iter().count(),
        2,
        "registry should contain two loaded models"
    );

    let ids: HashSet<_> = state.registry.iter().map(|m| m.model_id.clone()).collect();
    assert!(ids.contains("model-a"), "model-a should be loaded");
    assert!(ids.contains("model-b"), "model-b should be loaded");

    let app = app(state);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/embeddings")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"input": "hello world"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();
    let data = value["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert!(
        !data[0]["embedding"].as_array().unwrap().is_empty(),
        "embedding should be non-empty"
    );
}

#[tokio::test]
async fn partial_failure_keeps_healthy_model_ready() {
    let valid = model_dir();
    let missing = "/tmp/definitely-not-a-real-model-path-12345".to_string();

    let config = Config {
        host: "127.0.0.1".to_string(),
        port: 0,
        models: vec![valid.clone(), missing],
        default_model: Some(valid),
        model_owner: "minishlab".to_string(),
        api_key: None,
        max_batch_size: 32,
        max_input_length: 512,
        log_level: "warn".to_string(),
        request_timeout_seconds: 30,
    };

    let state =
        AppState::new(config, metrics_handle()).expect("AppState should load with one valid model");
    assert_eq!(
        state.registry.iter().count(),
        1,
        "exactly one model should be loaded"
    );

    let app = app(state);
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

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();
    assert!(value["ready"].as_bool().unwrap());
}

#[tokio::test]
async fn first_configured_failure_falls_back_to_first_loaded_default() {
    let valid = model_dir();
    let missing = "/tmp/definitely-not-a-real-model-path-12345".to_string();

    let config = Config {
        host: "127.0.0.1".to_string(),
        port: 0,
        models: vec![missing, valid.clone()],
        default_model: None,
        model_owner: "minishlab".to_string(),
        api_key: None,
        max_batch_size: 32,
        max_input_length: 512,
        log_level: "warn".to_string(),
        request_timeout_seconds: 30,
    };

    let state = AppState::new(config, metrics_handle())
        .expect("AppState should load with first loaded model as default");
    assert_eq!(
        state.registry.iter().count(),
        1,
        "exactly one model should be loaded"
    );

    let loaded_id = state
        .registry
        .iter()
        .next()
        .map(|m| m.model_id.clone())
        .expect("one model should be loaded");
    assert_eq!(
        state.registry.default_model_id(),
        loaded_id,
        "default should be the first successfully loaded model"
    );
}
