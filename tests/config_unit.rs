#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Unit tests for configuration parsing.

use clap::Parser;
use model2vec_serve::config::Config;

const DEFAULT_MODEL: &str = "minishlab/potion-multilingual-128M";

#[test]
fn default_values_are_reasonable() {
    let config = Config {
        host: "0.0.0.0".to_string(),
        port: 8080,
        model: DEFAULT_MODEL.to_string(),
        api_key: None,
        max_batch_size: 256,
        max_input_length: 512,
        log_level: "info".to_string(),
        request_timeout_seconds: 30,
    };

    assert_eq!(config.bind_address(), "0.0.0.0:8080");
    assert_eq!(config.max_batch_size, 256);
    assert_eq!(config.model, DEFAULT_MODEL);
}

#[test]
fn parse_empty_cli_args_uses_default_model() {
    let config = Config::parse_from(Vec::<&str>::new());

    assert_eq!(config.model, DEFAULT_MODEL);
}
