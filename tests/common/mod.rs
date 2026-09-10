//! Shared test helpers.

#![allow(dead_code)]
#![allow(clippy::unused_async)]

use hf_hub::HFClientSync;
use metrics_exporter_prometheus::PrometheusHandle;
use model2vec_serve::{config::Config, routes::app, state::AppState, telemetry};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tempfile::TempDir;
use time::OffsetDateTime;

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
        model_alias: Vec::new(),
        api_key,
        max_batch_size: 32,
        max_input_length: 512,
        log_level: "warn".to_string(),
        request_timeout_seconds: 30,
        tls_cert: None,
        tls_key: None,
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
        model_alias: Vec::new(),
        api_key,
        max_batch_size: 32,
        max_input_length: 512,
        log_level: "warn".to_string(),
        request_timeout_seconds: 30,
        tls_cert: None,
        tls_key: None,
    }
}

/// Build a test configuration with an explicit model list, default model,
/// and per-model path aliases.
pub fn test_config_with_aliases(
    models: Vec<String>,
    default_model: Option<String>,
    model_alias: Vec<(String, String)>,
    api_key: Option<String>,
) -> Config {
    Config {
        host: "127.0.0.1".to_string(),
        port: 0,
        models,
        default_model,
        model_owner: "minishlab".to_string(),
        model_alias,
        api_key,
        max_batch_size: 32,
        max_input_length: 512,
        log_level: "warn".to_string(),
        request_timeout_seconds: 30,
        tls_cert: None,
        tls_key: None,
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

/// Create an axum app for testing with explicit models, default model, and
/// path aliases.
pub async fn test_app_with_aliases(
    models: Vec<String>,
    default_model: Option<String>,
    model_alias: Vec<(String, String)>,
    api_key: Option<String>,
) -> axum::Router {
    let config = test_config_with_aliases(models, default_model, model_alias, api_key);
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

/// TLS certificate fixture: paths to a certificate/key pair plus the temporary
/// directories backing them.
///
/// The `dirs` field keeps the backing temporary directories alive until the
/// fixture is dropped; tests only use the public path fields.
pub struct TlsFiles {
    dirs: Vec<TempDir>,
    /// Path to the PEM certificate file (leaf, optionally with chain).
    pub cert_path: PathBuf,
    /// Path to the PEM private key file.
    pub key_path: PathBuf,
}

impl TlsFiles {
    /// Path to a file that does not exist inside the fixture directory.
    pub fn missing_path(&self) -> PathBuf {
        self.dirs
            .first()
            .expect("fixture always has one directory")
            .path()
            .join("does-not-exist.pem")
    }
}

/// Generate a self-signed certificate and private key as PEM strings.
///
/// `not_after` overrides the certificate expiry (used to build expired
/// fixtures); when `None` the rcgen default (about one week) applies.
fn generate_pems(not_after: Option<OffsetDateTime>) -> (String, String) {
    let key_pair = rcgen::KeyPair::generate().expect("failed to generate test key pair");
    let mut params = rcgen::CertificateParams::new(vec!["localhost".to_string()])
        .expect("valid subject alt name");
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "localhost");
    if let Some(expiry) = not_after {
        params.not_before = OffsetDateTime::from_unix_timestamp(0).expect("valid unix epoch");
        params.not_after = expiry;
    }
    let cert = params.self_signed(&key_pair).expect("failed to self-sign");
    (cert.pem(), key_pair.serialize_pem())
}

/// Write the given PEM strings to a fresh temporary directory.
fn write_pair(cert_pem: &str, key_pem: &str) -> TlsFiles {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let cert_path = dir.path().join("cert.pem");
    let key_path = dir.path().join("key.pem");
    std::fs::write(&cert_path, cert_pem).expect("failed to write cert fixture");
    std::fs::write(&key_path, key_pem).expect("failed to write key fixture");
    TlsFiles {
        dirs: vec![dir],
        cert_path,
        key_path,
    }
}

/// Generate a valid self-signed certificate/key pair.
pub fn tls_pair() -> TlsFiles {
    let (cert, key) = generate_pems(None);
    write_pair(&cert, &key)
}

/// Generate a certificate/key pair whose key does not match the certificate.
pub fn mismatched_tls_pair() -> TlsFiles {
    let (cert, _) = generate_pems(None);
    let (_, key) = generate_pems(None);
    write_pair(&cert, &key)
}

/// Generate a certificate/key pair with an already expired certificate.
pub fn expired_tls_pair() -> TlsFiles {
    let expired = OffsetDateTime::from_unix_timestamp(1_000_000_000).expect("valid unix timestamp");
    let (cert, key) = generate_pems(Some(expired));
    write_pair(&cert, &key)
}

/// Generate a valid key with a certificate file containing non-PEM garbage.
pub fn garbage_cert_tls_pair() -> TlsFiles {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let cert_path = dir.path().join("cert.pem");
    let key_path = dir.path().join("key.pem");
    std::fs::write(&cert_path, b"this is not a pem file\n").expect("failed to write garbage");
    let (_, key) = generate_pems(None);
    std::fs::write(&key_path, key).expect("failed to write key fixture");
    TlsFiles {
        dirs: vec![dir],
        cert_path,
        key_path,
    }
}

/// Generate a garbage private-key file with a valid certificate.
pub fn garbage_key_tls_pair() -> TlsFiles {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let cert_path = dir.path().join("cert.pem");
    let key_path = dir.path().join("key.pem");
    let (cert, _) = generate_pems(None);
    std::fs::write(&cert_path, cert).expect("failed to write cert fixture");
    std::fs::write(&key_path, b"\x00\x01\x02 not pem either\n").expect("failed to write garbage");
    TlsFiles {
        dirs: vec![dir],
        cert_path,
        key_path,
    }
}

/// Generate a certificate with a private key exported as an encrypted PKCS#8
/// PEM.
///
/// Prefers `openssl` (available in development and CI environments) to produce
/// a genuine encrypted key; when the binary is unavailable, writes a
/// PEM-label-correct stub instead. Both satisfy the E4 detection contract,
/// which keys off the PEM label before any key parsing happens.
pub fn encrypted_key_tls_pair() -> TlsFiles {
    let pair = tls_pair();
    let encrypted = encrypt_key_with_openssl(&pair.key_path).unwrap_or_else(encrypted_key_stub);
    std::fs::write(&pair.key_path, encrypted).expect("failed to write encrypted key fixture");
    pair
}

/// Convert a plaintext PEM key into an encrypted PKCS#8 PEM via `openssl`.
///
/// Returns `None` when the binary is missing or the conversion fails.
fn encrypt_key_with_openssl(plain_key: &std::path::Path) -> Option<String> {
    let output = std::process::Command::new("openssl")
        .args([
            "pkcs8",
            "-topk8",
            "-in",
            &plain_key.to_string_lossy(),
            "-passout",
            "pass:test-password",
            "-v2",
            "aes256",
        ])
        .output()
        .ok()?;
    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}

/// PEM-label-correct encrypted-key stub for environments without `openssl`.
///
/// The label drives the E4 detection contract; the body never reaches a key
/// parser because detection fails first.
fn encrypted_key_stub() -> String {
    [
        "-----BEGIN ENCRYPTED PRIVATE KEY-----",
        "TENYcnlwdmVydGVzdHN0dWJib2Nrb2xlaHNob2Nrb2xlaG9sYWhvbGFob2xh",
        "-----END ENCRYPTED PRIVATE KEY-----",
        "",
    ]
    .join("\n")
}
