# model2vec-serve — Agent Guide

This file provides project-specific guidance for AI agents working on the
`model2vec-serve` repository.

## Project Overview

`model2vec-serve` is a lightweight Rust (axum) HTTP service that serves static
[model2vec](https://github.com/MinishLab/model2vec) embedding models through
OpenAI-compatible and Text Embedding Inference (TEI) compatible endpoints. It is
intended to run as a small container in Kubernetes (Helm) or locally for
development.

Key capabilities:

- `POST /v1/embeddings` — OpenAI-compatible embeddings.
- `POST /embed` and `GET /info` — TEI-compatible endpoints.
- Optional `Authorization: Bearer` API-key authentication.
- `GET /health`, `GET /ready`, `GET /metrics` — operational endpoints.
- Interactive OpenAPI/Scalar docs at `/docs`.
- Structured JSON logs with request-correlation IDs.
- Multi-stage Dockerfile and Helm chart.

## Repository Layout

```
.
├── src/                            # Rust source
│   ├── main.rs                     # Binary entry point
│   ├── lib.rs                      # Library exports
│   ├── config.rs                   # CLI / env configuration (clap)
│   ├── state.rs                    # Application state (model, metrics, config)
│   ├── telemetry.rs                # Tracing + Prometheus metrics middleware
│   ├── auth.rs                     # Bearer-token auth layer
│   ├── errors.rs                   # AppError + error responses
│   ├── model/                      # Model loading / embedding wrapper
│   └── routes/                     # Axum handlers and DTOs
│       ├── mod.rs                  # Router composition
│       ├── dto.rs                  # Request/response types
│       ├── embeddings.rs           # OpenAI /v1/embeddings
│       ├── tei.rs                  # TEI /embed and /info
│       ├── health.rs               # /health and /ready
│       └── metrics.rs              # /metrics
├── specs/                          # Feature specifications and contracts
│   └── 001-model2vec-embedding-api/
│       ├── spec.md                 # Requirements and user stories
│       ├── quickstart.md           # End-to-end validation guide
│       ├── data-model.md           # Entity/field tables
│       ├── research.md             # Architecture decisions
│       └── contracts/              # API endpoint contracts
├── helm/model2vec-serve/           # Helm chart
│   ├── values.yaml                 # Default chart values
│   ├── README.md                   # Chart usage documentation
│   └── templates/                  # Kubernetes manifests
├── tests/                          # Test suites
│   ├── *_contract.rs               # API-shape contract tests
│   ├── *_integration.rs            # End-to-end integration tests
│   ├── config_unit.rs              # Configuration unit tests
│   ├── common/mod.rs               # Shared test helpers
│   └── helm/                       # Helm lint/template scripts
├── benches/                        # Criterion benchmarks
├── docs/                           # VitePress documentation site
├── Dockerfile
├── Cargo.toml
└── rustfmt.toml
```

## Build, Test & Lint Commands

All commands run from the repository root.

```bash
# Build
cargo build --release

# Run the full test suite
cargo test

# Check formatting
cargo fmt -- --check

# Run clippy (zero warnings allowed in CI)
cargo clippy --all-targets --all-features -- -D warnings

# Run benchmarks
cargo bench

# Run the service locally
cargo run --release -- --model minishlab/potion-multilingual-128M --port 8080
```

## Code Conventions

- **Rust edition**: 2024; MSRV is 1.85.
- `unsafe_code = "forbid"` — never introduce `unsafe` blocks.
- `unwrap_used = "deny"` — prefer `?`, `Result`, `Option` combinators, or
  `anyhow::Context`. Only `expect` when the invariant is truly obvious and
  document why with a comment.
- `missing_docs = "warn"` — add doc comments for new public items.
- Clippy `pedantic` is enabled at warning level; keep warnings clean.
- Follow the existing module structure:
  - handlers live in `src/routes/`;
  - shared DTOs live in `src/routes/dto.rs`;
  - errors live in `src/errors.rs`;
  - cross-cutting concerns (auth, telemetry) live at `src/` root.
- Use `utoipa` path/response annotations to keep `/docs` in sync with code.
- Keep handlers thin; validation logic should be extracted into small, testable
  functions.

## Architecture

### Startup flow

1. `main.rs` parses `Config` from CLI args / environment variables via `clap`.
2. `telemetry::init_tracing` configures JSON logging.
3. `telemetry::init_metrics` installs the Prometheus recorder.
4. `AppState::new` loads the model2vec model from Hugging Face Hub or a local
   path.
5. The axum listener binds to `host:port` and serves `app(state)`.

### Router composition

`src/routes/mod.rs` assembles the router:

- `/v1/embeddings`, `/embed`, `/info` are grouped and protected by the optional
  API-key middleware when `--api-key` / `API_KEY` is set.
- `/health`, `/ready`, `/metrics` are unprotected.
- Scalar OpenAPI UI is mounted at `/docs`.
- Tower middleware stack: trace → timeout → CORS → compression → request-tracing.

### Model inference

- The `model2vec-rs` crate loads the static model.
- `state.model.encode(&inputs, max_input_length, batch_size)` returns
  `Vec<Vec<f32>>`.
- The model is loaded once at startup and shared via `Arc<AppState>`.

### Authentication

- Implemented as an axum middleware layer in `src/auth.rs`.
- When enabled, only embedding endpoints require a valid `Authorization: Bearer
  <key>` header.
- Health, readiness, and metrics endpoints remain public.

### Observability

- `tracing` + `tracing-subscriber` emit structured JSON logs.
- Each request gets a correlation ID, either from the incoming `x-request-id`
  header or a generated UUID, and the ID is returned in the response headers.
- `metrics` + `metrics-exporter-prometheus` expose:
  - `http_requests_total` counter by method/path/status;
  - `http_request_duration_seconds` histogram by method/path;
  - `http_errors_total` counter for 5xx responses.

## API Compatibility Rules

When modifying endpoints, preserve the documented contracts:

- **OpenAI**: request body with `input`/`model`/`encoding_format`; response
  shape `{ object, data: [{ object, index, embedding }], model, usage }`.
- **TEI embed**: request body with `inputs`; response is a JSON array of float
  arrays in the same order as inputs.
- **TEI info**: response shape `{ model_id, max_input_length,
  embedding_dimension, pooling }`.
- **Errors**: always return JSON `{ error: "code", message: "..." }` using the
  codes documented in `specs/001-model2vec-embedding-api/contracts/errors.md`.

Always update the corresponding contract tests (`tests/*_contract.rs`) and the
VitePress docs in `docs/` when endpoint behavior changes.

## Error Handling

- `AppError` in `src/errors.rs` maps domain errors to HTTP status codes and the
  standard `ErrorResponse` body.
- Use `thiserror` for typed error variants.
- Use `anyhow` for ad-hoc errors, mainly in startup code.
- Error messages must be safe to expose to API clients: no internal paths,
  secrets, or stack traces in response bodies.

## Deployment Artifacts

### Docker

- Multi-stage build: `rust:1.85-slim` builder → `debian:bookworm-slim` runtime.
- Final image exposes port `8080` and runs `model2vec-serve`.
- Build: `docker build -t model2vec-serve:latest .`
- Run: `docker run -p 8080:8080 -e MODEL=minishlab/potion-multilingual-128M model2vec-serve:latest`

### Helm

- Chart location: `helm/model2vec-serve/`.
- Key values: `model`, `apiKey`, `replicaCount`, `image.tag`, `resources`,
  `autoscaling.enabled`, `extraVolumes`, `extraVolumeMounts`.
- Install: `helm install model2vec-serve ./helm/model2vec-serve --set model=...`
- Volume-mounted models are supported via `extraVolumes` / `extraVolumeMounts`.

When chart values or templates change, update both `helm/model2vec-serve/README.md`
and the VitePress docs page `docs/deployment/helm.md`.

## Spec & Documentation Workflow

- Behavioral requirements and API contracts live under
  `specs/001-model2vec-embedding-api/`.
- Treat the spec as the source of truth for what the service must do.
- The VitePress docs site in `docs/` is generated from the spec, contracts,
  README, and code. Keep it in sync when:
  - adding or changing CLI arguments (`src/config.rs`);
  - adding or changing endpoints (`src/routes/`);
  - changing error responses (`src/errors.rs`, `contracts/errors.md`);
  - changing deployment packaging (`Dockerfile`, Helm chart);
  - updating the README.
- The docs site is built and deployed to GitHub Pages automatically by
  `.github/workflows/docs.yml`.

## External References

Read these files when relevant to the task:

- `README.md` — project summary and quick commands.
- `specs/001-model2vec-embedding-api/spec.md` — requirements and user stories.
- `specs/001-model2vec-embedding-api/quickstart.md` — end-to-end validation.
- `specs/001-model2vec-embedding-api/contracts/*.md` — API endpoint contracts.
- `specs/001-model2vec-embedding-api/data-model.md` — entity/field tables.
- `specs/001-model2vec-embedding-api/research.md` — architecture decisions.
- `helm/model2vec-serve/README.md` and `values.yaml` — chart documentation.
