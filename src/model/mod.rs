//! Model inference modules.

pub mod embedding;

use crate::errors::AppError;
use embedding::EmbeddingModel;
use std::{collections::HashMap, path::Path, sync::Arc};

/// Metadata and a loaded model2vec instance for a single model.
#[derive(Clone)]
pub struct LoadedModel {
    /// Canonical identifier used for routing and responses.
    pub model_id: String,
    /// Maximum input length in tokens accepted by this model.
    pub max_input_length: usize,
    /// Embedding vector dimension produced by this model.
    pub embedding_dimension: usize,
    /// Pooling method used by the model.
    pub pooling: &'static str,
    /// The underlying model instance.
    pub model: Arc<EmbeddingModel>,
}

impl LoadedModel {
    /// Create a new loaded model from a path or Hugging Face identifier.
    ///
    /// # Errors
    ///
    /// Returns an error if the model cannot be loaded from the given path.
    pub fn load(path: &str, max_input_length: usize) -> anyhow::Result<Self> {
        let model_id = derive_model_id(path);
        let model = Arc::new(EmbeddingModel::load(path)?);
        let embedding_dimension = model.dimension();

        Ok(Self {
            model_id,
            max_input_length,
            embedding_dimension,
            pooling: "mean",
            model,
        })
    }
}

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

/// Registry of all loaded models available for inference.
#[derive(Clone)]
pub struct ModelRegistry {
    /// Loaded models keyed by canonical identifier.
    models: HashMap<String, LoadedModel>,
    /// Identifier to use when a request does not specify a model.
    default_model_id: String,
}

impl ModelRegistry {
    /// Build a registry from a list of model paths and a default model identifier.
    ///
    /// Models are loaded concurrently. Models that fail to load are recorded and
    /// skipped; the registry succeeds as long as at least one model loads.
    ///
    /// # Errors
    ///
    /// Returns an error if no models are configured or if none of the configured
    /// models can be loaded.
    pub fn load(
        model_paths: &[String],
        default_model_id: Option<String>,
        max_input_length: usize,
    ) -> anyhow::Result<Self> {
        if model_paths.is_empty() {
            return Err(anyhow::anyhow!("at least one model must be configured"));
        }

        let mut loaded = Vec::with_capacity(model_paths.len());
        let mut errors = Vec::new();

        std::thread::scope(|s| {
            let handles: Vec<_> = model_paths
                .iter()
                .map(|path| {
                    let path = path.clone();
                    s.spawn(move || (path.clone(), LoadedModel::load(&path, max_input_length)))
                })
                .collect();

            for handle in handles {
                match handle.join() {
                    Ok((path, result)) => match result {
                        Ok(model) => loaded.push(model),
                        Err(err) => {
                            tracing::error!(model = %path, error = %err, "failed to load model");
                            errors.push((path, err));
                        }
                    },
                    Err(err) => {
                        tracing::error!(error = ?err, "model loading thread panicked");
                        errors.push((
                            "unknown".to_string(),
                            anyhow::anyhow!("loading thread panicked"),
                        ));
                    }
                }
            }
        });

        if loaded.is_empty() {
            return Err(anyhow::anyhow!(
                "none of the configured models could be loaded: {errors:?}"
            ));
        }

        let mut models = HashMap::with_capacity(loaded.len());
        for model in loaded {
            models.insert(model.model_id.clone(), model);
        }

        let default_model_id = default_model_id
            .or_else(|| model_paths.first().cloned())
            .map(|path| derive_model_id(&path))
            .ok_or_else(|| anyhow::anyhow!("no default model available"))?;

        if !models.contains_key(&default_model_id) {
            return Err(anyhow::anyhow!(
                "default model '{default_model_id}' is not loaded"
            ));
        }

        Ok(Self {
            models,
            default_model_id,
        })
    }

    /// Return the default model identifier.
    #[must_use]
    pub fn default_model_id(&self) -> &str {
        &self.default_model_id
    }

    /// Look up a model by identifier.
    ///
    /// Returns `None` if the identifier is not loaded.
    #[must_use]
    pub fn get(&self, model_id: &str) -> Option<&LoadedModel> {
        self.models.get(model_id)
    }

    /// Look up a model by identifier, falling back to the default when `None` is passed.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::ModelNotFound`] if the requested or default model is
    /// not loaded.
    pub fn resolve(&self, model_id: Option<&str>) -> Result<&LoadedModel, AppError> {
        let id = model_id.unwrap_or(&self.default_model_id);
        self.models
            .get(id)
            .ok_or_else(|| AppError::ModelNotFound(format!("model '{id}' is not loaded")))
    }

    /// Return an iterator over all loaded models.
    pub fn iter(&self) -> impl Iterator<Item = &LoadedModel> {
        self.models.values()
    }

    /// Return per-model status information for health checks.
    #[must_use]
    pub fn model_statuses(&self) -> Vec<ModelStatus> {
        self.models
            .values()
            .map(|m| ModelStatus {
                model_id: m.model_id.clone(),
                status: "ready",
                message: "model loaded".to_string(),
            })
            .collect()
    }
}

/// Per-model status exposed by the health endpoint.
#[derive(Debug, Clone)]
pub struct ModelStatus {
    /// Model identifier.
    pub model_id: String,
    /// Load status: `ready` or `failed`.
    pub status: &'static str,
    /// Human-readable status message.
    pub message: String,
}
