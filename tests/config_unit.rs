#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Unit tests for configuration parsing.

use clap::Parser;
use model2vec_serve::config::Config;

const DEFAULT_MODEL: &str = "minishlab/potion-multilingual-128M";
const CODE_MODEL: &str = "minishlab/potion-code-16M-v2";

#[test]
fn default_values_are_reasonable() {
    let config = Config {
        host: "0.0.0.0".to_string(),
        port: 8080,
        models: vec![DEFAULT_MODEL.to_string()],
        default_model: None,
        api_key: None,
        max_batch_size: 256,
        max_input_length: 512,
        log_level: "info".to_string(),
        request_timeout_seconds: 30,
    };

    assert_eq!(config.bind_address(), "0.0.0.0:8080");
    assert_eq!(config.max_batch_size, 256);
    assert_eq!(config.default_model(), Some(DEFAULT_MODEL.to_string()));
}

#[test]
fn parse_empty_cli_args_uses_default_model() {
    let config = Config::parse_from(Vec::<&str>::new());

    assert_eq!(config.models, vec![DEFAULT_MODEL]);
    assert_eq!(config.default_model(), Some(DEFAULT_MODEL.to_string()));
}

#[test]
fn parse_multiple_model_flags() {
    let config = Config::parse_from([
        "model2vec-serve",
        "--model",
        DEFAULT_MODEL,
        "--model",
        CODE_MODEL,
    ]);

    assert_eq!(config.models, vec![DEFAULT_MODEL, CODE_MODEL]);
    assert_eq!(config.default_model(), Some(DEFAULT_MODEL.to_string()));
}

#[test]
fn parse_comma_separated_model_flag() {
    let config = Config::parse_from([
        "model2vec-serve",
        "--model",
        &format!("{DEFAULT_MODEL},{CODE_MODEL}"),
    ]);

    assert_eq!(config.models, vec![DEFAULT_MODEL, CODE_MODEL]);
}

#[test]
fn explicit_default_model_is_used() {
    let config = Config::parse_from([
        "model2vec-serve",
        "--model",
        DEFAULT_MODEL,
        "--model",
        CODE_MODEL,
        "--default-model",
        CODE_MODEL,
    ]);

    assert_eq!(config.default_model(), Some(CODE_MODEL.to_string()));
}
