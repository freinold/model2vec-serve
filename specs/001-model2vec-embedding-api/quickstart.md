# Quickstart: model2vec-embedding-api

This guide validates the feature end-to-end after implementation.

## Prerequisites

- Rust toolchain (stable, 1.85+)
- Docker (for container validation)
- Helm 3+ and a Kubernetes cluster or local environment such as kind/minikube
- A model2vec model identifier or local model directory

## Local development validation

1. Build and run the service:

   ```bash
   cargo run -- --model minishlab/potion-base-32M --port 8080
   ```

2. Verify health:

   ```bash
   curl http://localhost:8080/health
   ```

   Expected: `{"status":"healthy","ready":true,...}`

3. Call the OpenAI-compatible endpoint:

   ```bash
   curl -X POST http://localhost:8080/v1/embeddings \
     -H "Content-Type: application/json" \
     -d '{"input":"Hello world","model":"minishlab/potion-base-32M"}'
   ```

   Expected: JSON list response with one embedding object.

4. Call the TEI-compatible endpoint:

   ```bash
   curl -X POST http://localhost:8080/embed \
     -H "Content-Type: application/json" \
     -d '{"inputs":"Hello world"}'
   ```

   Expected: JSON array containing one embedding vector.

5. Inspect metrics:

   ```bash
   curl http://localhost:8080/metrics
   ```

   Expected: Prometheus-style text with request counters and latency histograms.

## API key validation

1. Start the service with an API key:

   ```bash
   cargo run -- --model minishlab/potion-base-32M --api-key secret-key
   ```

2. Confirm unauthorized requests are rejected:

   ```bash
   curl -X POST http://localhost:8080/v1/embeddings \
     -H "Content-Type: application/json" \
     -d '{"input":"Hello"}'
   ```

   Expected: `401 Unauthorized`

3. Confirm authorized requests succeed:

   ```bash
   curl -X POST http://localhost:8080/v1/embeddings \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer secret-key" \
     -d '{"input":"Hello"}'
   ```

   Expected: `200 OK` with embeddings.

## Container validation

1. Build the image:

   ```bash
   docker build -t model2vec-serve:latest .
   ```

2. Run the container:

   ```bash
   docker run -p 8080:8080 \
     -e MODEL=minishlab/potion-base-32M \
     model2vec-serve:latest
   ```

3. Run the same curl checks as in the local section.

## Published image validation

After a release, the container image is published to GHCR and can be pulled
without building locally:

1. Pull the image for a released version:

   ```bash
   docker pull ghcr.io/freinold/model2vec-serve:v0.1.0
   ```

2. Run the published image:

   ```bash
   docker run -p 8080:8080 \
     -e MODEL=minishlab/potion-base-32M \
     ghcr.io/freinold/model2vec-serve:v0.1.0
   ```

3. Verify health and call the OpenAI-compatible endpoint as in the local section.

## Helm validation

1. Install the chart with a model and optional API key:

   ```bash
   helm install model2vec-serve ./helm/model2vec-serve \
     --set model=minishlab/potion-base-32M \
     --set apiKey=secret-key \
     --set replicaCount=2
   ```

2. Wait for pods to become ready:

   ```bash
   kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=model2vec-serve
   ```

3. Port-forward and test:

   ```bash
   kubectl port-forward svc/model2vec-serve 8080:80
   curl http://localhost:8080/health
   curl -X POST http://localhost:8080/v1/embeddings \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer secret-key" \
     -d '{"input":"Hello from Kubernetes"}'
   ```

## Volume-mounted model validation

1. Download or copy a model to a local directory.
2. Install the chart with an extra volume mount:

   ```bash
   helm install model2vec-serve ./helm/model2vec-serve \
     --set model=/models/my-model \
     --set extraVolumes[0].name=model-volume \
     --set extraVolumes[0].hostPath.path=/path/to/local/model \
     --set extraVolumeMounts[0].name=model-volume \
     --set extraVolumeMounts[0].mountPath=/models/my-model
   ```

3. Verify the service loads the model from the mounted path and responds to
   health checks.

## Contract test validation

Run the contract tests to confirm OpenAI and TEI compatibility:

```bash
cargo test --test contract
```

Expected: all tests pass, including response-shape assertions for `/v1/embeddings`
and `/embed`.
