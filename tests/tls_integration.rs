#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Integration tests for TLS/HTTPS serving (spec 007).
//
// These tests drive a real rustls listener (not the tower in-process harness)
// so the full TLS handshake and transport behavior are exercised. The same
// AppState is served over both HTTPS and plain HTTP to prove response
// identity across transports (SC-002).

use common::{metrics_handle, test_config, tls_pair};
use model2vec_serve::{routes::app, state::AppState};
use reqwest::Client;
use serde_json::{Value, json};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::task::JoinHandle;

/// Build a client that verifies the server against the generated test
/// certificate as its pinned trust root.
///
/// This keeps the full rustls verification path (chain + IP SAN) exercised in
/// tests instead of disabling certificate validation.
fn pinned_client(cert_der: &[u8]) -> Client {
    let mut roots = rustls::RootCertStore::empty();
    roots
        .add(rustls::pki_types::CertificateDer::from(cert_der.to_vec()))
        .expect("test certificate must be a valid trust root");
    let provider = rustls::crypto::aws_lc_rs::default_provider();
    let config = rustls::ClientConfig::builder_with_provider(provider.into())
        .with_safe_default_protocol_versions()
        .expect("default protocol versions")
        .with_root_certificates(roots)
        .with_no_client_auth();
    Client::builder()
        .use_preconfigured_tls(config)
        .build()
        .expect("failed to build pinned https test client")
}

/// Build a client that never negotiates TLS.
fn plain_client() -> Client {
    Client::builder()
        .build()
        .expect("failed to build http test client")
}

/// Spawn the shared app behind a rustls listener on an ephemeral port.
///
/// Returns the bound address, the server task handle, and the TEI path
/// identifier of the default model (for the per-model routes).
async fn spawn_tls_server(
    files: &common::TlsFiles,
    api_key: Option<&str>,
) -> (SocketAddr, JoinHandle<()>, String) {
    let tls =
        model2vec_serve::tls::load(&files.cert_path, &files.key_path).expect("valid TLS fixture");
    let config = test_config(api_key.map(str::to_string));
    let state = AppState::new(config, metrics_handle()).expect("failed to load model");
    let path_id = state
        .registry
        .path_identifier_for(state.registry.default_model_id())
        .expect("default model should have a path identifier")
        .to_string();

    let server = axum_server::bind_rustls(
        "127.0.0.1:0".parse().expect("valid loopback address"),
        tls.config,
    );
    let handle = axum_server::Handle::new();
    let bound_handle = handle.clone();
    let join = tokio::spawn(async move {
        server
            .handle(handle)
            .serve(app(state).into_make_service())
            .await
            .expect("tls server terminated unexpectedly");
    });
    let addr = bound_handle
        .listening()
        .await
        .expect("tls server failed to bind");

    wait_until_ready(
        &pinned_client(&files.cert_der),
        &format!("https://{addr}/health"),
    )
    .await;
    (addr, join, path_id)
}

/// Spawn the same app behind a plain HTTP listener on an ephemeral port.
async fn spawn_plain_server(api_key: Option<&str>) -> (SocketAddr, JoinHandle<()>, String) {
    let config = test_config(api_key.map(str::to_string));
    let state = AppState::new(config, metrics_handle()).expect("failed to load model");
    let path_id = state
        .registry
        .path_identifier_for(state.registry.default_model_id())
        .expect("default model should have a path identifier")
        .to_string();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind plain listener");
    let addr = listener
        .local_addr()
        .expect("bound listener has an address");
    let join = tokio::spawn(async move {
        axum::serve(listener, app(state))
            .await
            .expect("http server terminated unexpectedly");
    });

    wait_until_ready(&plain_client(), &format!("http://{addr}/health")).await;
    (addr, join, path_id)
}

/// Poll the given URL until it answers 200 or the timeout elapses.
async fn wait_until_ready(client: &Client, url: &str) {
    let deadline = Duration::from_secs(30);
    let start = std::time::Instant::now();
    loop {
        if client
            .get(url)
            .send()
            .await
            .is_ok_and(|r| r.status().is_success())
        {
            return;
        }
        assert!(
            start.elapsed() < deadline,
            "server at {url} did not become ready in time"
        );
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

/// The embedding payload used for transport-identity comparisons.
fn embed_payload() -> String {
    json!({"inputs": "hello"}).to_string()
}

#[tokio::test]
async fn all_endpoint_classes_served_over_https() {
    let files = tls_pair();
    let (addr, server, _) = spawn_tls_server(&files, None).await;
    let base = format!("https://{addr}");
    let client = pinned_client(&files.cert_der);

    let health = client.get(format!("{base}/health")).send().await.unwrap();
    assert_eq!(health.status(), 200);
    assert!(health.json::<Value>().await.unwrap().is_object());

    let models = client
        .get(format!("{base}/v1/models"))
        .send()
        .await
        .unwrap();
    assert_eq!(models.status(), 200);
    let models_json = models.json::<Value>().await.unwrap();
    assert_eq!(models_json["object"], json!("list"));

    let embed = client
        .post(format!("{base}/embed"))
        .header("content-type", "application/json")
        .body(embed_payload())
        .send()
        .await
        .unwrap();
    assert_eq!(embed.status(), 200);
    let embed_json = embed.json::<Value>().await.unwrap();
    assert!(embed_json.is_array());
    assert!(embed_json[0].is_array());

    server.abort();
}

#[tokio::test]
async fn https_response_equals_plain_http_response() {
    let files = tls_pair();
    let (tls_addr, tls_server, path_id) = spawn_tls_server(&files, None).await;
    let (plain_addr, plain_server, _) = spawn_plain_server(None).await;
    let https = pinned_client(&files.cert_der);
    let plain = plain_client();

    // `compare_body` is false for endpoints whose bodies legitimately differ
    // between the two server instances (/metrics counters) or that return
    // non-JSON content (/docs HTML); those are checked for status parity.
    for (path, body, compare_body) in [
        ("/health", None, true),
        ("/ready", None, true),
        ("/info", None, true),
        ("/v1/models", None, true),
        ("/docs", None, false),
        ("/metrics", None, false),
        ("/embed", Some(embed_payload()), true),
        (
            "/v1/embeddings",
            Some(json!({"input": "hello"}).to_string()),
            true,
        ),
        // Error parity: the unknown-model error body must be identical too.
        (
            "/v1/embeddings",
            Some(json!({"input": "hello", "model": "unknown-model"}).to_string()),
            true,
        ),
        (
            format!("/tei/{path_id}/embed").as_str(),
            Some(embed_payload()),
            true,
        ),
        (format!("/tei/{path_id}/info").as_str(), None, true),
    ] {
        let (tls_result, plain_result) = if let Some(payload) = body {
            let tls_future = https
                .post(format!("https://{tls_addr}{path}"))
                .header("content-type", "application/json")
                .body(payload.clone())
                .send();
            let plain_future = plain
                .post(format!("http://{plain_addr}{path}"))
                .header("content-type", "application/json")
                .body(payload.clone())
                .send();
            (tls_future.await, plain_future.await)
        } else {
            let tls_future = https.get(format!("https://{tls_addr}{path}")).send();
            let plain_future = plain.get(format!("http://{plain_addr}{path}")).send();
            (tls_future.await, plain_future.await)
        };

        let tls_response = tls_result.unwrap();
        let plain_response = plain_result.unwrap();
        assert_eq!(
            tls_response.status(),
            plain_response.status(),
            "status for {path}"
        );
        if !compare_body {
            continue;
        }
        assert_eq!(
            tls_response.json::<Value>().await.unwrap(),
            plain_response.json::<Value>().await.unwrap(),
            "body for {path}"
        );
    }

    tls_server.abort();
    plain_server.abort();
}

#[tokio::test]
async fn https_auth_behaviour_matches_plain_http() {
    let files = tls_pair();
    let (tls_addr, tls_server, _) = spawn_tls_server(&files, Some("secret")).await;
    let (plain_addr, plain_server, _) = spawn_plain_server(Some("secret")).await;
    let https = pinned_client(&files.cert_der);
    let plain = plain_client();

    // (authorization header, expected status) — auth must behave identically
    // over both transports, including the public operational endpoints.
    for (header, expected) in [
        (None, 401),
        (Some("Bearer wrong"), 401),
        (Some("Bearer secret"), 200),
    ] {
        let send = |client: &Client, url: String, header: Option<&str>| {
            let mut request = client.post(url).header("content-type", "application/json");
            if let Some(value) = header {
                request = request.header("authorization", value);
            }
            request.body(embed_payload()).send()
        };

        let tls_status = send(&https, format!("https://{tls_addr}/embed"), header)
            .await
            .unwrap()
            .status();
        let plain_status = send(&plain, format!("http://{plain_addr}/embed"), header)
            .await
            .unwrap()
            .status();
        assert_eq!(tls_status, plain_status, "status for auth {header:?}");
        assert_eq!(tls_status.as_u16(), expected, "auth {header:?}");
    }

    // Operational endpoints stay public with auth enabled, over both
    // transports.
    for (label, url) in [
        ("health", format!("https://{tls_addr}/health")),
        ("ready", format!("https://{tls_addr}/ready")),
        ("metrics", format!("https://{tls_addr}/metrics")),
    ] {
        let status = pinned_client(&files.cert_der)
            .get(&url)
            .send()
            .await
            .unwrap()
            .status();
        assert_eq!(status.as_u16(), 200, "{label} stays public with auth on");
    }

    tls_server.abort();
    plain_server.abort();
}

#[tokio::test]
async fn plain_http_to_tls_port_fails_but_service_stays_healthy() {
    let files = tls_pair();
    let (addr, server, _) = spawn_tls_server(&files, None).await;
    let plain = plain_client();

    let rejected = plain.get(format!("http://{addr}/health")).send().await;
    assert!(rejected.is_err(), "plain HTTP to the TLS port must fail");

    let still_healthy = pinned_client(&files.cert_der)
        .get(format!("https://{addr}/health"))
        .send()
        .await
        .unwrap();
    assert_eq!(still_healthy.status(), 200);

    server.abort();
}

#[tokio::test]
async fn tls_setup_reports_enabled() {
    let files = tls_pair();
    let setup = model2vec_serve::tls::load(&files.cert_path, &files.key_path);
    let setup = match setup {
        Ok(setup) => setup,
        Err(err) => panic!("valid pair must load: {err}"),
    };
    assert!(setup.metadata.expired_since.is_none());
}
