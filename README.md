# model2vec-serve

Lightweight OpenAI and Text Embedding Inference (TEI) compatible embeddings
server for [model2vec](https://github.com/MinishLab/model2vec) static embedding
models.

## Features

- OpenAI-compatible `POST /v1/embeddings` and `GET /v1/models`
- TEI-compatible `POST /embed` and `GET /info`
- Optional API key authentication
- Health (`/health`, `/ready`) and metrics (`/metrics`) endpoints
- Interactive OpenAPI documentation at `/docs`
- Structured JSON logs with request correlation IDs
- Small, containerized Rust binary
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

## Configuration

All configuration is passed as command-line arguments:

| Argument | Default | Description |
|----------|---------|-------------|
| `--model` | `minishlab/potion-multilingual-128M` | Hugging Face model id or local path; repeatable |
| `--default-model` | first `--model` | Model to use when a request does not specify one |
| `--model-owner` | `minishlab` | Model publisher or owner shown in `/v1/models` responses |
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
docker pull ghcr.io/freinold/model2vec-serve:v0.1.0
docker run -p 8080:8080 -e MODEL=minishlab/potion-multilingual-128M ghcr.io/freinold/model2vec-serve:v0.1.0
```

See [docs/deployment/docker.md](docs/deployment/docker.md) for the full release
and tagging strategy.

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
