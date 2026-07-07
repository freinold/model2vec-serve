# Architecture

This page explains how `model2vec-serve` is structured, why those choices were
made, and how the pieces fit together.

## High-level goals

The project aims to be a small, self-contained embeddings server:

1. Load a static model2vec model once at startup.
2. Expose OpenAI- and TEI-compatible HTTP endpoints.
3. Remain observable and operable with minimal configuration.
4. Package cleanly as a container and Helm chart.

## Technology choices

### Model inference: `model2vec-rs`

The official Rust implementation from MinishLab is used for static embedding
inference. It loads models from Hugging Face Hub, local paths, or raw bytes and
exposes a simple `encode(sentences)` API returning `Vec<Vec<f32>>`.

Rationale:

- Optimized for CPU inference with `safetensors` weights.
- Supports f32, f16, and i8 weight types and batch encoding.
- ~1.7× faster than the Python implementation in upstream benchmarks.
- Keeps the container small and free of a Python runtime.

For air-gapped deployments that only load mounted local models, the dependency
can be built with `--no-default-features --features onig,local-only` to avoid
network dependencies.

### Web framework: axum + utoipa

- **axum** provides typed extractors, composable routers, and clean integration
  with Tower middleware.
- **utoipa** derives an OpenAPI spec from annotated handlers and `ToSchema`
  types, which keeps the interactive `/docs` UI in sync with the code.

This combination keeps request handlers thin and the API contract explicit.

### Configuration: clap

All runtime configuration is parsed as CLI arguments via `clap`. Each argument
has an environment-variable equivalent, which makes mapping to Kubernetes
containers and secrets straightforward.

### Observability: tracing + metrics

- **tracing** + **tracing-subscriber** produce structured JSON logs.
- A per-request correlation ID is either taken from the incoming `x-request-id`
  header or generated as a UUID and returned in the response headers.
- **metrics** + **metrics-exporter-prometheus** expose request counts, latency
  histograms, and 5xx error counters at `/metrics`.

No external observability agent is required inside the container.

## Startup flow

```
main.rs
  ├── Config::parse()          # CLI / env configuration
  ├── telemetry::init_tracing  # JSON logging
  ├── telemetry::init_metrics  # Prometheus recorder
  ├── AppState::new            # load model2vec model
  └── axum::serve              # bind TCP and serve requests
```

The model is loaded synchronously at startup. If loading fails, the process
exits. Once `AppState` is ready, all request handlers share it through an
`Arc<AppState>`.

## Router composition

`src/routes/mod.rs` builds the axum router:

1. Create a router for `/v1/embeddings`, `/embed`, and `/info`.
2. If `api_key` is configured, wrap those routes in the Bearer-token auth layer.
3. Add public `/health`, `/ready`, and `/metrics` routes.
4. Mount the Scalar OpenAPI UI at `/docs`.
5. Apply global Tower middleware in this order:
   - `TraceLayer`
   - `TimeoutLayer`
   - `CorsLayer` (permissive)
   - `CompressionLayer`
   - custom request-tracing middleware that records metrics and injects
     `x-request-id`

Authentication is therefore applied only to embedding endpoints; operational
endpoints remain public.

## Request lifecycle

1. Middleware extracts or generates a request ID.
2. The request is routed to the appropriate handler.
3. The handler validates input, calls `state.model.encode(...)`, and builds the
   response.
4. Telemetry middleware records method, path, status, and latency to Prometheus
   counters/histograms.
5. The response is returned with the `x-request-id` header attached.

## Error handling

All handlers return `Result<T, AppError>`. `AppError` (defined in
`src/errors.rs`) maps domain errors to HTTP status codes and the standard
`ErrorResponse` JSON body:

```json
{
  "error": "invalid_request",
  "message": "..."
}
```

Error messages are safe to expose to API clients: they never include internal
paths, secrets, or stack traces.

## Deployment packaging

- **Docker**: multi-stage build from `rust:1.85-slim` to
  `debian:bookworm-slim`. The final image is small, exposes port `8080`, and
  runs `model2vec-serve` as the entry point.
- **Helm**: chart under `helm/model2vec-serve/` with `values.yaml` for model
  source, API key, resources, autoscaling, and volume mounts. Volume-mounted
  local models are supported through `extraVolumes` and `extraVolumeMounts`.

## Design decisions

Key architecture decisions are recorded in
`specs/001-model2vec-embedding-api/research.md`, including the rationale for:

- choosing `model2vec-rs` over candle, ONNX Runtime, or shelling out to Python;
- choosing axum + utoipa;
- using CLI-first configuration mirrored in Helm values;
- using tracing + Prometheus for observability;
- using a Tower layer for API-key authentication.
