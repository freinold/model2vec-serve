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

/// Return the default per-model path identifier for a canonical model
/// identifier: the substring after the final `/`.
///
/// Operators can override this per model via `--model-alias`; see
/// [`ModelRegistry::get_by_path`].
#[must_use]
pub fn path_identifier(model_id: &str) -> String {
    model_id
        .rsplit_once('/')
        .map_or(model_id.to_string(), |(_, last)| last.to_string())
}

/// Registry of all loaded models available for inference.
pub struct ModelRegistry {
    /// Loaded models keyed by canonical identifier.
    models: HashMap<String, LoadedModel>,
    /// Path identifiers mapped to canonical model identifiers.
    path_index: HashMap<String, String>,
    /// Identifier to use when a request does not specify a model.
    default_model_id: String,
    /// Models that failed to load, with their configured path and the error.
    failed_models: Vec<(String, anyhow::Error)>,
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
        model_alias: &[(String, String)],
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

        let default_model_id = match default_model_id {
            Some(path) => {
                let id = derive_model_id(&path);
                if !loaded.iter().any(|m| m.model_id == id) {
                    return Err(anyhow::anyhow!("default model '{id}' is not loaded"));
                }
                id
            }
            None => loaded
                .first()
                .map(|m| m.model_id.clone())
                .ok_or_else(|| anyhow::anyhow!("no default model available"))?,
        };

        let mut models = HashMap::with_capacity(loaded.len());
        for model in loaded {
            let model_id = model.model_id.clone();
            if models.insert(model_id.clone(), model).is_some() {
                return Err(anyhow::anyhow!("duplicate model identifier '{model_id}'"));
            }
        }

        let path_index = build_path_index(model_paths, &models, model_alias)?;

        Ok(Self {
            models,
            path_index,
            default_model_id,
            failed_models: errors,
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

    /// Look up a model by its per-model path identifier.
    ///
    /// The path identifier is the model's configured alias, or the default
    /// derived by [`path_identifier`] when no alias is configured. Uniqueness
    /// of path identifiers is enforced at startup.
    #[must_use]
    pub fn get_by_path(&self, path_id: &str) -> Option<&LoadedModel> {
        self.path_index
            .get(path_id)
            .and_then(|model_id| self.models.get(model_id))
    }

    /// Return the per-model path identifier for a canonical model identifier.
    #[must_use]
    pub fn path_identifier_for(&self, model_id: &str) -> Option<&str> {
        self.path_index
            .iter()
            .find(|(_, model)| model.as_str() == model_id)
            .map(|(path, _)| path.as_str())
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
    ///
    /// Includes both successfully loaded models and models that failed to load.
    #[must_use]
    pub fn model_statuses(&self) -> Vec<ModelStatus> {
        let mut statuses: Vec<ModelStatus> = self
            .models
            .values()
            .map(|m| ModelStatus {
                model_id: m.model_id.clone(),
                status: "ready",
                message: "model loaded".to_string(),
            })
            .collect();

        statuses.extend(self.failed_models.iter().map(|(path, _err)| ModelStatus {
            model_id: path.clone(),
            status: "failed",
            message: "model failed to load".to_string(),
        }));

        statuses
    }

    /// Return the number of models that loaded successfully.
    #[must_use]
    pub fn loaded_count(&self) -> usize {
        self.models.len()
    }
}

/// Derive the per-model path identifier for every loaded model and validate
/// uniqueness.
///
/// A model's path identifier is its configured alias, or the default from
/// [`path_identifier`] when no alias matches. Alias keys must match a
/// configured model path (exactly as given to `--model`) or a canonical model
/// identifier; path identifiers must be unique across all loaded models.
///
/// # Errors
///
/// Returns an error when a `KEY=ALIAS` key is duplicated, when an alias key
/// matches no configured model, or when two loaded models resolve to the same
/// path identifier.
fn build_path_index(
    model_paths: &[String],
    models: &HashMap<String, LoadedModel>,
    model_alias: &[(String, String)],
) -> anyhow::Result<HashMap<String, String>> {
    let mut aliases: HashMap<&str, &str> = HashMap::with_capacity(model_alias.len());
    for (key, alias) in model_alias {
        if aliases.insert(key.as_str(), alias.as_str()).is_some() {
            return Err(anyhow::anyhow!("duplicate model alias key '{key}'"));
        }
    }

    for key in aliases.keys() {
        let matched = model_paths.iter().any(|path| path == key)
            || models.values().any(|model| model.model_id == *key);
        if !matched {
            return Err(anyhow::anyhow!(
                "model alias key '{key}' does not match any configured model; use a model identifier or local path from --model"
            ));
        }
    }

    let derived_by_path: HashMap<&str, String> = model_paths
        .iter()
        .map(|path| (path.as_str(), derive_model_id(path)))
        .collect();

    let mut path_index: HashMap<String, String> = HashMap::with_capacity(models.len());
    for model in models.values() {
        let alias = aliases
            .iter()
            .find(|(key, _)| {
                derived_by_path
                    .get(*key)
                    .is_some_and(|id| id == &model.model_id)
                    || **key == model.model_id
            })
            .map(|(_, alias)| (*alias).to_string());
        let path_id = alias.unwrap_or_else(|| path_identifier(&model.model_id));

        if let Some(existing) = path_index.insert(path_id.clone(), model.model_id.clone()) {
            return Err(anyhow::anyhow!(
                "models '{existing}' and '{}' resolve to the same per-model path identifier '{path_id}'; configure distinct aliases via --model-alias",
                model.model_id
            ));
        }
    }

    Ok(path_index)
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
