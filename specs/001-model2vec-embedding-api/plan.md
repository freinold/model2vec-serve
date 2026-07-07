# Implementation Plan: model2vec-embedding-api

**Branch**: `001-model2vec-embedding-api` | **Date**: 2026-07-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-model2vec-embedding-api/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Build a lightweight, containerized Rust web service that serves static model2vec
embeddings through OpenAI-compatible and Text-Embedding-Inference-compatible
endpoints. The service will use axum for the HTTP layer, utoipa for OpenAPI
documentation, tracing/metrics for observability, API-key middleware for access
control, and a Helm chart for Kubernetes deployment. Configuration is passed as
runtime arguments and mirrored into Helm `values.yaml`, including volume mounts
for cached or air-gapped models.

## Technical Context

**Language/Version**: Rust (edition 2024, stable toolchain 1.85+)

**Primary Dependencies**:
- `axum` — HTTP router and handlers
- `utoipa` + `utoipa-scalar` — OpenAPI spec and interactive docs
- `tokio` — async runtime
- `serde` / `serde_json` — serialization
- `tracing` / `tracing-subscriber` — structured logs and request correlation
- `metrics` / `metrics-exporter-prometheus` — Prometheus-compatible metrics
- `clap` — runtime argument parsing
- `tower-http` — CORS, timeout, trace, compression layers
- `model2vec-rs` crate from crates.io — official static model2vec inference (see research.md)

**Storage**: N/A for request state; model weights are loaded into memory at
startup. Local model files may be mounted into the container via Helm volumes.

**Testing**: `cargo test` with unit, contract, and integration tests. Contract
 tests validate OpenAI and TEI response shapes against the OpenAPI spec.

**CI/CD**: GitHub Actions workflows build and push the container image to GHCR on
 every GitHub release, and `release-plz` automates SemVer releases from
 Conventional Commits. See `research.md` for workflow details.


**Target Platform**: Linux container (`linux/amd64`, `linux/arm64`) deployed to
Kubernetes via Helm.

**Project Type**: web-service / container

**Performance Goals**:
- p99 latency for a single string embedding: < 50 ms on a 1 vCPU container.
- Throughput: > 500 embeddings/second per replica for small batches.
- Startup time: model loaded and health endpoint ready within 60 seconds.
- Container memory: < 1.5 GiB peak for a 128-dimensional model.

**Constraints**:
- No GPU required; inference runs on CPU.
- All configuration must be expressible as CLI args and mapped to Helm values.
- API responses must remain compatible with OpenAI and TEI clients.
- Static models only; no training or fine-tuning at runtime.
- Secrets (API keys) must be configurable, never hard-coded.

**Scale/Scope**:
- Stateless replicas; scale horizontally via Helm `replicaCount`.
- Single-model deployment per replica; model swap requires a new deployment.
- No request persistence or queueing beyond in-flight HTTP requests.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Gate | Status | Notes |
|-----------|------|--------|-------|
| Code Quality | Rust idioms, `clippy`, `rustfmt`, explicit error handling | PASS | Project will deny warnings in CI; no `unwrap` outside tests. |
| Test Coverage | Contract tests for `/v1/embeddings` and TEI endpoints; 80% line coverage; 100% contract-path coverage | PASS | Contract tests and integration tests will be included for every endpoint. |
| API Conformity | OpenAPI docs via utoipa; response shapes validated against OpenAI/TEI specs; breaking changes trigger MAJOR bump | PASS | utoipa derives will keep `/docs` in sync with implementation. |
| Simplicity Over Complexity | Minimal justified dependencies; no speculative abstractions | PASS | Stack is axum + utoipa + model inference crate + observability; no database or message queue. |
| Performance Focus | Benchmarks for embeddings endpoint; avoid blocking hot path; define latency/throughput budgets | PASS | Performance goals are defined above; benchmarks will be committed. |

No violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/001-model2vec-embedding-api/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
Cargo.toml
Cargo.lock
Dockerfile
.github/
└── workflows/
    ├── docker.yml       # Build and push image to GHCR on releases
    ├── release.yml      # release-plz semantic-release automation
    └── docs.yml         # VitePress documentation deployment
src/
├── main.rs              # CLI parsing, startup, graceful shutdown
├── config.rs            # Runtime configuration struct and validation
├── state.rs             # Shared application state (model handle, config)
├── errors.rs            # Structured AppError and IntoResponse mapping
├── telemetry.rs         # Tracing subscriber, request IDs, metrics recorder
├── routes/
│   ├── mod.rs           # Router composition
│   ├── embeddings.rs    # OpenAI-compatible /v1/embeddings
│   ├── tei.rs           # TEI-compatible /embed and /info endpoints
│   ├── health.rs        # /health and /ready
│   └── metrics.rs       # /metrics Prometheus scrape endpoint
├── model/
│   └── embedding.rs     # model2vec model loading and inference wrapper
└── auth.rs              # API key middleware

tests/
├── contract/
│   ├── openai.rs
│   └── tei.rs
├── integration/
│   ├── health.rs
│   ├── metrics.rs
│   └── auth.rs
└── unit/
    ├── config.rs
    └── embeddings.rs

helm/
└── model2vec-serve/
    ├── Chart.yaml
    ├── values.yaml
    ├── README.md
    └── templates/
        ├── _helpers.tpl
        ├── deployment.yaml
        ├── service.yaml
        ├── configmap.yaml
        ├── secret.yaml
        ├── hpa.yaml
        └── NOTES.txt
```

**Structure Decision**: Single Rust binary crate plus a Helm chart. The binary
is organized by vertical slice (routes, model, auth, telemetry) to keep each
module focused. Tests mirror the route structure for clear ownership. The Helm
chart is co-located under `helm/` so packaging and application code move
together.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations to justify. The chosen stack directly maps to the feature
requirements without speculative abstraction.
