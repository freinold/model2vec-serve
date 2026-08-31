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
        model_owner: "minishlab".to_string(),
        model_alias: Vec::new(),
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

#[test]
fn parse_single_model_alias_flag() {
    let config = Config::parse_from([
        "model2vec-serve",
        "--model-alias",
        "minishlab/potion-multilingual-128M=potion-multi",
    ]);

    assert_eq!(
        config.model_alias,
        vec![(
            "minishlab/potion-multilingual-128M".to_string(),
            "potion-multi".to_string()
        )]
    );
}

#[test]
fn parse_multiple_model_alias_flags() {
    let config = Config::parse_from([
        "model2vec-serve",
        "--model-alias",
        &format!("{DEFAULT_MODEL}=potion-multi"),
        "--model-alias",
        &format!("{CODE_MODEL}=potion-code"),
    ]);

    assert_eq!(
        config.model_alias,
        vec![
            (DEFAULT_MODEL.to_string(), "potion-multi".to_string()),
            (CODE_MODEL.to_string(), "potion-code".to_string()),
        ]
    );
}

#[test]
fn parse_comma_separated_model_alias_flag() {
    let config = Config::parse_from([
        "model2vec-serve",
        "--model-alias",
        &format!("{CODE_MODEL}=code,minishlab/potion-base-2M=base"),
    ]);

    assert_eq!(
        config.model_alias,
        vec![
            (CODE_MODEL.to_string(), "code".to_string()),
            ("minishlab/potion-base-2M".to_string(), "base".to_string()),
        ]
    );
}

#[test]
fn malformed_model_alias_is_rejected() {
    let err = Config::try_parse_from(["model2vec-serve", "--model-alias", "missing-equals-sign"])
        .unwrap_err();

    assert!(err.to_string().contains("KEY=ALIAS"));
}

#[test]
fn empty_alias_is_rejected() {
    let err =
        Config::try_parse_from(["model2vec-serve", "--model-alias", "some-model="]).unwrap_err();

    assert!(err.to_string().contains("alias must not be empty"));
}

#[test]
fn alias_with_slash_is_rejected() {
    let err = Config::try_parse_from(["model2vec-serve", "--model-alias", "some-model=foo/bar"])
        .unwrap_err();

    assert!(err.to_string().contains("single path segment"));
}
