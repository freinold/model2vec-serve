//! Shared application state passed to axum handlers.

use crate::{config::Config, model::embedding::EmbeddingModel};
use metrics_exporter_prometheus::PrometheusHandle;
use std::{path::Path, sync::Arc};

fn derive_model_id(model: &str) -> String {
    let path = Path::new(model);
    if path.exists() {
        path.file_name().map_or_else(
            || model.to_string(),
            |name| name.to_string_lossy().to_string(),
        )
    } else {
        model.to_string()
    }
}

/// State shared across all HTTP handlers.
pub struct AppState {
    /// Runtime configuration.
    pub config: Config,
    /// Loaded model2vec model.
    pub model: EmbeddingModel,
    /// Identifier of the loaded model.
    pub model_id: String,
    /// Embedding dimension inferred from the model.
    pub embedding_dimension: usize,
    /// Prometheus metrics recorder handle.
    pub metrics_handle: Arc<PrometheusHandle>,
}

impl AppState {
    /// Create a new shared state instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the model cannot be loaded.
    pub fn new(config: Config, metrics_handle: Arc<PrometheusHandle>) -> anyhow::Result<Arc<Self>> {
        let model_id = derive_model_id(&config.model);
        let model = EmbeddingModel::load(&config.model)?;
        let embedding_dimension = model.dimension();

        Ok(Arc::new(Self {
            config,
            model,
            model_id,
            embedding_dimension,
            metrics_handle,
        }))
    }
}
