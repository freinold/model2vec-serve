# Introduction

`model2vec-serve` is a small, fast Rust service that exposes static
[model2vec](https://github.com/MinishLab/model2vec) embedding models through
both OpenAI-compatible and Text Embedding Inference (TEI) compatible HTTP
endpoints.

It is designed for teams that want a self-hosted embeddings API without the
overhead of a large inference framework.

## Why model2vec-serve?

- **Drop-in compatibility**: Existing OpenAI or TEI clients can point at this
  service with only a base-URL (and optional API key) change.
- **Lightweight**: Static model2vec models are tiny and run efficiently on CPU,
  so the container starts quickly and runs with modest resources.
- **Observable**: Structured JSON logs with a per-request correlation ID and a
  Prometheus `/metrics` endpoint come out of the box.
- **Kubernetes friendly**: A Helm chart is included for repeatable cluster
  deployments, including support for volume-mounted models.

## Supported endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/v1/embeddings` | `POST` | OpenAI-compatible embeddings |
| `/embed` | `POST` | TEI-compatible embeddings |
| `/info` | `GET` | TEI-compatible model information |
| `/health` | `GET` | Health / readiness probe |
| `/ready` | `GET` | Kubernetes readiness alias |
| `/metrics` | `GET` | Prometheus metrics |
| `/docs` | `GET` | Interactive OpenAPI (Scalar) UI |

## Typical use cases

- Replace OpenAI embeddings calls in an existing application with a private,
  on-premise model.
- Run a local embeddings API for development, evaluation, or CI pipelines.
- Deploy a small, stateless embeddings service into Kubernetes behind a load
  balancer.

## What’s next?

- [Getting Started](./getting-started.md) — run the server locally.
- [Configuration](./configuration.md) — CLI arguments and environment variables.
- [Architecture](./architecture.md) — how the service is built.
- [API Reference](./api/openai.md) — request/response contracts.
