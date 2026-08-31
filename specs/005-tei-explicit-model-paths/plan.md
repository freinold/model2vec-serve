# Implementation Plan: TEI-Explicit Per-Model Endpoints

**Branch**: `005-tei-explicit-model-paths` | **Date**: 2026-08-29 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/005-tei-explicit-model-paths/spec.md`

## Summary

Serve every loaded model behind explicit TEI-compatible paths
(`/tei/{model_id}/embed` and `/tei/{model_id}/info`) so TEI clients select a
model by base URL alone. The path identifier is an operator-configured alias
or, when absent, the model identifier's last segment; startup fails with an
operator-facing error when two models share the same identifier. The hidden
`?model=` qualifier on `/embed` and `/info` is retired: requests carrying it
receive an explicit 400 error pointing to the per-model paths, and unknown
models in a per-model path return 404. This is a breaking change shipped as
release 0.5.0.

## Technical Context

**Language/Version**: Rust 2024 edition, MSRV 1.85 (Cargo.toml)

**Primary Dependencies**: axum 0.8, model2vec-rs 0.2, clap 4 (derive + env),
utoipa 5 + utoipa-scalar 0.3, tracing/tracing-subscriber,
metrics-exporter-prometheus, tower-http (trace/timeout/CORS/compression)

**Storage**: N/A (in-memory model registry; models loaded from Hugging Face
Hub or local paths at startup)

**Testing**: cargo test (contract + integration suites in `tests/`, shared
helpers in `tests/common/`), cargo clippy `-D warnings`, cargo fmt --check

**Target Platform**: Linux server (Docker container, Debian bookworm-slim
runtime; Helm chart for Kubernetes)

**Project Type**: web-service (single Rust binary, axum HTTP server)

**Performance Goals**: Per-model endpoints within 10% latency/throughput of
the existing `/embed` endpoint under equivalent load (SC-004); same
single-threaded-per-request inference path, no added allocations on the hot
path beyond one path-segment lookup

**Constraints**: No new dependencies; path identifiers are single URL
segments (slash-free); zero-warnings clippy pedantic; `unsafe` forbidden;
`unwrap` denied

**Scale/Scope**: Small registry (handful of models per process); 2 new HTTP
endpoints, 1 new CLI flag, registry + telemetry touch points, contract/
integration test updates, docs (VitePress + README) and version bump to
0.5.0

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | PASS | No new deps; explicit error handling for all new failure paths (unknown path model, retired qualifier, startup conflicts); doc comments on all new public items |
| II. Test Coverage | PASS | Contract tests for both per-model endpoints, integration tests for alias/last-segment resolution, 404, retired-qualifier rejection (root + per-model), startup conflict error, auth on `/tei` paths; OpenAI contract tests untouched and still passing |
| III. API Conformity | PASS | Per-model responses reuse the TEI shapes (`Vec<Vec<f32>>`, `ModelInfo`); breaking change (qualifier removal) documented with migration note and ships as major release 0.5.0; `/docs` regenerated via utoipa path annotations; error codes follow errors.md |
| IV. Simplicity Over Complexity | PASS | Alias map is a flat `Vec<(String, String)>` parsed by clap; no new abstractions beyond one registry index; rejected alternatives recorded below and in research.md |
| V. Performance Focus | PASS | No locks/copies added on hot path; resolution is one HashMap lookup by precomputed path identifier; benchmarks (`benches/`) extended to cover per-model route; 10% regression threshold enforced by SC-004 |

**Post-Phase-1 re-check**: PASS — design adds one registry lookup structure
and one error variant; no gate violations.

## Project Structure

### Documentation (this feature)

```text
specs/005-tei-explicit-model-paths/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── tei-per-model.md
│   └── errors.md
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── config.rs                # + --model-alias flag (MODEL_ALIAS, repeatable KEY=ALIAS)
├── errors.rs                # + AppError variant mapping unknown path models to 404 not_found
├── model/mod.rs             # + path identifier derivation, alias map, startup conflict validation, lookup by path id
├── routes/mod.rs            # + /tei/{model_id}/embed and /tei/{model_id}/info routes (inside auth group), utoipa paths
├── routes/tei.rs            # − TeiModelQuery (retired qualifier); + per-model handlers reusing validation/encoding
└── telemetry.rs             # unchanged (RequestModelId attribution reused)

tests/
├── tei_contract.rs          # updated: per-model endpoints, retired-qualifier 400, 404 unknown path model
├── tei_integration.rs       # updated: root endpoints qualifier-free default behavior
├── multi_model_integration.rs # updated: per-model isolation across ≥3 models, alias + last-segment resolution
├── auth_integration.rs      # updated: /tei paths require Bearer when API key set
└── config_unit.rs           # updated: MODEL_ALIAS parsing, duplicate path-id startup error
```

**Structure Decision**: Existing single-crate web-service layout is extended
in place; no new modules beyond adding per-model handlers to
`src/routes/tei.rs` and registry support in `src/model/mod.rs`.

## Complexity Tracking

> No constitution gate violations. Rejected simpler alternatives are recorded
> to satisfy Principle IV:

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | — | Full hierarchical id in path (`minishlab%2F...`): rejected — depends on `%2F` surviving reverse proxies, clunky client URLs. Short-name-only without overrides: rejected — silent collisions across namespaces. Request-time ambiguity errors: rejected — fail-fast at startup surfaces operator mistakes before traffic (see research.md D1/D2). |
