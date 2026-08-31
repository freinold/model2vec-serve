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
use model2vec_serve::{config::Config, model::path_identifier, routes::app, state::AppState};
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
        model_alias: Vec::new(),
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
        model_alias: Vec::new(),
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
        model_alias: Vec::new(),
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

#[tokio::test]
async fn path_identifier_uses_last_segment() {
    assert_eq!(
        path_identifier("minishlab/potion-base-2M"),
        "potion-base-2M",
        "path identifier should be the substring after the final '/'"
    );
    assert_eq!(
        path_identifier("model-a"),
        "model-a",
        "id without '/' should be used verbatim"
    );
    assert_eq!(
        path_identifier("a/b/c/d"),
        "d",
        "only the last segment should be kept"
    );
}

#[tokio::test]
async fn path_identifiers_are_derived_per_model() {
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
        model_alias: Vec::new(),
        api_key: None,
        max_batch_size: 32,
        max_input_length: 512,
        log_level: "warn".to_string(),
        request_timeout_seconds: 30,
    };

    let state = AppState::new(config, metrics_handle()).expect("failed to load models");

    let served_a = state
        .registry
        .get_by_path("model-a")
        .expect("model-a should resolve via its derived path identifier");
    assert_eq!(served_a.model_id, "model-a");

    let served_b = state
        .registry
        .get_by_path("model-b")
        .expect("model-b should resolve via its derived path identifier");
    assert_eq!(served_b.model_id, "model-b");

    assert_eq!(
        state.registry.path_identifier_for("model-a"),
        Some("model-a"),
        "path identifier for 'model-a' should be its last path segment"
    );

    assert!(
        state.registry.get_by_path("missing").is_none(),
        "unknown path identifier should not resolve to a model"
    );
}

#[tokio::test]
async fn alias_overrides_path_identifier() {
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
        model_alias: vec![("model-a".to_string(), "alpha".to_string())],
        api_key: None,
        max_batch_size: 32,
        max_input_length: 512,
        log_level: "warn".to_string(),
        request_timeout_seconds: 30,
    };

    let state = AppState::new(config, metrics_handle()).expect("failed to load models");

    let served = state
        .registry
        .get_by_path("alpha")
        .expect("alias 'alpha' should resolve to a model");
    assert_eq!(
        served.model_id, "model-a",
        "alias 'alpha' should resolve to the model configured as 'model-a'"
    );

    assert!(
        state.registry.get_by_path("model-a").is_none(),
        "the alias should replace the derived path identifier"
    );
}

#[tokio::test]
async fn duplicate_path_identifier_aborts_startup() {
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
        model_alias: vec![("model-b".to_string(), "model-a".to_string())],
        api_key: None,
        max_batch_size: 32,
        max_input_length: 512,
        log_level: "warn".to_string(),
        request_timeout_seconds: 30,
    };

    let Err(err) = AppState::new(config, metrics_handle()) else {
        panic!("startup should fail on duplicate path identifiers");
    };
    let message = format!("{err:#}");
    assert!(
        message.contains("model-a") && message.contains("model-b"),
        "error should name both conflicting canonical ids: {message}"
    );
    assert!(
        message.contains("--model-alias"),
        "error should hint at --model-alias: {message}"
    );
}

#[tokio::test]
async fn three_models_reachable_via_own_paths() {
    let original = model_dir();
    let temp = TempDir::new().expect("failed to create temp dir");

    let model_a = temp.path().join("model-a");
    let model_b = temp.path().join("model-b");
    let model_c = temp.path().join("model-c");
    std::os::unix::fs::symlink(&original, &model_a).expect("failed to create symlink model-a");
    std::os::unix::fs::symlink(&original, &model_b).expect("failed to create symlink model-b");
    std::os::unix::fs::symlink(&original, &model_c).expect("failed to create symlink model-c");

    let model_a_path = model_a.to_string_lossy().to_string();
    let model_b_path = model_b.to_string_lossy().to_string();
    let model_c_path = model_c.to_string_lossy().to_string();

    let config = Config {
        host: "127.0.0.1".to_string(),
        port: 0,
        models: vec![
            model_a_path.clone(),
            model_b_path.clone(),
            model_c_path.clone(),
        ],
        default_model: Some(model_a_path),
        model_owner: "minishlab".to_string(),
        model_alias: Vec::new(),
        api_key: None,
        max_batch_size: 32,
        max_input_length: 512,
        log_level: "warn".to_string(),
        request_timeout_seconds: 30,
    };

    let state = AppState::new(config, metrics_handle()).expect("failed to load models");
    assert_eq!(
        state.registry.iter().count(),
        3,
        "registry should contain three loaded models"
    );

    let mut path_ids: Vec<&str> = state
        .registry
        .iter()
        .map(|m| state.registry.path_identifier_for(&m.model_id).unwrap())
        .collect();
    path_ids.sort_unstable();
    assert_eq!(
        path_ids,
        vec!["model-a", "model-b", "model-c"],
        "each model should have its own derived path identifier"
    );

    let app = app(state);

    for path_id in ["model-a", "model-b", "model-c"] {
        let response = app
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
        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let value: Value = serde_json::from_slice(&body).unwrap();
        let embedding = value[0]
            .as_array()
            .unwrap_or_else(|| panic!("response for {path_id} should be an array of embeddings"));
        assert!(
            !embedding.is_empty(),
            "embedding for {path_id} should be non-empty"
        );
    }
}

#[tokio::test]
async fn per_model_paths_are_isolated() {
    let original = model_dir();
    let temp = TempDir::new().expect("failed to create temp dir");

    let model_a = temp.path().join("model-a");
    let model_b = temp.path().join("model-b");
    let model_c = temp.path().join("model-c");
    std::os::unix::fs::symlink(&original, &model_a).expect("failed to create symlink model-a");
    std::os::unix::fs::symlink(&original, &model_b).expect("failed to create symlink model-b");
    std::os::unix::fs::symlink(&original, &model_c).expect("failed to create symlink model-c");

    let model_a_path = model_a.to_string_lossy().to_string();
    let model_b_path = model_b.to_string_lossy().to_string();
    let model_c_path = model_c.to_string_lossy().to_string();

    let config = Config {
        host: "127.0.0.1".to_string(),
        port: 0,
        models: vec![
            model_a_path.clone(),
            model_b_path.clone(),
            model_c_path.clone(),
        ],
        default_model: Some(model_a_path),
        model_owner: "minishlab".to_string(),
        model_alias: Vec::new(),
        api_key: None,
        max_batch_size: 32,
        max_input_length: 512,
        log_level: "warn".to_string(),
        request_timeout_seconds: 30,
    };

    let state = AppState::new(config, metrics_handle()).expect("failed to load models");

    let served_a = state
        .registry
        .get_by_path("model-a")
        .expect("model-a should resolve via its derived path identifier");
    assert_eq!(served_a.model_id, "model-a");

    let served_b = state
        .registry
        .get_by_path("model-b")
        .expect("model-b should resolve via its derived path identifier");
    assert_eq!(served_b.model_id, "model-b");

    let served_c = state
        .registry
        .get_by_path("model-c")
        .expect("model-c should resolve via its derived path identifier");
    assert_eq!(served_c.model_id, "model-c");

    assert!(
        state.registry.get_by_path("alt-model").is_none(),
        "a path identifier that was never configured should not resolve"
    );
}

#[tokio::test]
async fn unmatched_alias_key_aborts_startup() {
    let model = model_dir();

    let config = Config {
        host: "127.0.0.1".to_string(),
        port: 0,
        models: vec![model.clone()],
        default_model: Some(model),
        model_owner: "minishlab".to_string(),
        model_alias: vec![("does-not-exist".to_string(), "whatever".to_string())],
        api_key: None,
        max_batch_size: 32,
        max_input_length: 512,
        log_level: "warn".to_string(),
        request_timeout_seconds: 30,
    };

    let Err(err) = AppState::new(config, metrics_handle()) else {
        panic!("startup should fail on unmatched alias key");
    };
    let message = format!("{err:#}");
    assert!(
        message.contains("does-not-exist"),
        "error should name the unmatched alias key: {message}"
    );
}

#[tokio::test]
async fn each_model_info_reports_its_own_metadata() {
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
        model_alias: Vec::new(),
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

    let expected: Vec<(String, String)> = state
        .registry
        .iter()
        .map(|m| {
            (
                state
                    .registry
                    .path_identifier_for(&m.model_id)
                    .unwrap()
                    .to_string(),
                m.model_id.clone(),
            )
        })
        .collect();
    assert_eq!(
        expected.len(),
        2,
        "both models should expose path identifiers"
    );

    let app = app(state);

    for (path_id, canonical_id) in &expected {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/tei/{path_id}/info"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            value["model_id"], *canonical_id,
            "info for path '{path_id}' should report its own canonical model id"
        );
    }

    let ids: HashSet<_> = expected.iter().map(|(_, id)| id.clone()).collect();
    assert!(
        ids.contains("model-a"),
        "model-a metadata should be reported"
    );
    let b_reported = expected
        .iter()
        .any(|(path, id)| id == "model-b" && path == "model-b");
    assert!(
        b_reported,
        "model-b's own path must report model-b's canonical id, proving non-default metadata is discoverable"
    );
}

#[tokio::test]
async fn alias_path_info_reports_canonical_id() {
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
        model_alias: vec![("model-a".to_string(), "alpha".to_string())],
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

    let app = app(state);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/tei/alpha/info")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        value["model_id"], "model-a",
        "info via alias path 'alpha' should report the canonical id 'model-a'"
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/tei/model-a/info")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "'model-a' should 404 because the alias replaced its derived path segment"
    );

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        value["error"], "not_found",
        "404 body should use the documented error code"
    );
}
