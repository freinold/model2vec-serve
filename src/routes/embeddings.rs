//! OpenAI-compatible `/v1/embeddings` endpoint.

use crate::{
    errors::AppError,
    routes::dto::{
        EmbeddingObject, EmbeddingRequest, EmbeddingResponse, EmbeddingVector, ErrorResponse, Usage,
    },
    state::AppState,
};
use axum::{Json, extract::State};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::sync::Arc;

/// Encode a vector of floats as base64.
fn encode_base64(values: &[f32]) -> String {
    let bytes: Vec<u8> = values.iter().flat_map(|f| f.to_le_bytes()).collect();
    STANDARD.encode(bytes)
}

/// OpenAI-compatible embeddings endpoint.
///
/// # Errors
///
/// Returns `AppError::BadRequest` for invalid input, `AppError::Unauthorized`
/// when authentication is enabled and fails, or `AppError::Internal` if model
/// inference fails.
#[utoipa::path(
    post,
    path = "/v1/embeddings",
    tag = "embeddings",
    request_body = EmbeddingRequest,
    responses(
        (status = 200, description = "Embeddings generated", body = EmbeddingResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Model inference error", body = ErrorResponse)
    )
)]
pub async fn create_embeddings(
    State(state): State<Arc<AppState>>,
    Json(request): Json<EmbeddingRequest>,
) -> Result<Json<EmbeddingResponse>, AppError> {
    validate_request(&request, &state)?;

    let inputs = request.input.as_strings();
    let embeddings = state
        .model
        .encode(&inputs, state.config.max_input_length, inputs.len());

    let data: Vec<EmbeddingObject> = embeddings
        .into_iter()
        .enumerate()
        .map(|(index, vector)| {
            let embedding = if request.encoding_format == "base64" {
                EmbeddingVector::Base64(encode_base64(&vector))
            } else {
                EmbeddingVector::Float(vector)
            };
            EmbeddingObject {
                object: "embedding",
                index,
                embedding,
            }
        })
        .collect();

    let prompt_tokens = inputs.iter().map(String::len).sum::<usize>() / 4;
    let response = EmbeddingResponse {
        object: "list",
        data,
        model: state.model_id.clone(),
        usage: Usage {
            prompt_tokens,
            total_tokens: prompt_tokens,
        },
    };

    Ok(Json(response))
}

fn validate_request(request: &EmbeddingRequest, state: &AppState) -> Result<(), AppError> {
    if request.input.is_empty() {
        return Err(AppError::BadRequest("input cannot be empty".to_string()));
    }

    if request.input.len() > state.config.max_batch_size {
        return Err(AppError::BadRequest(format!(
            "batch size exceeds maximum of {}",
            state.config.max_batch_size
        )));
    }

    if !matches!(request.encoding_format.as_str(), "float" | "base64") {
        return Err(AppError::BadRequest(
            "encoding_format must be 'float' or 'base64'".to_string(),
        ));
    }

    if let Some(ref model) = request.model {
        if model != &state.model_id {
            return Err(AppError::BadRequest(format!(
                "model '{model}' does not match loaded model '{}'",
                state.model_id
            )));
        }
    }

    Ok(())
}
