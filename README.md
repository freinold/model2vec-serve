# model2vec-serve

Lightweight OpenAI and Text Embedding Inference (TEI) compatible embeddings
server for [model2vec](https://github.com/MinishLab/model2vec) static embedding
models.

## Features

- OpenAI-compatible `POST /v1/embeddings`
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
cargo run --release -- --model minishlab/potion-base-2M --port 8080
```

Request embeddings:

```bash
curl -X POST http://localhost:8080/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{"input":"Hello world"}'
```

## Configuration

All configuration is passed as command-line arguments:

| Argument | Default | Description |
|----------|---------|-------------|
| `--model` | required | Hugging Face model id or local path |
| `--host` | `0.0.0.0` | Bind address |
| `--port` | `8080` | Listen port |
| `--api-key` | none | Enables Bearer token authentication |
| `--max-batch-size` | `256` | Maximum inputs per request |
| `--max-input-length` | `512` | Maximum tokens per input |
| `--log-level` | `info` | Log level |
| `--request-timeout-seconds` | `30` | Per-request timeout |

## Container

```bash
docker build -t model2vec-serve:latest .
docker run -p 8080:8080 -e MODEL=minishlab/potion-base-2M model2vec-serve:latest
```

## Helm

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set model=minishlab/potion-base-2M \
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
