//! Runtime configuration parsed from CLI arguments and environment variables.

use clap::Parser;

/// Configuration for the model2vec-serve HTTP server.
#[derive(Parser, Clone, Debug)]
#[command(name = "model2vec-serve", version, about)]
pub struct Config {
    /// Network interface to bind to.
    #[arg(long, default_value = "0.0.0.0", env = "HOST")]
    pub host: String,

    /// Port to listen on.
    #[arg(long, default_value_t = 8080, env = "PORT")]
    pub port: u16,

    /// Hugging Face model id or local path to a model2vec model directory.
    #[arg(
        long,
        default_value = "minishlab/potion-multilingual-128M",
        env = "MODEL"
    )]
    pub model: String,

    /// Optional API key. When set, embedding endpoints require a Bearer token.
    #[arg(long, env = "API_KEY")]
    pub api_key: Option<String>,

    /// Maximum number of input strings accepted in a single request.
    #[arg(long, default_value_t = 256, env = "MAX_BATCH_SIZE")]
    pub max_batch_size: usize,

    /// Maximum tokens per input string.
    #[arg(long, default_value_t = 512, env = "MAX_INPUT_LENGTH")]
    pub max_input_length: usize,

    /// Log level (e.g., trace, debug, info, warn, error).
    #[arg(long, default_value = "info", env = "LOG_LEVEL")]
    pub log_level: String,

    /// Per-request timeout in seconds.
    #[arg(long, default_value_t = 30, env = "REQUEST_TIMEOUT_SECONDS")]
    pub request_timeout_seconds: u64,
}

impl Config {
    /// Returns the socket address the server should bind to.
    #[must_use]
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
