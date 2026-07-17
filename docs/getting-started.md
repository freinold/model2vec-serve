# Getting Started

This guide walks through running `model2vec-serve` locally, calling its
endpoints, and validating the container and Helm chart.

## Prerequisites

- Rust toolchain (stable, 1.85+)
- A model2vec model identifier (e.g. `minishlab/potion-multilingual-128M`) or a local model
  directory
- Docker (for container validation)
- Helm 3+ and a Kubernetes cluster or local environment such as kind/minikube
  (for Helm validation)

## Run locally

Start the server with a Hugging Face model id:

```bash
cargo run --release -- --model minishlab/potion-multilingual-128M --port 8080
```

You can also set values via environment variables:

```bash
MODEL=minishlab/potion-multilingual-128M PORT=8080 cargo run --release
```

## Verify health

```bash
curl http://localhost:8080/health
```

Expected response:

```json
{
  "status": "healthy",
  "ready": true,
  "message": "model loaded and serving requests"
}
```

## Request embeddings

### OpenAI-compatible endpoint

```bash
curl -X POST http://localhost:8080/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{"input":"Hello world"}'
```

Expected response shape:

```json
{
  "object": "list",
  "data": [
    {
      "object": "embedding",
      "index": 0,
      "embedding": [0.0123, -0.0456, "..."]
    }
  ],
  "model": "minishlab/potion-multilingual-128M",
  "usage": {
    "prompt_tokens": 2,
    "total_tokens": 2
  }
}
```

### TEI-compatible endpoint

```bash
curl -X POST http://localhost:8080/embed \
  -H "Content-Type: application/json" \
  -d '{"inputs":"Hello world"}'
```

Expected response:

```json
[[0.0123, -0.0456, "..."]]
```

## Inspect metrics

```bash
curl http://localhost:8080/metrics
```

The response is Prometheus-compatible text with request counters and latency
histograms.

## Enable API key authentication

Start the service with a key:

```bash
cargo run --release -- --model minishlab/potion-multilingual-128M --api-key secret-key
```

An unauthenticated request is rejected:

```bash
curl -X POST http://localhost:8080/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{"input":"Hello"}'
```

Expected: `401 Unauthorized`.

A request with the Bearer token succeeds:

```bash
curl -X POST http://localhost:8080/v1/embeddings \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer secret-key" \
  -d '{"input":"Hello"}'
```

## Run in Docker

Build the image:

```bash
docker build -t model2vec-serve:latest .
```

Run it:

```bash
docker run -p 8080:8080 \
  -e MODEL=minishlab/potion-multilingual-128M \
  model2vec-serve:latest
```

Run the same curl checks as in the local section.

## Deploy with Helm

Install the chart:

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set model=minishlab/potion-multilingual-128M \
  --set apiKey=secret-key \
  --set replicaCount=2
```

Wait for pods:

```bash
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=model2vec-serve
```

Port-forward and test:

```bash
kubectl port-forward svc/model2vec-serve 8080:80
curl http://localhost:8080/health
```

See the [Helm](./deployment/helm.md) page for volume-mounted models and all
configuration options.
