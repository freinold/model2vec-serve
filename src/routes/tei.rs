//! Text Embedding Inference compatible endpoints.

use crate::{
    errors::AppError,
    routes::dto::{ErrorResponse, ModelInfo, TeiEmbedRequest},
    state::AppState,
    telemetry::RequestModelId,
};
use axum::{Extension, Json, extract::Query, extract::State};
use serde::Deserialize;
use std::sync::Arc;

/// Optional model selector for TEI-compatible endpoints.
#[derive(Debug, Deserialize, Default)]
pub struct TeiModelQuery {
    /// Model identifier to use. If omitted, the configured default model is used.
    model: Option<String>,
}

/// TEI-compatible embed endpoint.
///
/// # Errors
///
/// Returns `AppError::BadRequest` for invalid input, `AppError::Unauthorized`
/// when authentication is enabled and fails, or `AppError::Internal` if model
/// inference fails.
#[utoipa::path(
    post,
    path = "/embed",
    tag = "tei",
    request_body = TeiEmbedRequest,
    responses(
        (status = 200, description = "Embeddings generated", body = Vec<Vec<f32>>),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Model inference error", body = ErrorResponse)
    )
)]
pub async fn tei_embed(
    State(state): State<Arc<AppState>>,
    Extension(model_id_ext): Extension<RequestModelId>,
    Query(query): Query<TeiModelQuery>,
    Json(request): Json<TeiEmbedRequest>,
) -> Result<Json<Vec<Vec<f32>>>, AppError> {
    if request.inputs.is_empty() {
        return Err(AppError::BadRequest("inputs cannot be empty".to_string()));
    }

    if request.inputs.len() > state.config.max_batch_size {
        return Err(AppError::BadRequest(format!(
            "batch size exceeds maximum of {}",
            state.config.max_batch_size
        )));
    }

    let loaded = state.registry.resolve(query.model.as_deref())?;
    let inputs = request.inputs.as_strings();
    let embeddings = loaded
        .model
        .encode(&inputs, loaded.max_input_length, inputs.len());

    model_id_ext.set(loaded.model_id.clone());

    Ok(Json(embeddings))
}

/// TEI-compatible model information endpoint.
///
/// # Errors
///
/// Returns `AppError::ModelNotFound` when the requested model is not loaded.
#[utoipa::path(
    get,
    path = "/info",
    tag = "tei",
    responses(
        (status = 200, description = "Model information", body = ModelInfo),
        (status = 400, description = "Invalid model", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
pub async fn tei_info(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TeiModelQuery>,
) -> Result<Json<ModelInfo>, AppError> {
    let loaded = state.registry.resolve(query.model.as_deref())?;
    Ok(Json(ModelInfo {
        model_id: loaded.model_id.clone(),
        max_input_length: loaded.max_input_length,
        embedding_dimension: loaded.embedding_dimension,
        pooling: loaded.pooling,
    }))
}
