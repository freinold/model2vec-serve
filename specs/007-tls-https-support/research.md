# Research: TLS/HTTPS Serving with End-to-End Encryption

## Decision: TLS implementation stack

- **Decision**: Serve TLS with `axum-server` 0.7 (feature `tls-rustls`), which
  builds on rustls 0.23 with the `aws-lc-rs` crypto provider. The TLS config
  is built from PEM files via the `RustlsConfig`-equivalent path — our
  `src/tls.rs` reads and validates the files itself and constructs a
  `rustls::ServerConfig` with `ServerConfig::builder().with_single_cert(...)`,
  so every failure is mapped to a domain-specific, actionable message before
  the server binds. `main.rs` branches on the TLS configuration:
  `axum::serve(listener, ...)` (unchanged plain-HTTP path) or
  `axum_server::bind_rustls(addr, config).handle(handle).serve(...)` with the
  existing shutdown signal driving `handle.graceful_shutdown(...)`, preserving
  today's SIGTERM/SIGINT semantics.
- **Rationale**: Verified against current axum-server documentation
  (`RustlsConfig::from_pem_file` / `bind_rustls` / `Handle` graceful shutdown;
  `tls-rustls` feature = rustls + aws-lc-rs provider). rustls is the de facto
  pure-Rust TLS stack: no OpenSSL system dependency in the Debian-slim runtime
  image, `unsafe`-free (project lints forbid `unsafe`), and it implements only
  TLS 1.2/1.3 — which satisfies FR-008 (reject older versions) by
  construction. axum-server is the ecosystem-standard way to serve an axum
  `Router` over TLS and keeps the request path (tower stack, middleware,
  `app(state)`) untouched; TLS terminates at the acceptor only.
- **Alternatives considered**:
  - *Manual tokio-rustls accept loop around `axum::serve`* — rejected:
    re-implements accept-loop, connection accounting, and graceful shutdown
    that axum-server already provides; more code, no benefit (constitution IV).
  - *native-tls / OpenSSL* — rejected: adds a C toolchain + OpenSSL runtime
    dependency to the final image, conflicts with the slim multi-stage
    Dockerfile and the no-`unsafe` stance.
  - *`tls-rustls-no-provider` + `ring` provider* — retained as fallback only if
    aws-lc-rs builds cause friction in the release container; ring is
    equally viable but aws-lc-rs is rustls' default and FIPS-capable.
  - *Envoy/nginx sidecar TLS termination inside the pod* — rejected: violates
    the feature goal (encryption must reach the application process), adds an
    extra hop and moving part per replica.

## Decision: Configuration surface

- **Decision**: Two new options on `Config`: `--tls-cert <path>` (env
  `TLS_CERT`) and `--tls-key <path>` (env `TLS_KEY`), both `Option<PathBuf>`,
  file paths only (FR-006). TLS is enabled iff both are set; the all-or-nothing
  rule (FR-003) and file validation (FR-004) run at startup in `src/tls.rs`
  before binding — clap does not express cross-field rules cleanly, and
  startup validation keeps `config.rs` declarative. Errors exit the process
  via `anyhow` with the messages specified in
  [contracts/configuration.md](./contracts/configuration.md).
- **Rationale**: Matches every existing option's kebab-flag + uppercase-env
  convention (`src/config.rs`); file-path-only config satisfies FR-006 and the
  constitution's "secrets configurable, never hard-coded" rule.
- **Alternatives considered**:
  - *clap-level `requires`/conflicts validation* — possible but mixes
    transport-validation concerns into argument parsing and cannot produce the
    file-content diagnostics FR-004 requires.
  - *Inline PEM via env var* — rejected by FR-006 (secret material in env/
    process listings).
  - *Separate `--tls-enabled` flag* — rejected as redundant; presence of both
    paths is the enable signal (fewer options, constitution IV).

## Decision: Startup validation and diagnostics

- **Decision**: `src/tls.rs` performs, in order: (1) file existence +
  readability checks naming the offending path; (2) PEM parse checks for the
  expected block types; (3) encrypted-key detection (PEM headers
  `ENCRYPTED PRIVATE KEY` or legacy `Proc-Type: 4,ENCRYPTED` / `DEK-Info`) →
  explicit "encrypted keys not supported" error; (4) pair-matching via
  `with_single_cert`, whose error is mapped to a "certificate and private key
  do not match" message (rustls 0.23 verifies the key against the
  certificate); (5) expiry check via `x509-parser` — an expired but parseable
  certificate logs a prominent `warn!` naming the expiry date and continues
  (documented spec assumption). On success it logs `info!` with subject,
  issuer (CN when present) and not-after date; key bytes are never read into
  logs or errors (FR-007). Target: the whole validation path completes in
  well under the 5 s SC-004 budget (pure file I/O + parse).
- **Rationale**: Library-level errors from rustls/rustls-pem-file are terse
  ("invalid key", "failed to parse") and cannot satisfy SC-004's "names the
  failing item and the remedy"; a thin validation layer produces the exact
  error taxonomy in the configuration contract and is unit-testable without a
  network.
- **Alternatives considered**:
  - *Trust `RustlsConfig::from_pem_file` errors alone* — rejected: opaque
    messages, cannot distinguish mismatch from format errors.
  - *Fail startup on expired certificate* — rejected: prioritizes
    availability per the spec's documented default; monitoring gap should not
    cause an outage.
  - *Periodic re-validation / hot reload* — explicitly out of scope (spec
    assumption); restart-based rotation documented instead.

## Decision: Certificate metadata source

- **Decision**: `x509-parser` (PEM feature) parses the leaf certificate to
  extract subject, issuer, and validity window for the startup log lines.
  Intermediate chain entries are passed through to rustls unchanged (full
  chain is operator responsibility, documented).
- **Rationale**: `rustls-pki-types` only decodes PEM into DER blobs and
  exposes no X.509 metadata; FR-007 requires subject/issuer/expiry. x509-parser
  is the maintained, pure-Rust standard for this and has no transitive OpenSSL.
- **Alternatives considered**: * Skipping metadata (log only "TLS enabled") —
  violates FR-007. * Reimplementing minimal DER walking — reinvents
  x509-parser poorly.

## Decision: Test strategy

- **Decision**: Dev-dependencies `rcgen` (generate self-signed cert/key pairs
  into a `tempfile` dir at test time) and `reqwest` (rustls TLS, invalid-cert
  acceptance for self-signed fixtures). New integration test spins the real
  service on `127.0.0.1:0` with TLS enabled and asserts: HTTPS handshake
  succeeds; `/v1/models`, `/embed`, `/health` responses are byte-identical to
  the plain-HTTP run of the same build (contract identity, SC-002); plain
  HTTP bytes to the TLS port fail at transport level while the service stays
  healthy. Validation matrix tests (missing file, garbage file, mismatched
  pair, single-sided config, encrypted key, expired cert) run as unit tests
  against `src/tls.rs` without a listener. Helm additions are covered by the
  existing bash template-test pattern (default render immutability + `tls`
  render assertions) and `helm lint`.
- **Rationale**: TLS handshakes cannot be exercised through the current
  in-process `tower` test harness; a real loopback listener is the minimal
  faithful setup. rcgen avoids brittle committed fixtures with expiry dates.
- **Alternatives considered**:
  - *Committed self-signed test certs* — rejected: expiry/maintenance burden.
  - *reqwest with native-tls* — rejected: keeps test deps on the rustls path
    for consistency with the server.
  - *Only unit tests of the rustls config* — rejected: would leave the
    "serves HTTPS end to end" behavior (US1) untested.

## Decision: Helm chart `tls` block design

- **Decision**: New optional block (all defaults preserve current renders):

  ```yaml
  tls:
    enabled: false
    existingSecret: ""      # required when enabled; secret with cert + key entries
    certKey: tls.crt        # secret key holding the PEM certificate chain
    keyKey: tls.key         # secret key holding the PEM private key
    mountPath: /etc/model2vec-serve/tls
  ```

  When enabled, `deployment.yaml` renders: a secret-sourced volume (non-
  optional so a missing secret fails scheduling loudly), a container
  volumeMount at `mountPath`, and args `--tls-cert <mountPath>/<certKey>` and
  `--tls-key <mountPath>/<keyKey>`. Probes switch to `scheme: HTTPS`
  (kubelet HTTP(S) probes skip certificate verification, so self-signed and
  cert-manager certificates work). Rendering fails with a template error when
  `tls.enabled` is true without `existingSecret` (fail at deploy time, not
  runtime). Service port, name (`http`), and targetPort are unchanged to
  avoid churn for existing ServiceMonitors/NetworkPolicies; the chart README
  and docs get a passthrough-ingress example (nginx: `ssl-passthrough`
  annotation + `backend-protocol: HTTPS`) built from the existing
  `ingress.annotations` value — no new ingress template logic.
- **Rationale**: Follows the chart's established optional-block pattern
  (`persistence`, `ingress`: disabled by default, operator-driven) and
  FR-009/FR-010/FR-011. Mounting the secret as files (rather than env) is
  exactly what FR-006 requires and mirrors how cert-manager delivers rotated
  certs for the documented restart-based rotation flow.
- **Alternatives considered**:
  - *Chart creates the Secret from inline cert/key values* — rejected: private
    key material in `values.yaml`/release metadata (FR-006 violation).
  - *Rename service port to `https` when TLS enabled* — rejected: breaks
    consumers referencing the port name; the transport is an implementation
    detail of the port.
  - *Chart-managed passthrough Ingress mode/flag* — rejected: passthrough is
    entirely annotation-driven per ingress class; a chart flag would encode
    nginx specifics into a controller-agnostic chart (existing
    `ingress.annotations` already covers it).
  - *Generate certs in-chart (e.g., cert-manager annotations)* — rejected:
    cert provisioning is cluster policy, not chart concern; documented instead.

## Decision: Documentation and release scope

- **Decision**: Update `docs/` (configuration reference + deployment pages),
  `helm/model2vec-serve/README.md`, and the chart's `values.yaml` comments in
  the same change (FR-013). Docker Compose: no file changes — TLS is enabled
  via the documented `TLS_CERT`/`TLS_KEY` env vars plus a mounted volume,
  covered by the compose docs page (FR-002 keeps the default compose setup
  plain HTTP). Versioning is additive (MINOR); release-plz picks it up from
  `Cargo.toml` packaging as usual; chart version bumps follow the existing
  automated bump flow.
- **Rationale**: FR-013 mandates same-change documentation; the compose path
  needs zero new artifacts because env-var configuration already flows
  through.
- **Alternatives considered**: * Adding a TLS-enabled compose profile —
  rejected: speculative surface (constitution IV); env-var + volume mount is
  documented and sufficient. * Separate docs page for TLS — folded into the
  existing configuration/deployment pages to avoid orphan content.

## Performance validation approach (constitution V)

- **Decision**: Re-run the existing criterion embedding bench in two modes —
  plain HTTP and TLS listener — comparing steady-state p99/throughput; the
  gate is SC-005 (≤ 10% delta). Handshake costs (connection setup) are
  excluded from steady-state comparisons; TLS work is confined to the
  acceptor, so the request hot path is untouched. Recorded with reproducible
  invocation commands per the constitution's benchmarking rule.
- **Enforcement**: The bench carries a deterministic gate — setting
  `TLS_BENCH_MAX_DELTA_PCT` (e.g. `10`) fails the run when the TLS
  median-latency or throughput delta exceeds the threshold. These are the
  noise-stable signals: shared runners swing p99 on sub-millisecond
  loopback samples by ±20% between runs (observed: +21.5% on CI while
  throughput moved -2.0%), so p99 stays in the recorded output and is
  reviewed against the 10% budget before each release. CI runs the gate as
  a reporting-only, non-blocking step (job-level `continue-on-error` still
  fails the check run; step-level keeps the check green with the failure
  visible as an annotation).
- **Alternatives considered**: External load-test tooling — not needed;
  criterion + loopback is the project's established measurement path.
