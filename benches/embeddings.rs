#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hf_hub::api::sync::Api;
use model2vec_serve::model::embedding::EmbeddingModel;

fn model_dir() -> String {
    let api = Api::new().expect("hf-hub API init failed");
    let repo = api.model("minishlab/potion-base-2M".to_owned());
    let config = repo
        .get("config.json")
        .expect("failed to fetch config.json");
    let _ = repo.get("tokenizer.json");
    let _ = repo.get("model.safetensors");
    config
        .parent()
        .expect("config.json has no parent")
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
