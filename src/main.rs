//! model2vec-serve binary entry point.

use clap::Parser;
use model2vec_serve::{
    config::{Config, TlsMode},
    routes::app,
    state::AppState,
    telemetry, tls,
};
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();
    telemetry::init_tracing(&config.log_level);

    let metrics_handle = Arc::new(telemetry::init_metrics());
    let state = AppState::new(config.clone(), metrics_handle)?;

    let bind_address = config.bind_address();

    match config.tls_mode() {
        Err(message) => anyhow::bail!(message),
        Ok(TlsMode::Disabled) => {
            let listener = tokio::net::TcpListener::bind(&bind_address).await?;
            tracing::info!("listening on {}", bind_address);

            axum::serve(listener, app(state))
                .with_graceful_shutdown(shutdown_signal())
                .await?;
        }
        Ok(TlsMode::Enabled { cert, key }) => {
            let setup = tls::load(&cert, &key)?;
            let listener = bind_tls_listener(&bind_address).await?;

            let handle = axum_server::Handle::new();
            let shutdown_handle = handle.clone();
            tokio::spawn(async move {
                shutdown_signal().await;
                shutdown_handle.graceful_shutdown(None);
            });

            // A bound listener always reports its local address.
            let bound_addr = listener
                .local_addr()
                .expect("bound listener has a local address");
            tracing::info!("listening on {} (tls)", bound_addr);
            axum_server::from_tcp_rustls(listener, setup.config)
                .handle(handle)
                .serve(app(state).into_make_service())
                .await?;
        }
    }

    Ok(())
}

/// Bind a TLS listener, trying every resolved address in turn.
///
/// Numeric addresses bind directly; hostnames resolve and each candidate is
/// attempted so a single unroutable resolved address does not abort startup,
/// mirroring how the plain-HTTP path binds.
async fn bind_tls_listener(bind_address: &str) -> anyhow::Result<std::net::TcpListener> {
    if let Ok(addr) = bind_address.parse::<SocketAddr>() {
        return std::net::TcpListener::bind(addr).map_err(|err| {
            anyhow::Error::new(err).context(format!("failed to bind TLS listener on {addr}"))
        });
    }

    let mut last_error = None;
    for addr in tokio::net::lookup_host(bind_address).await? {
        match std::net::TcpListener::bind(addr) {
            Ok(listener) => return Ok(listener),
            Err(err) => {
                tracing::debug!("skipping TLS bind candidate {addr}: {err}");
                last_error = Some(err);
            }
        }
    }

    match last_error {
        Some(err) => Err(anyhow::Error::new(err).context(format!(
            "failed to bind TLS listener on any resolved address for {bind_address}"
        ))),
        None => Err(anyhow::anyhow!(
            "host '{bind_address}' resolved to no addresses"
        )),
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };

    #[cfg(unix)]
    let terminate = async {
        let Ok(mut stream) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        else {
            return;
        };
        stream.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
}
