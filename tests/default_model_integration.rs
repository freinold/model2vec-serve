#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

// Integration test for the default multilingual model.
//
// This test deliberately loads the default model from Hugging Face Hub and is
// therefore slower than the fixture-based integration tests. It is marked with
// `#[ignore]` so that `cargo test` remains fast; run it explicitly with
// `cargo test --test default_model_integration -- --ignored` when validation of
// the default model is required.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use clap::Parser;
use http_body_util::BodyExt;
use metrics_exporter_prometheus::PrometheusHandle;
use model2vec_serve::{config::Config, routes::app, state::AppState, telemetry};
use serde_json::{Value, json};
use std::sync::{Arc, OnceLock};
use tower::ServiceExt;

static METRICS: OnceLock<Arc<PrometheusHandle>> = OnceLock::new();

fn metrics_handle() -> Arc<PrometheusHandle> {
    METRICS
        .get_or_init(|| Arc::new(telemetry::init_metrics()))
        .clone()
}

#[tokio::test]
#[ignore = "downloads the default 128M model from Hugging Face Hub"]
async fn default_model_loads_and_embeds_non_english_input() {
    let config = Config::parse_from(Vec::<&str>::new());
    assert_eq!(
        config.model, "minishlab/potion-multilingual-128M",
        "default model should be the multilingual model"
    );

    let state = AppState::new(config, metrics_handle()).expect("failed to load default model");
    let app = app(state);

    let info_response = app
        .clone()
        .oneshot(Request::builder().uri("/info").body(Body::empty()).unwrap())
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
    assert_eq!(
        info["model_id"].as_str().unwrap(),
        "minishlab/potion-multilingual-128M"
    );

    let embed_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/embeddings")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"input": "Hola mundo"}).to_string()))
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
    let data = embed["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert!(
        !data[0]["embedding"].as_array().unwrap().is_empty(),
        "non-English input should produce a non-empty embedding"
    );
}
