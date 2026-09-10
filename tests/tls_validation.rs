#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

mod common;

// Unit tests for the TLS startup validation matrix (spec 007, US3).
//
// Error identifiers refer to
// specs/007-tls-https-support/contracts/configuration.md. E1 (single-sided
// configuration) is covered by the Config tests in tests/config_unit.rs; the
// remaining rows are asserted here against `tls::load` directly, without a
// listener.

use common::{
    encrypted_key_tls_pair, expired_tls_pair, garbage_cert_tls_pair, garbage_key_tls_pair,
    mismatched_tls_pair, tls_pair,
};
use model2vec_serve::tls::TlsError;

#[test]
fn e2_missing_certificate_file_is_reported_with_path() {
    let files = tls_pair();
    let Err(err) = model2vec_serve::tls::load(&files.missing_path(), &files.key_path) else {
        panic!("missing cert must fail")
    };

    assert!(matches!(
        err,
        TlsError::Read {
            kind: "certificate",
            ..
        }
    ));
    assert!(
        err.to_string()
            .contains("failed to read TLS certificate file")
    );
    assert!(
        err.to_string()
            .contains(files.missing_path().to_string_lossy().as_ref())
    );
}

#[test]
fn e2_missing_key_file_is_reported_with_path() {
    let files = tls_pair();
    let Err(err) = model2vec_serve::tls::load(&files.cert_path, &files.missing_path()) else {
        panic!("missing key must fail")
    };

    assert!(matches!(
        err,
        TlsError::Read {
            kind: "private key",
            ..
        }
    ));
    assert!(
        err.to_string()
            .contains("failed to read TLS private key file")
    );
}

#[test]
fn e3_garbage_certificate_is_rejected() {
    let files = garbage_cert_tls_pair();
    let Err(err) = model2vec_serve::tls::load(&files.cert_path, &files.key_path) else {
        panic!("garbage cert must fail")
    };

    assert!(matches!(err, TlsError::InvalidPem { .. }));
    assert_eq!(
        err.to_string(),
        format!(
            "TLS certificate file '{}' is not a valid PEM X.509 certificate",
            files.cert_path.display()
        )
    );
}

#[test]
fn e3_garbage_private_key_is_rejected() {
    let files = garbage_key_tls_pair();
    let Err(err) = model2vec_serve::tls::load(&files.cert_path, &files.key_path) else {
        panic!("garbage key must fail")
    };

    assert_eq!(
        err.to_string(),
        format!(
            "TLS private key file '{}' is not a valid PEM private key",
            files.key_path.display()
        )
    );
}

#[test]
fn e4_encrypted_private_key_is_rejected() {
    let files = encrypted_key_tls_pair();
    let Err(err) = model2vec_serve::tls::load(&files.cert_path, &files.key_path) else {
        panic!("encrypted key must fail")
    };

    assert!(matches!(err, TlsError::EncryptedKey { .. }));
    assert_eq!(
        err.to_string(),
        format!(
            "TLS private key file '{}' is encrypted; encrypted keys are not supported",
            files.key_path.display()
        )
    );
}

#[test]
fn e5_mismatched_pair_is_rejected_with_both_paths() {
    let files = mismatched_tls_pair();
    let Err(err) = model2vec_serve::tls::load(&files.cert_path, &files.key_path) else {
        panic!("mismatched pair must fail")
    };

    assert!(matches!(err, TlsError::Mismatch { .. }));
    assert!(
        err.to_string()
            .contains("does not match the private key in")
    );
    assert!(
        err.to_string()
            .contains(files.cert_path.to_string_lossy().as_ref())
    );
    assert!(
        err.to_string()
            .contains(files.key_path.to_string_lossy().as_ref())
    );
}

#[test]
fn e6_expired_certificate_loads_with_expiry_metadata() {
    let files = expired_tls_pair();
    let setup = model2vec_serve::tls::load(&files.cert_path, &files.key_path)
        .expect("expired but valid certificate must load");

    let expiry = setup
        .metadata
        .expired_since
        .expect("expired metadata must be set");
    assert!(!expiry.is_empty());
}

#[test]
fn valid_pair_exposes_certificate_metadata() {
    let files = tls_pair();
    let setup = model2vec_serve::tls::load(&files.cert_path, &files.key_path)
        .expect("valid pair must load");

    assert!(setup.metadata.expired_since.is_none());
    assert!(!setup.metadata.subject.is_empty());
    assert!(!setup.metadata.issuer.is_empty());
    assert!(!setup.metadata.not_after.is_empty());
}

#[test]
fn errors_and_metadata_never_contain_key_material() {
    let files = mismatched_tls_pair();
    let Err(err) = model2vec_serve::tls::load(&files.cert_path, &files.key_path) else {
        panic!("mismatch")
    };

    let rendered = err.to_string();
    assert!(
        !rendered.contains("BEGIN"),
        "error must not embed PEM content"
    );
    assert!(
        !rendered.contains("PRIVATE"),
        "error must not embed key material"
    );

    let files = tls_pair();
    let setup = model2vec_serve::tls::load(&files.cert_path, &files.key_path).expect("valid pair");
    let rendered_metadata = format!("{:?}", setup.metadata);
    assert!(
        !rendered_metadata.contains("BEGIN"),
        "metadata must never embed PEM content"
    );
}
