//! Shared test helpers.

#![allow(dead_code)]
#![allow(clippy::unused_async)]

use hf_hub::HFClientSync;
use metrics_exporter_prometheus::PrometheusHandle;
use model2vec_serve::{config::Config, routes::app, state::AppState, telemetry};
use std::sync::{Arc, OnceLock};

const TEST_MODEL: &str = "minishlab/potion-base-2M";

static METRICS: OnceLock<Arc<PrometheusHandle>> = OnceLock::new();

pub fn metrics_handle() -> Arc<PrometheusHandle> {
    METRICS
        .get_or_init(|| Arc::new(telemetry::init_metrics()))
        .clone()
}

/// Download a small model fixture and return its local directory path.
pub fn model_dir() -> String {
    let client = HFClientSync::new().expect("hf-hub API init failed");
    let (namespace, repo) = TEST_MODEL
        .split_once('/')
        .expect("TEST_MODEL must be in namespace/repo format");

    let snapshot_dir = client
        .model(namespace, repo)
        .snapshot_download()
        .allow_patterns(vec![
            "config.json".to_string(),
            "tokenizer.json".to_string(),
            "model.safetensors".to_string(),
        ])
        .send()
        .expect("failed to download model snapshot");

    snapshot_dir.to_string_lossy().to_string()
}

/// Build a default test configuration pointing at the cached model.
pub fn test_config(api_key: Option<String>) -> Config {
    let model = model_dir();
    Config {
        host: "127.0.0.1".to_string(),
        port: 0,
        models: vec![model.clone()],
        default_model: Some(model),
        model_owner: "minishlab".to_string(),
        api_key,
        max_batch_size: 32,
        max_input_length: 512,
        log_level: "warn".to_string(),
        request_timeout_seconds: 30,
    }
}

/// Create an axum app for testing.
pub async fn test_app(api_key: Option<String>) -> axum::Router {
    let config = test_config(api_key);
    let state = AppState::new(config, metrics_handle()).expect("failed to load model");
    app(state)
}

/// Build a test configuration with an explicit model list and default model.
pub fn test_config_with_models(
    models: Vec<String>,
    default_model: Option<String>,
    api_key: Option<String>,
) -> Config {
    Config {
        host: "127.0.0.1".to_string(),
        port: 0,
        models,
        default_model,
        model_owner: "minishlab".to_string(),
        api_key,
        max_batch_size: 32,
        max_input_length: 512,
        log_level: "warn".to_string(),
        request_timeout_seconds: 30,
    }
}

/// Create an axum app for testing together with its state.
pub async fn test_app_with_state(api_key: Option<String>) -> (axum::Router, Arc<AppState>) {
    let config = test_config(api_key);
    let state = AppState::new(config, metrics_handle()).expect("failed to load model");
    (app(state.clone()), state)
}

/// Create an axum app for testing with an explicit model list and default model.
pub async fn test_app_with_models(
    models: Vec<String>,
    default_model: Option<String>,
    api_key: Option<String>,
) -> axum::Router {
    let config = test_config_with_models(models, default_model, api_key);
    let state = AppState::new(config, metrics_handle()).expect("failed to load model");
    app(state)
}

/// Return a second model directory that contains a copy of the fixture model.
///
/// The directory name is different from the fixture snapshot directory, so the
/// derived model id is distinct (`alt-model`). The temporary directory is
/// converted to a plain path and left for the test process to clean up.
pub fn alt_model_dir() -> String {
    let source = model_dir();
    let dir = tempfile::tempdir()
        .expect("failed to create temp dir")
        .keep();
    let alt_dir = dir.join("alt-model");
    std::fs::create_dir_all(&alt_dir).expect("failed to create alt model dir");

    for entry in std::fs::read_dir(&source).expect("failed to read model dir") {
        let entry = entry.expect("failed to read dir entry");
        let src = entry.path();
        if src.is_file() {
            let dst = alt_dir.join(src.file_name().expect("missing file name"));
            std::fs::copy(&src, &dst).expect("failed to copy model file");
        }
    }

    alt_dir.to_string_lossy().to_string()
}
