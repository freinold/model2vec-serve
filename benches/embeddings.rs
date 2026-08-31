#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hf_hub::HFClientSync;
use model2vec_serve::model::embedding::EmbeddingModel;
use model2vec_serve::{config::Config, state::AppState, telemetry};
use std::sync::Arc;

const BENCH_MODEL: &str = "minishlab/potion-base-2M";

fn model_dir() -> String {
    let client = HFClientSync::new().expect("hf-hub API init failed");
    let (namespace, repo) = BENCH_MODEL
        .split_once('/')
        .expect("BENCH_MODEL must be in namespace/repo format");

    client
        .model(namespace, repo)
        .snapshot_download()
        .allow_patterns(vec![
            "config.json".to_string(),
            "tokenizer.json".to_string(),
            "model.safetensors".to_string(),
        ])
        .send()
        .expect("failed to download model snapshot")
        .to_string_lossy()
        .to_string()
}

fn bench_embeddings(c: &mut Criterion) {
    let model = EmbeddingModel::load(&model_dir()).expect("failed to load model");
    let inputs: Vec<String> = (0..64)
        .map(|i| format!("this is sentence number {i}"))
        .collect();

    let model_path = model_dir();
    let config = Config {
        host: "127.0.0.1".to_string(),
        port: 0,
        models: vec![model_path.clone()],
        default_model: Some(model_path),
        model_owner: "minishlab".to_string(),
        model_alias: Vec::new(),
        api_key: None,
        max_batch_size: 64,
        max_input_length: 512,
        log_level: "warn".to_string(),
        request_timeout_seconds: 30,
    };
    let state = AppState::new(config, Arc::new(telemetry::init_metrics()))
        .expect("failed to load state for per-model benchmark");
    let path_id = state
        .registry
        .path_identifier_for(state.registry.default_model_id())
        .expect("default model must have a path identifier")
        .to_string();

    let mut group = c.benchmark_group("embeddings");
    group.throughput(Throughput::Elements(inputs.len() as u64));
    group.bench_function("batch_of_64", |b| {
        b.iter(|| {
            let result = model.encode(&inputs, 512, inputs.len());
            assert_eq!(result.len(), inputs.len());
        });
    });
    group.bench_function("per_model_batch_of_64", |b| {
        b.iter(|| {
            let loaded = state
                .registry
                .get_by_path(&path_id)
                .expect("per-model path identifier must resolve to a loaded model");
            let result = loaded
                .model
                .encode(&inputs, loaded.max_input_length, inputs.len());
            assert_eq!(result.len(), inputs.len());
        });
    });
    group.finish();
}

criterion_group!(benches, bench_embeddings);
criterion_main!(benches);
