//! Text Embedding Inference compatible endpoints.

use crate::{
    errors::AppError,
    routes::dto::{ErrorResponse, ModelInfo, TeiEmbedRequest},
    state::AppState,
};
use axum::{Json, extract::State};
use std::sync::Arc;

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

    let inputs = request.inputs.as_strings();
    let embeddings = state
        .model
        .encode(&inputs, state.config.max_input_length, inputs.len());

    Ok(Json(embeddings))
}

/// TEI-compatible model information endpoint.
#[utoipa::path(
    get,
    path = "/info",
    tag = "tei",
    responses(
        (status = 200, description = "Model information", body = ModelInfo),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
pub async fn tei_info(State(state): State<Arc<AppState>>) -> Json<ModelInfo> {
    Json(ModelInfo {
        model_id: state.model_id.clone(),
        max_input_length: state.config.max_input_length,
        embedding_dimension: state.embedding_dimension,
        pooling: "mean",
    })
}
