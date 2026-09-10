#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

use axum::{body::Body, http::Request};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hf_hub::HFClientSync;
use model2vec_serve::model::embedding::EmbeddingModel;
use model2vec_serve::{config::Config, routes::app, state::AppState, telemetry};
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tower::ServiceExt;

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
        tls_cert: None,
        tls_key: None,
    };
    let state = AppState::new(config, Arc::new(telemetry::init_metrics()))
        .expect("failed to load state for per-model benchmark");
    let path_id = state
        .registry
        .path_identifier_for(state.registry.default_model_id())
        .expect("default model must have a path identifier")
        .to_string();

    let router = app(Arc::clone(&state));
    let rt = Runtime::new().expect("failed to create tokio runtime for benchmark");
    let request_body = json!({ "inputs": inputs }).to_string();

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

    let per_model_uri = format!("/tei/{path_id}/embed");
    group.bench_function("http_root_batch_of_64", |b| {
        b.iter(|| {
            let request = Request::builder()
                .method("POST")
                .uri("/embed")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body.clone()))
                .expect("valid request");
            let response = rt
                .block_on(router.clone().oneshot(request))
                .expect("response");
            assert_eq!(response.status(), 200);
        });
    });
    group.bench_function("http_per_model_batch_of_64", |b| {
        b.iter(|| {
            let request = Request::builder()
                .method("POST")
                .uri(&per_model_uri)
                .header("Content-Type", "application/json")
                .body(Body::from(request_body.clone()))
                .expect("valid request");
            let response = rt
                .block_on(router.clone().oneshot(request))
                .expect("response");
            assert_eq!(response.status(), 200);
        });
    });
    group.finish();

    bench_transport(c, Arc::clone(&state), &rt, &request_body, inputs.len());
}

/// Keep-alive requests per benchmark iteration (steady-state transport load).
const TRANSPORT_REQUESTS_PER_ITER: usize = 16;

/// Per-request round-trip latencies collected for the p50/p99 report.
static PLAIN_LATENCIES: Mutex<Vec<Duration>> = Mutex::new(Vec::new());
static TLS_LATENCIES: Mutex<Vec<Duration>> = Mutex::new(Vec::new());

/// Servers and clients for the transport comparison.
struct TransportServers {
    /// Bound plain-HTTP listener address.
    plain_addr: std::net::SocketAddr,
    /// Bound TLS listener address.
    tls_addr: std::net::SocketAddr,
    /// Plain-HTTP client with connection pooling.
    plain_client: reqwest::Client,
    /// HTTPS client that accepts the self-signed bench certificate.
    tls_client: reqwest::Client,
    /// Backing directory for the generated bench certificate.
    _cert_dir: tempfile::TempDir,
}

/// Spawn the plain-HTTP and TLS listeners sharing the same app state.
async fn spawn_transport_servers(state: Arc<AppState>) -> TransportServers {
    let plain_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind plain bench listener");
    let plain_addr = plain_listener.local_addr().expect("bound listener address");
    let plain_state = Arc::clone(&state);
    tokio::spawn(async move {
        axum::serve(plain_listener, app(plain_state))
            .await
            .expect("plain bench server terminated unexpectedly");
    });

    let key_pair = rcgen::KeyPair::generate().expect("failed to generate bench key");
    let mut params = rcgen::CertificateParams::new(vec!["localhost".to_string()])
        .expect("valid bench subject alt name");
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "localhost");
    let cert = params.self_signed(&key_pair).expect("failed to self-sign");
    let dir = tempfile::tempdir().expect("failed to create bench temp dir");
    let cert_path = dir.path().join("cert.pem");
    let key_path = dir.path().join("key.pem");
    std::fs::write(&cert_path, cert.pem()).expect("failed to write bench cert");
    std::fs::write(&key_path, key_pair.serialize_pem()).expect("failed to write bench key");
    let tls = model2vec_serve::tls::load(&cert_path, &key_path)
        .expect("failed to build bench TLS config");

    let tls_state = state;
    let server =
        axum_server::bind_rustls("127.0.0.1:0".parse().expect("valid loopback"), tls.config);
    let handle = axum_server::Handle::new();
    let listen_handle = handle.clone();
    tokio::spawn(async move {
        server
            .handle(handle)
            .serve(app(tls_state).into_make_service())
            .await
            .expect("tls bench server terminated unexpectedly");
    });
    let tls_addr = listen_handle
        .listening()
        .await
        .expect("tls bench server failed to bind");

    let plain_client = reqwest::Client::new();
    let tls_client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("failed to build bench https client");

    for (client, url) in [
        (&plain_client, format!("http://{plain_addr}/health")),
        (&tls_client, format!("https://{tls_addr}/health")),
    ] {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if client
                .get(&url)
                .send()
                .await
                .is_ok_and(|response| response.status().is_success())
            {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "bench server at {url} did not become ready"
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    TransportServers {
        plain_addr,
        tls_addr,
        plain_client,
        tls_client,
        _cert_dir: dir,
    }
}

/// Compare steady-state plain-HTTP and TLS request handling (SC-005).
///
/// The same app state is served on two real loopback listeners (plain HTTP
/// via `axum::serve`, HTTPS via `axum_server` with a freshly generated
/// self-signed certificate). Requests run through pooled connections so
/// handshakes are excluded and only steady-state request handling is
/// measured.
fn bench_transport(
    c: &mut Criterion,
    state: Arc<AppState>,
    rt: &Runtime,
    request_body: &str,
    batch_size: usize,
) {
    let TransportServers {
        plain_addr,
        tls_addr,
        plain_client,
        tls_client,
        _cert_dir,
    } = rt.block_on(spawn_transport_servers(state));

    let mut group = c.benchmark_group("transport");
    group.throughput(Throughput::Elements(
        (batch_size * TRANSPORT_REQUESTS_PER_ITER) as u64,
    ));

    group.bench_function("plain_http_batch_of_64", |b| {
        b.iter(|| {
            rt.block_on(async {
                for _ in 0..TRANSPORT_REQUESTS_PER_ITER {
                    let start = Instant::now();
                    let response = plain_client
                        .post(format!("http://{plain_addr}/embed"))
                        .header("Content-Type", "application/json")
                        .body(request_body.to_string())
                        .send()
                        .await
                        .expect("plain bench request failed");
                    assert_eq!(response.status(), 200);
                    std::hint::black_box(response.bytes().await.expect("plain bench body"));
                    PLAIN_LATENCIES
                        .lock()
                        .expect("latency buffer")
                        .push(start.elapsed());
                }
            });
        });
    });

    group.bench_function("tls_batch_of_64", |b| {
        b.iter(|| {
            rt.block_on(async {
                for _ in 0..TRANSPORT_REQUESTS_PER_ITER {
                    let start = Instant::now();
                    let response = tls_client
                        .post(format!("https://{tls_addr}/embed"))
                        .header("Content-Type", "application/json")
                        .body(request_body.to_string())
                        .send()
                        .await
                        .expect("tls bench request failed");
                    assert_eq!(response.status(), 200);
                    std::hint::black_box(response.bytes().await.expect("tls bench body"));
                    TLS_LATENCIES
                        .lock()
                        .expect("latency buffer")
                        .push(start.elapsed());
                }
            });
        });
    });

    group.finish();

    report_percentiles(
        "plain_http",
        &PLAIN_LATENCIES.lock().expect("latency buffer"),
    );
    report_percentiles("tls", &TLS_LATENCIES.lock().expect("latency buffer"));
}

/// Index into a sorted sample for the given percentile (0..=100).
///
/// `p` comes from a fixed set of call sites (50.0, 99.0), so the rounded
/// result is always in range; the casts cannot truncate or lose sign.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn percentile_index(len: usize, p: f64) -> usize {
    (p / 100.0 * (len as f64 - 1.0)).round() as usize
}

/// Print the collected per-request latency percentiles for the SC-005 delta.
fn report_percentiles(label: &str, latencies: &[Duration]) {
    let mut millis: Vec<f64> = latencies.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
    if millis.is_empty() {
        eprintln!("[transport] {label}: no samples collected");
        return;
    }
    millis.sort_by(f64::total_cmp);
    let percentile = |p: f64| -> f64 { millis[percentile_index(millis.len(), p)] };
    eprintln!(
        "[transport] {label}: n={} p50={:.2}ms p99={:.2}ms",
        millis.len(),
        percentile(50.0),
        percentile(99.0)
    );
}

criterion_group!(benches, bench_embeddings);
criterion_main!(benches);
