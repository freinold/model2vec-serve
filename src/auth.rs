//! API key authentication as a Tower layer.

use crate::errors::AppError;
use axum::{extract::Request, middleware::Next, response::Response};
use std::sync::Arc;

/// State required by the auth middleware.
#[derive(Clone)]
pub struct ApiKeyAuth {
    key: Arc<String>,
}

impl ApiKeyAuth {
    /// Create a new auth layer guarding the provided API key.
    #[must_use]
    pub fn new(key: String) -> Self {
        Self { key: Arc::new(key) }
    }
}

/// Middleware that checks the `Authorization: Bearer <key>` header.
///
/// # Errors
///
/// Returns `AppError::Unauthorized` when the header is missing, malformed, or
/// contains an incorrect key.
pub async fn require_api_key(
    auth: axum::extract::State<ApiKeyAuth>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let provided = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .map(str::trim);

    let valid = provided
        .and_then(|header| header.strip_prefix("Bearer "))
        .map(str::trim)
        == Some(auth.key.as_str());

    if !valid {
        return Err(AppError::Unauthorized);
    }

    Ok(next.run(request).await)
}
