# Implementation Plan: Multi-Model Serving

**Branch**: `003-multi-model-serving` | **Date**: 2026-07-19 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-multi-model-serving/spec.md`

## Summary

Extend the model2vec-serve HTTP service to load and serve multiple static embedding models in a single process. Clients select a model via the OpenAI-compatible `/v1/models` and `/v1/embeddings` endpoints, while TEI-compatible endpoints remain usable through a documented default-model or per-model selection strategy. The feature is validated with `minishlab/potion-multilingual-128M` and `minishlab/potion-code-16M-v2`.

## Technical Context

**Language/Version**: Rust 1.85 (MSRV), edition 2024.

**Primary Dependencies**: `axum`, `tokio`, `model2vec-rs`, `clap`, `tracing`, `tracing-subscriber`, `metrics`, `metrics-exporter-prometheus`, `utoipa`, `serde`, `thiserror`, `anyhow`. No new dependencies are expected; the registry can be built from the standard library and the existing stack.

**Storage**: N/A — all state is in memory. Models are loaded at startup and shared via application state.

**Testing**: `cargo test`, contract tests (`tests/*_contract.rs`), integration tests (`tests/*_integration.rs`), and config unit tests (`tests/config_unit.rs`).

**Target Platform**: Linux container and Kubernetes via the Helm chart.

**Project Type**: web-service / HTTP API.

**Performance Goals**: p99 latency < 20 ms for a single (batch-1) embedding request under light load; throughput ≥ 2,000 batch-1 requests/sec per model; peak RSS < 2 GB with both validation models loaded; cold-start model loading < 3 s from local disk.

**Constraints**:
- `unsafe_code = "forbid"` and `unwrap_used = "deny"`.
- Clippy pedantic enabled at warning level; CI treats warnings as errors.
- `missing_docs = "warn"` for all new public items.
- New dependencies must be justified (Simplicity Over Complexity principle).
- API changes that break existing clients require a MAJOR version bump.

**Scale/Scope**: Serve two or more static model2vec models in one process. The MVP validates the two-model case (multilingual and code) and keeps the configuration format backward-compatible for single-model deployments.

**Research Resolution**: All unknowns are resolved in `research.md`:
1. Validation model is `minishlab/potion-code-16M-v2`.
2. TEI strategy is default-model fallback with optional `model` query parameter.
3. Performance goals are p99 < 20 ms, ≥ 2,000 RPS per model, peak RSS < 2 GB, cold start < 3 s.
4. `model2vec-rs` supports multiple independent `StaticModel` instances in one process; instances are `Send + Sync` when wrapped in `Arc`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | Pass | Existing gates (`clippy`, `rustfmt`, `unwrap` deny) continue to apply. New registry code must be reviewed and documented. |
| II. Test Coverage | Pass with action | Every new route, config path, and error case must have tests that fail before and pass after. Contract tests must cover `/v1/models` and per-model `/v1/embeddings`. |
| III. API Conformity | Pass with action | Adding `/v1/models` and using the `model` field in `/v1/embeddings` is OpenAI-compatible. TEI changes must be documented; if a breaking change is chosen, the release version must be bumped. |
| IV. Simplicity Over Complexity | Pass with action | A model registry is the simplest in-process solution. Must document why multiple containers are insufficient (operator overhead, resource duplication). No unjustified new dependencies. |
| V. Performance Focus | Pass | Goals are defined in research.md and the Technical Context: p99 < 20 ms, ≥ 2,000 RPS per model, peak RSS < 2 GB, cold start < 3 s. Benchmarks must be run before release to confirm no regression > 10 %. |

### Post-Design Re-Check

After completing Phase 1 design artifacts (`data-model.md`, `contracts/`, `quickstart.md`):

- **I. Code Quality**: still Pass — no new dependencies introduced; registry is standard-library plus existing `Arc`/`HashMap` patterns.
- **II. Test Coverage**: still Pass with action — contract tests cover `/v1/models`, per-model `/v1/embeddings`, TEI default and `?model=` selection, and error cases for unknown models.
- **III. API Conformity**: still Pass with action — TEI default-model fallback is backward-compatible; the optional `model` query parameter is a documented extension. If the Helm chart removes the old single-model `model` value, a migration note is required.
- **IV. Simplicity Over Complexity**: still Pass with action — the registry abstraction and per-model labels are justified in the Complexity Tracking table.
- **V. Performance Focus**: still Pass — performance goals are concrete and measurable; benchmarks are required before release.

## Project Structure

### Documentation (this feature)

```text
specs/003-multi-model-serving/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── main.rs              # Entry point; loads config and starts server
├── lib.rs               # Library exports
├── config.rs            # CLI/env parsing; extended for multiple models and default model
├── state.rs             # AppState extended to hold a model registry
├── telemetry.rs         # Metrics and tracing; labels per model
├── auth.rs              # Unchanged API-key layer
├── errors.rs            # New error variant for unknown/unavailable model
├── model/
│   └── mod.rs           # Model wrapper / registry abstraction
└── routes/
    ├── mod.rs           # Router composition; adds /v1/models
    ├── dto.rs           # Request/response DTOs for multi-model
    ├── embeddings.rs    # OpenAI /v1/embeddings with model selection
    ├── tei.rs           # TEI /embed and /info with default or per-model routing
    ├── health.rs        # Readiness reflects all configured models
    └── metrics.rs       # Unchanged metrics endpoint

tests/
├── openai_contract.rs   # Tests /v1/models and /v1/embeddings
├── tei_contract.rs      # Tests /embed and /info behavior
├── multi_model_integration.rs # End-to-end validation with two models
├── config_unit.rs       # Config parsing for multi-model
└── common/mod.rs        # Shared test helpers

helm/model2vec-serve/
├── values.yaml          # Supports a list of models and a default model
├── README.md            # Updated deployment examples
└── templates/           # Deployment manifests
```

**Structure Decision**: The existing single-project layout is kept. The only structural additions are a `model` registry module and per-model labeling in telemetry. This is the minimal change that satisfies the spec while keeping the codebase maintainable.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Model registry abstraction | Multiple models must be addressable in one process; a registry is the smallest abstraction that supports OpenAI `/v1/models` and per-model routing. | Running one container per model is rejected because it duplicates base image/runtime overhead, requires separate Helm releases, and prevents a single OpenAI-compatible endpoint from listing all models. |
| Per-model telemetry labels | Operators need to distinguish load and latency per model; adding a `model` label to existing metrics is the minimal change. | Aggregated metrics only would hide per-model behavior and make incident response harder. |
