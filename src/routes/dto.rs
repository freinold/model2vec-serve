//! Shared request and response data transfer objects.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// OpenAI-compatible embeddings request.
#[derive(Debug, Deserialize, ToSchema)]
pub struct EmbeddingRequest {
    /// Input text or list of texts.
    #[schema(value_type = Object, example = json!("Hello world"))]
    pub input: EmbeddingInput,

    /// Optional model identifier. Must match the loaded model.
    pub model: Option<String>,

    /// Desired encoding format: `float` or `base64`.
    #[serde(default = "default_encoding")]
    pub encoding_format: String,
}

/// Input accepted by the embeddings endpoint.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum EmbeddingInput {
    /// A single text string.
    Single(String),
    /// A list of text strings.
    Batch(Vec<String>),
}

impl EmbeddingInput {
    /// Return the contained strings as a slice.
    #[must_use]
    pub fn as_strings(&self) -> Vec<String> {
        match self {
            Self::Single(s) => vec![s.clone()],
            Self::Batch(v) => v.clone(),
        }
    }

    /// Return the number of inputs.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::Single(_) => 1,
            Self::Batch(v) => v.len(),
        }
    }

    /// Returns `true` if there are no inputs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn default_encoding() -> String {
    "float".to_string()
}

/// A single embedding object.
#[derive(Debug, Serialize, ToSchema)]
pub struct EmbeddingObject {
    /// Object type, always `embedding`.
    pub object: &'static str,
    /// Index of the input this embedding corresponds to.
    pub index: usize,
    /// The embedding vector or base64-encoded bytes.
    pub embedding: EmbeddingVector,
}

/// Embedding value representation.
#[derive(Debug, Serialize, ToSchema)]
#[serde(untagged)]
pub enum EmbeddingVector {
    /// Float array.
    Float(Vec<f32>),
    /// Base64-encoded string.
    Base64(String),
}

/// Token usage metadata.
#[derive(Debug, Serialize, ToSchema)]
pub struct Usage {
    /// Tokens in the prompt/input.
    pub prompt_tokens: usize,
    /// Total tokens consumed.
    pub total_tokens: usize,
}

/// OpenAI-compatible embeddings response.
#[derive(Debug, Serialize, ToSchema)]
pub struct EmbeddingResponse {
    /// Object type, always `list`.
    pub object: &'static str,
    /// Embedding results.
    pub data: Vec<EmbeddingObject>,
    /// Model identifier.
    pub model: String,
    /// Token usage.
    pub usage: Usage,
}

/// Standard error response body.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Short error code.
    pub error: &'static str,
    /// Human-readable message.
    pub message: String,
}

/// TEI-compatible embed request.
#[derive(Debug, Deserialize, ToSchema)]
pub struct TeiEmbedRequest {
    /// Input text or list of texts.
    #[schema(value_type = Object, example = json!("Hello world"))]
    pub inputs: EmbeddingInput,
}

/// TEI-compatible model information response.
#[derive(Debug, Serialize, ToSchema)]
pub struct ModelInfo {
    /// Model identifier.
    pub model_id: String,
    /// Maximum input length in tokens.
    pub max_input_length: usize,
    /// Embedding vector dimension.
    pub embedding_dimension: usize,
    /// Pooling method.
    pub pooling: &'static str,
}

/// Health status response.
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthStatus {
    /// `healthy` or `unhealthy`.
    pub status: &'static str,
    /// Whether the service is ready to serve requests.
    pub ready: bool,
    /// Human-readable description.
    pub message: String,
}
