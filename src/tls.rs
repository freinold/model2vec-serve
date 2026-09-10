//! TLS setup: load, validate, and build the HTTPS listener configuration.
//!
//! Validation happens before the listener binds so a bad configuration fails
//! startup with an actionable message (spec 007, FR-004). Message shapes are
//! part of the configuration contract
//! (`specs/007-tls-https-support/contracts/configuration.md`) and are asserted
//! verbatim by tests. Private key material is never included in errors or
//! logs (FR-007).

use rustls::ServerConfig;
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use x509_parser::parse_x509_certificate;
use x509_parser::time::ASN1Time;

/// Error raised while loading or validating the TLS certificate package.
///
/// Each variant maps to one row of the configuration contract's startup error
/// matrix (E2 read failure, E3 unparseable content, E4 encrypted key, E5
/// mismatched pair).
#[derive(Debug, thiserror::Error)]
pub enum TlsError {
    /// A TLS file could not be read from disk.
    #[error("failed to read TLS {kind} file '{path}': {source}")]
    Read {
        /// Whether the certificate or the private key file failed.
        kind: &'static str,
        /// The configured file path.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// A TLS file does not contain the expected PEM block.
    #[error("TLS {kind} file '{path}' is not a valid PEM {expected}")]
    InvalidPem {
        /// Whether the certificate or the private key file failed.
        kind: &'static str,
        /// The configured file path.
        path: PathBuf,
        /// The expected PEM content, spelled out for the operator.
        expected: &'static str,
    },

    /// The private key is encrypted, which is not supported.
    #[error("TLS private key file '{path}' is encrypted; encrypted keys are not supported")]
    EncryptedKey {
        /// The configured private key file path.
        path: PathBuf,
    },

    /// The certificate and private key do not form a matching pair.
    #[error("TLS certificate in '{cert}' does not match the private key in '{key}'")]
    Mismatch {
        /// The configured certificate file path.
        cert: PathBuf,
        /// The configured private key file path.
        key: PathBuf,
    },
}

/// Metadata extracted from the leaf certificate for startup logging.
///
/// Only descriptive strings are carried; no certificate or key bytes are ever
/// embedded (FR-007).
#[derive(Clone, Debug)]
pub struct CertMetadata {
    /// Subject distinguished name of the leaf certificate.
    pub subject: String,
    /// Issuer distinguished name of the leaf certificate.
    pub issuer: String,
    /// Human-readable expiry date of the leaf certificate.
    pub not_after: String,
    /// Expiry date string when the certificate is already expired.
    pub expired_since: Option<String>,
}

/// Ready-to-serve TLS configuration plus certificate metadata.
pub struct TlsSetup {
    /// Listener configuration for `axum-server`.
    pub config: axum_server::tls_rustls::RustlsConfig,
    /// Leaf certificate metadata for startup logs.
    pub metadata: CertMetadata,
}

/// Load and validate the TLS certificate package.
///
/// Validates file existence/readability, PEM parseability, the absence of
/// key encryption, and the certificate/key pair match. Only modern TLS
/// versions (1.2/1.3) are negotiated, enforced by the rustls stack itself.
///
/// # Errors
///
/// Returns [`TlsError`] for every failure mode in the configuration
/// contract's startup error matrix (E2, E3, E4, E5).
pub fn load(cert_path: &Path, key_path: &Path) -> Result<TlsSetup, TlsError> {
    install_crypto_provider();

    let cert_bytes = read_file("certificate", cert_path)?;
    let key_bytes = read_file("private key", key_path)?;

    let certs = parse_certificates(&cert_bytes).map_err(|_| TlsError::InvalidPem {
        kind: "certificate",
        path: cert_path.to_path_buf(),
        expected: "X.509 certificate",
    })?;
    let metadata = extract_metadata(&certs).map_err(|_| TlsError::InvalidPem {
        kind: "certificate",
        path: cert_path.to_path_buf(),
        expected: "X.509 certificate",
    })?;

    if key_is_encrypted(&key_bytes) {
        return Err(TlsError::EncryptedKey {
            path: key_path.to_path_buf(),
        });
    }

    let key = PrivateKeyDer::from_pem_slice(&key_bytes).map_err(|_| TlsError::InvalidPem {
        kind: "private key",
        path: key_path.to_path_buf(),
        expected: "private key",
    })?;

    let server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|_| TlsError::Mismatch {
            cert: cert_path.to_path_buf(),
            key: key_path.to_path_buf(),
        })?;

    log_startup(cert_path, &metadata);

    Ok(TlsSetup {
        config: axum_server::tls_rustls::RustlsConfig::from_config(Arc::new(server_config)),
        metadata,
    })
}

/// Read a TLS file from disk, mapping I/O failures to the contract error.
fn read_file(kind: &'static str, path: &Path) -> Result<Vec<u8>, TlsError> {
    std::fs::read(path).map_err(|source| TlsError::Read {
        kind,
        path: path.to_path_buf(),
        source,
    })
}

/// Parse all PEM certificate blocks from the certificate file.
fn parse_certificates(
    bytes: &[u8],
) -> Result<Vec<CertificateDer<'static>>, rustls_pki_types::pem::Error> {
    CertificateDer::pem_slice_iter(bytes).collect()
}

/// Detect PEM markers of encrypted private keys (E4).
///
/// Covers the PKCS#8 `ENCRYPTED PRIVATE KEY` label as well as the legacy
/// `Proc-Type`/`DEK-Info` headers of traditional encrypted PEM. Detection is
/// marker-based so encrypted keys are reported precisely instead of surfacing
/// as a parse failure.
fn key_is_encrypted(bytes: &[u8]) -> bool {
    const MARKERS: [&[u8]; 3] = [
        b"ENCRYPTED PRIVATE KEY",
        b"Proc-Type: 4,ENCRYPTED",
        b"DEK-Info:",
    ];
    MARKERS
        .iter()
        .any(|marker| bytes.windows(marker.len()).any(|window| window == *marker))
}

/// Extract descriptive metadata from the leaf certificate.
///
/// An expired but otherwise parseable certificate is not an error: the expiry
/// is carried in [`CertMetadata::expired_since`] and logged as a warning so
/// availability is preserved (spec 007 assumption).
fn extract_metadata(
    certs: &[CertificateDer<'_>],
) -> Result<CertMetadata, x509_parser::error::X509Error> {
    let Some(leaf) = certs.first() else {
        return Err(x509_parser::error::X509Error::InvalidCertificate);
    };

    let (_, certificate) = parse_x509_certificate(leaf.as_ref())?;
    let validity = &certificate.validity;
    let not_after = validity.not_after;

    Ok(CertMetadata {
        subject: certificate.subject().to_string(),
        issuer: certificate.issuer().to_string(),
        not_after: not_after.to_string(),
        expired_since: (not_after < ASN1Time::now()).then(|| not_after.to_string()),
    })
}

/// Log the TLS state: enabled plus leaf certificate metadata (FR-007).
///
/// An expired certificate logs a prominent warning naming the expiry; neither
/// message ever contains key material.
fn log_startup(cert_path: &Path, metadata: &CertMetadata) {
    if let Some(expiry) = &metadata.expired_since {
        tracing::warn!(
            "TLS certificate '{}' expired on {expiry}; proceeding",
            cert_path.display()
        );
    }
    tracing::info!(
        "TLS enabled; certificate subject='{}' issuer='{}' expires='{}'",
        metadata.subject,
        metadata.issuer,
        metadata.not_after
    );
}

/// Install the process-default crypto provider once.
///
/// A failure means a provider is already installed (for example by another
/// component), which is acceptable.
fn install_crypto_provider() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
}
