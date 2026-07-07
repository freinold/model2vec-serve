//! Structured application errors and their HTTP response mapping.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

/// Application-level error variants.
#[derive(Debug, Error)]
pub enum AppError {
    /// Client sent an invalid request.
    #[error("invalid request: {0}")]
    BadRequest(String),

    /// Authentication failed or was missing.
    #[error("unauthorized")]
    Unauthorized,

    /// The requested model or embedding operation failed.
    #[error("model error: {0}")]
    ModelError(String),

    /// An unexpected internal failure occurred.
    #[error("internal server error")]
    Internal,
}

/// JSON body returned for every error response.
#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code) = match &self {
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "invalid_request"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
            AppError::ModelError(_) | AppError::Internal => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            }
        };

        let body = ErrorBody {
            error: error_code,
            message: self.to_string(),
        };

        (status, Json(body)).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!("internal error: {err:?}");
        Self::Internal
    }
}
