# Quickstart: Multi-Model Serving

This guide validates the multi-model feature end-to-end after implementation.

## Prerequisites

- Rust toolchain (stable, 1.85+)
- Docker (for container validation)
- Helm 3+ and a Kubernetes cluster or local environment such as kind/minikube
- Two model2vec model identifiers: `minishlab/potion-multilingual-128M` and `minishlab/potion-code-16M-v2`

## Local development validation

1. Build and run the service with two models and a default:

   ```bash
   cargo run --release -- \
     --model minishlab/potion-multilingual-128M \
     --model minishlab/potion-code-16M-v2 \
     --default-model minishlab/potion-multilingual-128M \
     --port 8080
   ```

   Expected: both models load and the readiness endpoint returns success.

2. Verify health shows both models:

   ```bash
   curl http://localhost:8080/health
   ```

   Expected: `{"status":"healthy","ready":true,"models":[{"model_id":"...","status":"ready"},...]}`

3. List available models via the OpenAI-compatible endpoint:

   ```bash
   curl http://localhost:8080/v1/models
   ```

   Expected: JSON list containing both model identifiers.

4. Request embeddings from the multilingual model:

   ```bash
   curl -X POST http://localhost:8080/v1/embeddings \
     -H "Content-Type: application/json" \
     -d '{"input":"Hello world","model":"minishlab/potion-multilingual-128M"}'
   ```

   Expected: JSON list response with one embedding object and `model` set to the multilingual identifier.

5. Request embeddings from the code model:

   ```bash
   curl -X POST http://localhost:8080/v1/embeddings \
     -H "Content-Type: application/json" \
     -d '{"input":"def hello(): pass","model":"minishlab/potion-code-16M-v2"}'
   ```

   Expected: JSON list response with one embedding object and `model` set to the code identifier.

6. Omit the model and verify the default is used:

   ```bash
   curl -X POST http://localhost:8080/v1/embeddings \
     -H "Content-Type: application/json" \
     -d '{"input":"Hello world"}'
   ```

   Expected: Response `model` field equals the configured default model.

7. Request an unknown model and verify the error:

   ```bash
   curl -X POST http://localhost:8080/v1/embeddings \
     -H "Content-Type: application/json" \
     -d '{"input":"Hello","model":"minishlab/unknown-model"}'
   ```

   Expected: `400 Bad Request` with `{"error":"model_not_found",...}`.

8. Call the TEI-compatible endpoint without a model selector:

   ```bash
   curl -X POST http://localhost:8080/embed \
     -H "Content-Type: application/json" \
     -d '{"inputs":"Hello world"}'
   ```

   Expected: JSON array containing one embedding vector from the default model.

9. Call the TEI-compatible endpoint with a model selector:

   ```bash
   curl -X POST 'http://localhost:8080/embed?model=minishlab/potion-code-16M-v2' \
     -H "Content-Type: application/json" \
     -d '{"inputs":"def hello(): pass"}'
   ```

   Expected: JSON array containing one embedding vector from the code model.

10. Inspect metrics:

    ```bash
    curl http://localhost:8080/metrics
    ```

    Expected: Prometheus-style text with request counters and latency histograms labeled by `model`.

## Single-model backward-compatibility validation

1. Start the service with a single model:

   ```bash
   cargo run --release -- --model minishlab/potion-multilingual-128M --port 8080
   ```

2. Verify `/v1/models` returns one model, `/v1/embeddings` without a model uses it, and TEI `/embed` and `/info` behave as before.

## Container validation

1. Build the image:

   ```bash
   docker build -t model2vec-serve:latest .
   ```

2. Run the container with two models:

   ```bash
   docker run -p 8080:8080 \
     -e MODEL=minishlab/potion-multilingual-128M,minishlab/potion-code-16M-v2 \
     -e DEFAULT_MODEL=minishlab/potion-multilingual-128M \
     model2vec-serve:latest
   ```

3. Run the same curl checks as in the local section.

## Helm validation

1. Install the chart with a list of models and a default:

   ```bash
   helm install model2vec-serve ./helm/model2vec-serve \
     --set models={minishlab/potion-multilingual-128M,minishlab/potion-code-16M-v2} \
     --set defaultModel=minishlab/potion-multilingual-128M \
     --set apiKey=secret-key
   ```

2. Wait for pods to become ready:

   ```bash
   kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=model2vec-serve
   ```

3. Port-forward and test:

   ```bash
   kubectl port-forward svc/model2vec-serve 8080:80
   curl http://localhost:8080/health
   curl http://localhost:8080/v1/models
   curl -X POST http://localhost:8080/v1/embeddings \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer secret-key" \
     -d '{"input":"Hello from Kubernetes","model":"minishlab/potion-multilingual-128M"}'
   ```

## Contract test validation

Run the contract tests to confirm OpenAI and TEI compatibility:

```bash
cargo test --test openai_contract
cargo test --test tei_contract
```

Expected: all tests pass, including response-shape assertions for `/v1/models`, `/v1/embeddings`, `/embed`, and `/info`.

## Performance validation

Run the benchmarks and verify the goals from `research.md`:

```bash
cargo bench
```

Expected:
- p99 latency for batch-1 requests under light load is < 20 ms.
- Throughput is ≥ 2,000 batch-1 requests/sec per model.
- Peak RSS with both models loaded is < 2 GB.
- Cold-start model loading from local disk is < 3 s.

If any metric regresses by more than 10 % compared to the previous release baseline, the regression must be justified or fixed before merge.
