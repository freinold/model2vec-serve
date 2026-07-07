#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Unit tests for configuration parsing.

use model2vec_serve::config::Config;

#[test]
fn default_values_are_reasonable() {
    let config = Config {
        host: "0.0.0.0".to_string(),
        port: 8080,
        model: "minishlab/potion-base-2M".to_string(),
        api_key: None,
        max_batch_size: 256,
        max_input_length: 512,
        log_level: "info".to_string(),
        request_timeout_seconds: 30,
    };

    assert_eq!(config.bind_address(), "0.0.0.0:8080");
    assert_eq!(config.max_batch_size, 256);
}
