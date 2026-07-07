//! Thin wrapper around the `model2vec-rs` inference crate.

use model2vec_rs::model::StaticModel;

/// Wrapper around a loaded `StaticModel`.
#[derive(Clone)]
pub struct EmbeddingModel {
    inner: StaticModel,
    dimension: usize,
}

impl EmbeddingModel {
    /// Load a model from a Hugging Face id or a local path.
    ///
    /// # Errors
    ///
    /// Returns an error if the model files cannot be found or parsed.
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let inner = StaticModel::from_pretrained(path, None, None, None)?;
        let dimension = inner.encode_single("hello").len();

        Ok(Self { inner, dimension })
    }

    /// Return the embedding dimension of the loaded model.
    #[must_use]
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Encode a batch of input strings into embeddings.
    ///
    /// # Panics
    ///
    /// Never panics; returns an empty vector only if the input is empty.
    #[must_use]
    pub fn encode(&self, inputs: &[String], max_length: usize, batch_size: usize) -> Vec<Vec<f32>> {
        if inputs.is_empty() {
            return Vec::new();
        }

        self.inner
            .encode_with_args(inputs, Some(max_length), batch_size)
    }

    /// Encode a single input string.
    #[must_use]
    pub fn encode_single(&self, input: &str, max_length: usize) -> Vec<f32> {
        self.inner
            .encode_with_args(&[input.to_owned()], Some(max_length), 1)
            .into_iter()
            .next()
            .unwrap_or_default()
    }
}
