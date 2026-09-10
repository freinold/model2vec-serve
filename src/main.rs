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
            let addr = resolve_addr(&bind_address).await?;

            let handle = axum_server::Handle::new();
            let shutdown_handle = handle.clone();
            tokio::spawn(async move {
                shutdown_signal().await;
                shutdown_handle.graceful_shutdown(None);
            });

            tracing::info!("listening on {} (tls)", addr);
            axum_server::bind_rustls(addr, setup.config)
                .handle(handle)
                .serve(app(state).into_make_service())
                .await?;
        }
    }

    Ok(())
}

/// Resolve the configured `host:port` into a socket address.
///
/// Numeric hosts parse directly; hostnames are resolved to their first
/// address so both transports accept the same host values.
async fn resolve_addr(bind_address: &str) -> anyhow::Result<SocketAddr> {
    if let Ok(addr) = bind_address.parse() {
        return Ok(addr);
    }
    tokio::net::lookup_host(bind_address)
        .await?
        .next()
        .ok_or_else(|| anyhow::anyhow!("failed to resolve bind address {bind_address}"))
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
