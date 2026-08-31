//! Text Embedding Inference compatible endpoints.

use crate::{
    errors::AppError,
    routes::dto::{ErrorResponse, ModelInfo, TeiEmbedRequest},
    state::AppState,
    telemetry::RequestModelId,
};
use axum::{Extension, Json, extract::Path, extract::RawQuery, extract::State};
use std::sync::Arc;

/// Reject requests carrying the retired `model` query parameter.
///
/// The hidden model qualifier was removed in 0.5.0; model selection is now
/// expressed by the per-model paths `/tei/{model_id}/embed` and
/// `/tei/{model_id}/info`, while the root endpoints serve the default model.
///
/// # Errors
///
/// Returns [`AppError::BadRequest`] when the raw query string contains a
/// `model` key (any value, including empty).
fn reject_retired_model_qualifier(raw_query: Option<&str>) -> Result<(), AppError> {
    let Some(query) = raw_query else {
        return Ok(());
    };

    let has_model_key = query
        .split('&')
        .any(|pair| pair.split('=').next().is_some_and(|key| key == "model"));

    if has_model_key {
        return Err(AppError::BadRequest(
            "the 'model' query parameter is no longer supported; use the per-model endpoints /tei/{model_id}/embed and /tei/{model_id}/info instead"
                .to_string(),
        ));
    }

    Ok(())
}

/// Shared input validation for TEI embed requests.
///
/// # Errors
///
/// Returns [`AppError::BadRequest`] when `inputs` is empty or exceeds the
/// configured maximum batch size.
fn validate_tei_request(state: &AppState, request: &TeiEmbedRequest) -> Result<(), AppError> {
    if request.inputs.is_empty() {
        return Err(AppError::BadRequest("inputs cannot be empty".to_string()));
    }

    if request.inputs.len() > state.config.max_batch_size {
        return Err(AppError::BadRequest(format!(
            "batch size exceeds maximum of {}",
            state.config.max_batch_size
        )));
    }

    Ok(())
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
        (status = 400, description = "Invalid request or retired model qualifier", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Model inference error", body = ErrorResponse)
    )
)]
pub async fn tei_embed(
    State(state): State<Arc<AppState>>,
    Extension(model_id_ext): Extension<RequestModelId>,
    RawQuery(raw_query): RawQuery,
    Json(request): Json<TeiEmbedRequest>,
) -> Result<Json<Vec<Vec<f32>>>, AppError> {
    reject_retired_model_qualifier(raw_query.as_deref())?;
    validate_tei_request(&state, &request)?;

    let loaded = state.registry.resolve(None)?;
    let inputs = request.inputs.as_strings();
    let embeddings = loaded
        .model
        .encode(&inputs, loaded.max_input_length, inputs.len());

    model_id_ext.set(loaded.model_id.clone());

    Ok(Json(embeddings))
}

/// TEI-compatible per-model embed endpoint.
///
/// The model is selected exclusively by the `{model_id}` path segment (an
/// operator-configured alias or the model identifier's last segment); no
/// model qualifier is used.
///
/// # Errors
///
/// Returns `AppError::ModelRouteNotFound` when no loaded model matches the
/// path identifier, `AppError::BadRequest` for invalid input,
/// `AppError::Unauthorized` when authentication is enabled and fails, or
/// `AppError::Internal` if model inference fails.
#[utoipa::path(
    post,
    path = "/tei/{model_id}/embed",
    tag = "tei",
    params(
        ("model_id" = String, Path, description = "Model path identifier"),
    ),
    request_body = TeiEmbedRequest,
    responses(
        (status = 200, description = "Embeddings generated", body = Vec<Vec<f32>>),
        (status = 400, description = "Invalid request or retired model qualifier", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Unknown model path", body = ErrorResponse),
        (status = 500, description = "Model inference error", body = ErrorResponse)
    )
)]
pub async fn tei_per_model_embed(
    State(state): State<Arc<AppState>>,
    Extension(model_id_ext): Extension<RequestModelId>,
    Path(path_id): Path<String>,
    RawQuery(raw_query): RawQuery,
    Json(request): Json<TeiEmbedRequest>,
) -> Result<Json<Vec<Vec<f32>>>, AppError> {
    reject_retired_model_qualifier(raw_query.as_deref())?;
    validate_tei_request(&state, &request)?;

    let loaded = state
        .registry
        .get_by_path(&path_id)
        .ok_or_else(|| AppError::ModelRouteNotFound(path_id.clone()))?;

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
    RawQuery(raw_query): RawQuery,
) -> Result<Json<ModelInfo>, AppError> {
    reject_retired_model_qualifier(raw_query.as_deref())?;
    let loaded = state.registry.resolve(None)?;
    Ok(Json(ModelInfo {
        model_id: loaded.model_id.clone(),
        max_input_length: loaded.max_input_length,
        embedding_dimension: loaded.embedding_dimension,
        pooling: loaded.pooling,
    }))
}

/// TEI-compatible per-model model information endpoint.
///
/// Returns metadata for exactly the model addressed by the `{model_id}` path
/// segment (an operator-configured alias or the model identifier's last
/// segment); the reported `model_id` is always the canonical identifier.
///
/// # Errors
///
/// Returns `AppError::ModelRouteNotFound` when no loaded model matches the
/// path identifier, `AppError::Unauthorized` when authentication is enabled
/// and fails, or `AppError::Internal` on unexpected failures.
#[utoipa::path(
    get,
    path = "/tei/{model_id}/info",
    tag = "tei",
    params(
        ("model_id" = String, Path, description = "Model path identifier"),
    ),
    responses(
        (status = 200, description = "Model information", body = ModelInfo),
        (status = 400, description = "Retired model qualifier", body = ErrorResponse),
        (status = 404, description = "Unknown model path", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
pub async fn tei_per_model_info(
    State(state): State<Arc<AppState>>,
    Extension(model_id_ext): Extension<RequestModelId>,
    Path(path_id): Path<String>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<ModelInfo>, AppError> {
    reject_retired_model_qualifier(raw_query.as_deref())?;
    let loaded = state
        .registry
        .get_by_path(&path_id)
        .ok_or(AppError::ModelRouteNotFound(path_id))?;

    model_id_ext.set(loaded.model_id.clone());

    Ok(Json(ModelInfo {
        model_id: loaded.model_id.clone(),
        max_input_length: loaded.max_input_length,
        embedding_dimension: loaded.embedding_dimension,
        pooling: loaded.pooling,
    }))
}
