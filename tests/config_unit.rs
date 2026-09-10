#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Unit tests for configuration parsing.

use clap::{CommandFactory, Parser};
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
        tls_cert: None,
        tls_key: None,
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

#[test]
fn alias_with_whitespace_is_rejected() {
    let err = Config::try_parse_from(["model2vec-serve", "--model-alias", "some-model=two words"])
        .unwrap_err();

    assert!(err.to_string().contains("whitespace"));
}

// TLS configuration parsing (spec 007).

#[test]
fn tls_defaults_to_disabled() {
    let config = Config::parse_from(Vec::<&str>::new());

    assert!(config.tls_cert.is_none());
    assert!(config.tls_key.is_none());
    assert!(matches!(
        config.tls_mode(),
        Ok(model2vec_serve::config::TlsMode::Disabled)
    ));
}

#[test]
fn parse_tls_cert_and_key_flags() {
    let config = Config::parse_from([
        "model2vec-serve",
        "--tls-cert",
        "/tmp/cert.pem",
        "--tls-key",
        "/tmp/key.pem",
    ]);

    assert_eq!(
        config.tls_cert.as_deref(),
        Some(std::path::Path::new("/tmp/cert.pem"))
    );
    assert_eq!(
        config.tls_key.as_deref(),
        Some(std::path::Path::new("/tmp/key.pem"))
    );
    assert!(matches!(
        config.tls_mode(),
        Ok(model2vec_serve::config::TlsMode::Enabled { .. })
    ));
}

#[test]
fn tls_cert_only_is_rejected() {
    let config = Config::parse_from(["model2vec-serve", "--tls-cert", "/tmp/cert.pem"]);

    let err = config.tls_mode().unwrap_err();
    assert_eq!(
        err,
        "TLS requires both --tls-cert and --tls-key; only --tls-cert was provided"
    );
}

#[test]
fn tls_key_only_is_rejected() {
    let config = Config::parse_from(["model2vec-serve", "--tls-key", "/tmp/key.pem"]);

    let err = config.tls_mode().unwrap_err();
    assert_eq!(
        err,
        "TLS requires both --tls-cert and --tls-key; only --tls-key was provided"
    );
}

#[test]
fn tls_flags_declare_env_aliases() {
    // Environment manipulation is unsafe in edition 2024 and forbidden in
    // this crate, so the env wiring is asserted via clap argument metadata
    // instead of executing a parse against process env vars.
    let command = Config::command();

    let cert_arg = command
        .get_arguments()
        .find(|arg| arg.get_id() == "tls_cert")
        .expect("--tls-cert argument must exist");
    let key_arg = command
        .get_arguments()
        .find(|arg| arg.get_id() == "tls_key")
        .expect("--tls-key argument must exist");

    assert_eq!(
        cert_arg.get_env().map(std::ffi::OsStr::to_string_lossy),
        Some(std::borrow::Cow::Borrowed("TLS_CERT"))
    );
    assert_eq!(
        key_arg.get_env().map(std::ffi::OsStr::to_string_lossy),
        Some(std::borrow::Cow::Borrowed("TLS_KEY"))
    );
}
