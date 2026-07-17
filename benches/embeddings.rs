#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hf_hub::HFClientSync;
use model2vec_serve::model::embedding::EmbeddingModel;

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

    let mut group = c.benchmark_group("embeddings");
    group.throughput(Throughput::Elements(inputs.len() as u64));
    group.bench_function("batch_of_64", |b| {
        b.iter(|| {
            let result = model.encode(&inputs, 512, inputs.len());
            assert_eq!(result.len(), inputs.len());
        });
    });
    group.finish();
}

criterion_group!(benches, bench_embeddings);
criterion_main!(benches);
