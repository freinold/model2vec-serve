# Implementation Plan: Default Model Update and Spec Drift Check

**Branch**: `002-default-model-multilingual` | **Date**: 2026-07-18 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/002-default-model-multilingual/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Change the service default model from `minishlab/potion-base-2M` to
`minishlab/potion-multilingual-128M` across the CLI, Helm chart, README,
AGENTS.md, and VitePress documentation. Keep test fixtures on a small, fast
model to preserve CI performance. Audit the existing specification and research
documents for drift against the current Hugging Face Hub integration and update
them if contradictions are found.

## Technical Context

**Language/Version**: Rust (edition 2024, stable toolchain 1.85+)

**Primary Dependencies**:
- `clap` — runtime argument parsing and default values
- `model2vec-rs` — static model loading and inference (unchanged)
- `hf-hub` — dev-dependency used only by test fixtures and benchmarks to download models

**Storage**: N/A

**Testing**: `cargo test` with existing contract, integration, and unit tests.

**Target Platform**: Linux container (`linux/amd64`, `linux/arm64`) deployed to Kubernetes via Helm.

**Project Type**: web-service / container

**Performance Goals**:
- No hot-path request changes; embedding latency remains unchanged for a given model.
- Startup time for the new default model may increase compared with the previous 2M default; acceptable because it is the production default.
- CI test suite must remain fast; test fixtures must not switch to the larger default model.

**Constraints**:
- Existing API contracts remain unchanged.
- No new runtime dependencies may be introduced.
- The change must not break deployments that explicitly set a model identifier or local path.
- All lint, format, and clippy gates must continue to pass.

**Scale/Scope**:
- Single default value change plus a documentation sweep.
- One-time audit of existing `specs/001-model2vec-embedding-api/research.md` for Hugging Face Hub drift.
- No new endpoints, no schema changes, no Helm template changes.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Gate | Status | Notes |
|-----------|------|--------|-------|
| Code Quality | Rust idioms, `clippy`, `rustfmt`, explicit error handling | PASS | Change is limited to default values and documentation; no new error paths. |
| Test Coverage | Contract tests for `/v1/embeddings` and TEI endpoints; 80% line coverage; 100% contract-path coverage | PASS | Existing tests continue to apply; add test that verifies the new CLI default. |
| API Conformity | OpenAPI docs via utoipa; response shapes validated against OpenAI/TEI specs; breaking changes trigger MAJOR bump | PASS | No API shape changes; default model identifier in responses changes only when the user does not supply one. |
| Simplicity Over Complexity | Minimal justified dependencies; no speculative abstractions | PASS | No new dependencies or abstractions; simplest possible default-value change. |
| Performance Focus | Benchmarks for embeddings endpoint; avoid blocking hot path; define latency/throughput budgets | PASS | No hot-path changes; test fixtures stay small to keep CI fast. |

No violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/002-default-model-multilingual/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command) - not required for this feature
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

The feature touches the following existing files only:

```text
src/
└── config.rs            # Add default_value to the --model argument

helm/model2vec-serve/
├── values.yaml          # Update default model value
└── README.md            # Update default model in examples and tables

README.md                # Update quick-start examples
AGENTS.md                # Update agent guide examples
docs/                    # Update VitePress docs (getting-started, configuration, deployment, api)
specs/001-model2vec-embedding-api/
├── research.md          # Audit and update HF Hub integration notes if drift is found
├── quickstart.md        # Update examples that use the old default
└── contracts/           # Update example model identifiers that imply a default
```

**Structure Decision**: Single Rust binary crate plus Helm chart, unchanged from
feature 001. This feature modifies existing configuration defaults and
documentation rather than adding new modules.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations to justify. The chosen approach changes a default value and
updates documentation without introducing new abstractions.

## Post-Design Constitution Check

*Re-check after Phase 1 design.*

| Principle | Gate | Status | Notes |
|-----------|------|--------|-------|
| Code Quality | Rust idioms, `clippy`, `rustfmt`, explicit error handling | PASS | Design does not introduce new code paths or error handling patterns. |
| Test Coverage | Contract tests for `/v1/embeddings` and TEI endpoints; 80% line coverage; 100% contract-path coverage | PASS | Existing tests cover unchanged contracts; a new config test will verify the default value. |
| API Conformity | OpenAPI docs via utoipa; response shapes validated against OpenAI/TEI specs; breaking changes trigger MAJOR bump | PASS | No endpoint or response-shape changes; `model_id` in responses will reflect the loaded model as before. |
| Simplicity Over Complexity | Minimal justified dependencies; no speculative abstractions | PASS | No new dependencies or abstractions. |
| Performance Focus | Benchmarks for embeddings endpoint; avoid blocking hot path; define latency/throughput budgets | PASS | Hot path unchanged; test fixtures remain small. |

No violations to justify after design.
