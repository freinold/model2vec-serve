//! Shared test helpers.

#![allow(dead_code)]
#![allow(clippy::unused_async)]

use hf_hub::HFClientSync;
use metrics_exporter_prometheus::PrometheusHandle;
use model2vec_serve::{config::Config, routes::app, state::AppState, telemetry};
use std::sync::{Arc, OnceLock};

const TEST_MODEL: &str = "minishlab/potion-base-2M";

static METRICS: OnceLock<Arc<PrometheusHandle>> = OnceLock::new();

fn metrics_handle() -> Arc<PrometheusHandle> {
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
    Config {
        host: "127.0.0.1".to_string(),
        port: 0,
        model: model_dir(),
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
