# Implementation Plan: TLS/HTTPS Serving with End-to-End Encryption

**Branch**: `007-tls-https-support` | **Date**: 2026-09-10 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/007-tls-https-support/spec.md`

## Summary

Add optional TLS to model2vec-serve: when the operator supplies a certificate
and private key file paths, the existing single port serves HTTPS for every
endpoint (identical contracts); without TLS configuration the service serves
plain HTTP exactly as today. The deployment chart gains a `tls` values block
that mounts an operator-provided Kubernetes secret as cert/key files, switches
probes to HTTPS, and documents edge passthrough so traffic is encrypted on
every hop up to the application inside the container.

## Technical Context

**Language/Version**: Rust 1.85 (edition 2024), MSRV unchanged

**Primary Dependencies**: axum 0.8, tokio 1, tower/tower-http 0.7, clap 4.
**New**: `axum-server` 0.7 with the `tls-rustls` feature (rustls 0.23,
aws-lc-rs provider) for the TLS listener; `x509-parser` for certificate
metadata (subject/issuer/expiry) in startup logs; dev-only `rcgen` (test
certificates) and `reqwest` (rustls-based HTTPS test client). Justification in
[research.md](./research.md) → dependency decisions; alternatives recorded in
Complexity Tracking.

**Storage**: N/A — cert/key are files read at startup (filesystem or
secret-mounted volume); no new persistent state.

**Testing**: `cargo test` (existing contract/integration suites extended);
new TLS config unit tests; HTTPS integration tests using a real listener on an
ephemeral port with a self-signed test certificate; `helm lint` /
`helm template` bash tests extended for the `tls` block; existing compose
config test unchanged.

**Target Platform**: Linux containers (Debian slim runtime image), Kubernetes
via the existing Helm chart; local binary runs behave identically.

**Project Type**: web-service (single binary, single listener)

**Performance Goals**: steady-state HTTPS p99 latency and throughput within
10% of plain HTTP under identical load (spec SC-005); TLS misconfiguration
fails startup in < 5 s (spec SC-004); no added steady-state allocations on the
request hot path (TLS terminates at the acceptor layer only).

**Constraints**: `unsafe_code = "forbid"`, `unwrap_used = "deny"` (Cargo
lints); no key material in config values, logs, or error messages; no change
to HTTP request/response contracts; additive Helm values only — a default
`helm template` render MUST be equivalent to the chart before this feature
(no new resources, mounts, args, or probe schemes), excluding version-derived
metadata (`helm.sh/chart` label, default image tag from `appVersion`), which
changes on routine version bumps.

**Scale/Scope**: 1 binary (~10 source files touched), 1 Helm chart, ~3 new
test files; single-feature scope, no new endpoints.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Code Quality | PASS | New TLS module gets doc comments (missing_docs = warn), explicit error handling via typed errors mapped to anyhow startup errors; no unwrap/expect; lint gates run in CI. |
| II. Test Coverage | PASS | Plan includes unit tests for config validation, TLS config build/mismatch/invalid-file paths, HTTPS contract tests asserting unchanged API shape over TLS, and chart template tests. API contract paths tested over both HTTP and TLS. |
| III. API Conformity | PASS | No endpoint, request, response, or error-body change; contract tests re-run over HTTPS to prove shape identity; `/docs` unaffected (transport-only feature). |
| IV. Simplicity Over Complexity | PASS | Single listener (either HTTP or HTTPS), no hot reload, no mTLS, no cipher customization surface; every new dependency is required by an FR and alternatives are recorded in Complexity Tracking. |
| V. Performance Focus | PASS | Performance goal defined (SC-005 ≤ 10% delta); validation via existing criterion bench re-run over TLS vs HTTP in tasks phase; TLS cost confined to the acceptor, no per-request changes. |

No gate violations. Post-design re-check: PASS (see end of this file).

## Project Structure

### Documentation (this feature)

```text
specs/007-tls-https-support/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── configuration.md # Service config contract (CLI/env, validation, startup errors)
│   └── chart-values.md  # Helm `tls` values contract and rendered behavior
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── main.rs              # [MODIFIED] branch: plain axum::serve vs axum-server TLS serve (same shutdown semantics)
├── config.rs            # [MODIFIED] new TLS options + all-or-nothing validation
├── tls.rs               # [NEW] TLS setup: file validation, rustls config build, cert metadata logging
└── errors.rs            # [UNCHANGED] no HTTP-layer changes (transport-level feature)

helm/model2vec-serve/
├── values.yaml          # [MODIFIED] new `tls:` block (disabled by default)
├── templates/deployment.yaml  # [MODIFIED] secret volume+mount, cert/key args, HTTPS probes
├── templates/_helpers.tpl     # [MODIFIED] tls path/secret helpers if needed
└── README.md            # [MODIFIED] tls values docs + passthrough example

tests/
├── tls_config_unit.rs   # [NEW] config validation: all-or-nothing, defaults, env aliases
├── tls_integration.rs   # [NEW] HTTPS end-to-end over real listener (self-signed rcgen certs)
├── config_unit.rs       # [EXTENDED] existing config tests for new fields
└── common/mod.rs        # [EXTENDED] shared helpers for TLS test setup

docs/                    # [MODIFIED] configuration reference + deployment pages
.env.example             # [UNCHANGED] (TLS is file-path based; compose docs cover env vars)
```

**Structure Decision**: Single-project layout retained; the feature adds one
new module (`src/tls.rs`), extends config/startup/chart wiring, and adds test
files — matching the existing module conventions (handlers unchanged).

## Complexity Tracking

> Constitution IV requires recording rejected simpler alternatives.

| Violation / Choice | Why Needed | Simpler Alternative Rejected Because |
|--------------------|------------|--------------------------------------|
| New crate `axum-server` (tls-rustls) | Serving axum over rustls with graceful shutdown requires a TLS-capable acceptor | Hand-rolling a tokio-rustls accept loop around `axum::serve` re-implements connection limits/graceful shutdown for no gain; axum-server is the pattern used by the axum ecosystem for this exact need. |
| New crate `x509-parser` | FR-007 requires subject/issuer/expiry in startup logs | rustls-pki-types parses PEM but exposes no X.509 metadata; skipping metadata logging would violate FR-007. |
| Startup-time file validation layer in `src/tls.rs` | FR-004 requires actionable errors (missing file, mismatch, encrypted key) distinct from opaque TLS library errors | Relying on raw library errors alone yields messages like "invalid key" that fail SC-004's "names the failing item and the remedy". |
| Chart secret-mount design (`tls.existingSecret` + key names, no secret creation from values) | Cert/key must come from operator-managed secrets | Generating a Secret from inline values would put private key material in `values.yaml`/release metadata, violating FR-006 and secret-handling practice. |
| Self-signed cert generation (`rcgen`) as dev-dependency | Integration tests need real TLS handshakes on a real listener | Committing test certs is brittle (expiry); pure in-process tower tests cannot exercise rustls handshakes. |

## Post-Design Constitution Re-Check (after Phase 1)

- **I/II**: Contracts define startup error taxonomy and chart render behavior
  that map 1:1 to planned test cases (unit: validation matrix; integration:
  HTTPS contract run; helm: render assertions incl. default-render
  immutability).
- **III**: `contracts/configuration.md` explicitly states HTTP API contracts
  are unchanged; quickstart includes a contract-check over both schemes.
- **IV**: Design stays within declared bounds (no dual-port, no mTLS, no hot
  reload); the only new runtime surface is two config options + one chart
  block. Dependencies justified above.
- **V**: Bench comparison (HTTP vs HTTPS) is scheduled in the tasks phase;
  goal thresholds copied from spec SC-005.

Result: PASS — no violations introduced by the design.

## Benchmark results (SC-005 evidence)

Reproducible invocation (2026-09-10, loopback, minishlab/potion-base-2M,
64-input batch × 16 keep-alive requests per iteration, pooled connections so
handshakes are excluded — steady state only):

```bash
cargo bench --bench embeddings -- \
  --measurement-time 3 --warm-up-time 1 --sample-size 10 "transport"
```

| Metric | plain HTTP (`plain_http_batch_of_64`) | TLS (`tls_batch_of_64`) | Delta | SC-005 budget |
|--------|----------------------------------------|--------------------------|-------|----------------|
| median time per iteration | 8.73 ms | 9.20 ms | +5.4% | ≤ 10% ✓ |
| throughput | 117.2 Kelem/s | 111.4 Kelem/s | −5.0% | ≤ 10% ✓ |
| per-request p50 | 0.51 ms | 0.54 ms | +5.9% | ≤ 10% ✓ |
| per-request p99 | 1.14 ms | 1.13 ms | −0.9% | ≤ 10% ✓ |

TLS overhead stays within the 10% gate on every measured dimension; p99 is
statistically identical (TLS work is confined to the acceptor layer, the
request hot path is untouched).
