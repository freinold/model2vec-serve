# model2vec-serve

Lightweight OpenAI and Text Embedding Inference (TEI) compatible embeddings
server for [model2vec](https://github.com/MinishLab/model2vec) static embedding
models.

## Features

- OpenAI-compatible `POST /v1/embeddings` and `GET /v1/models`
- TEI-compatible `POST /embed` and `GET /info`, plus per-model
  `POST /tei/{model_id}/embed` and `GET /tei/{model_id}/info`
- Optional API key authentication
- Health (`/health`, `/ready`) and metrics (`/metrics`) endpoints
- Interactive OpenAPI documentation at `/docs`
- Structured JSON logs with request correlation IDs
- Small, containerized Rust binary
- One-command two-model local deployment via Docker Compose
- Helm chart for Kubernetes deployment with volume mount support

## Quickstart

Run locally with a Hugging Face model id:

```bash
cargo run --release -- --model minishlab/potion-multilingual-128M --port 8080
```

Request embeddings:

```bash
curl -X POST http://localhost:8080/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{"input":"Hello world"}'
```

Serve multiple models in one process:

```bash
cargo run --release -- \
  --model minishlab/potion-multilingual-128M \
  --model minishlab/potion-code-16M-v2 \
  --default-model minishlab/potion-multilingual-128M \
  --port 8080
```

Select a model in the request:

```bash
curl -X POST http://localhost:8080/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{"input":"def hello(): pass","model":"minishlab/potion-code-16M-v2"}'
```

## TEI per-model endpoints

TEI clients serving multiple models address a model explicitly through
`POST /tei/{model_id}/embed` and `GET /tei/{model_id}/info`. The `{model_id}`
path segment is the model's `--model-alias` or, by default, the last segment of
its identifier (e.g. `minishlab/potion-code-16M-v2` → `potion-code-16M-v2`).
Root `/embed` and `/info` continue to serve the default model.

**Breaking change in 0.5.0**: the `?model=` query parameter on `/embed` and
`/info` was removed. Migrate as follows:

| Before (≤ 0.3.x) | After (0.5.0) |
|------------------|----------------|
| `POST /embed?model=<id>` | `POST /tei/{model_id}/embed` |
| `GET /info?model=<id>` | `GET /tei/{model_id}/info` |
| `POST /embed` (default) | `POST /embed` (default, unchanged) |
| `?model=` present | `400 invalid_request` |

## Configuration

All configuration is passed as command-line arguments:

| Argument | Default | Description |
|----------|---------|-------------|
| `--model` | `minishlab/potion-multilingual-128M` | Hugging Face model id or local path; repeatable |
| `--default-model` | first `--model` | Model to use when a request does not specify one |
| `--model-owner` | `minishlab` | Model publisher or owner shown in `/v1/models` responses |
| `--model-alias` | none | Path identifier alias for a model, as `KEY=ALIAS`; repeatable |
| `--host` | `0.0.0.0` | Bind address |
| `--port` | `8080` | Listen port |
| `--api-key` | none | Enables Bearer token authentication |
| `--max-batch-size` | `256` | Maximum inputs per request |
| `--max-input-length` | `512` | Maximum tokens per input |
| `--log-level` | `info` | Log level |
| `--request-timeout-seconds` | `30` | Per-request timeout |

## Container

### Build locally

```bash
docker build -t model2vec-serve:latest .
docker run -p 8080:8080 -e MODEL=minishlab/potion-multilingual-128M model2vec-serve:latest
```

Serve multiple models via comma-separated `MODEL`:

```bash
docker run -p 8080:8080 \
  -e MODEL=minishlab/potion-multilingual-128M,minishlab/potion-code-16M-v2 \
  -e DEFAULT_MODEL=minishlab/potion-multilingual-128M \
  model2vec-serve:latest
```

### Pull from GitHub Container Registry

Released images are published to GHCR:

```bash
docker pull ghcr.io/freinold/model2vec-serve:v0.5.0
docker run -p 8080:8080 -e MODEL=minishlab/potion-multilingual-128M ghcr.io/freinold/model2vec-serve:v0.5.0
```

See [docs/deployment/docker.md](docs/deployment/docker.md) for the full release
and tagging strategy.

## Docker Compose

Run a local two-model stack (multilingual + code v2) with a persisted model
cache using the published image:

```bash
docker compose up -d
```

The stack serves `minishlab/potion-multilingual-128M` (default) and
`minishlab/potion-code-16M-v2`. Models download once into `./models` and
survive restarts. Customize via `.env` (see `.env.example`).

See [docs/deployment/compose.md](docs/deployment/compose.md) for the full
guide, including prerequisites, volume mounting, configuration, and
troubleshooting.

## Helm

The chart is published to the GitHub Container Registry:

```bash
helm install model2vec-serve \
  oci://ghcr.io/freinold/model2vec-serve/model2vec-serve \
  --version 0.2.0 \
  --set models[0]=minishlab/potion-multilingual-128M \
  --set apiKey=your-secret-key
```

Or install from a local checkout:

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set model=minishlab/potion-multilingual-128M \
  --set apiKey=your-secret-key
```

Install with multiple models:

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set models={minishlab/potion-multilingual-128M,minishlab/potion-code-16M-v2} \
  --set defaultModel=minishlab/potion-multilingual-128M \
  --set apiKey=your-secret-key
```

See [helm/model2vec-serve/README.md](helm/model2vec-serve/README.md) for more
options, including volume-mounted models.

## Development

Run the test suite:

```bash
cargo test
```

Run linting and formatting:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

Run benchmarks:

```bash
cargo bench
```
