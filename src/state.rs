//! Shared application state passed to axum handlers.

use crate::{config::Config, model::ModelRegistry};
use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::Arc;

/// State shared across all HTTP handlers.
pub struct AppState {
    /// Runtime configuration.
    pub config: Config,
    /// Registry of all loaded models.
    pub registry: ModelRegistry,
    /// Prometheus metrics recorder handle.
    pub metrics_handle: Arc<PrometheusHandle>,
}

impl AppState {
    /// Create a new shared state instance.
    ///
    /// # Errors
    ///
    /// Returns an error if no models can be loaded.
    pub fn new(config: Config, metrics_handle: Arc<PrometheusHandle>) -> anyhow::Result<Arc<Self>> {
        let default_model = config.default_model.clone();
        let registry = ModelRegistry::load(&config.models, default_model, config.max_input_length)?;

        Ok(Arc::new(Self {
            config,
            registry,
            metrics_handle,
        }))
    }
}
