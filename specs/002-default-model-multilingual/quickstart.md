# Quickstart: Default Model Update and Spec Drift Check

This guide validates the feature end-to-end after implementation.

## Prerequisites

- Rust toolchain (stable, 1.85+)
- Docker (for container validation)
- Helm 3+ and a Kubernetes cluster or local environment such as kind/minikube
- Network access to Hugging Face Hub for downloading the default model

## Default model validation

1. Start the service without specifying a model:

   ```bash
   cargo run --release -- --port 8080
   ```

2. Verify the `/info` endpoint reports the new default model:

   ```bash
   curl http://localhost:8080/info
   ```

   Expected: `{"model_id":"minishlab/potion-multilingual-128M",...}`

3. Verify the health endpoint is ready:

   ```bash
   curl http://localhost:8080/health
   ```

   Expected: `{"status":"healthy","ready":true,...}`

4. Call the OpenAI-compatible endpoint with non-English input:

   ```bash
   curl -X POST http://localhost:8080/v1/embeddings \
     -H "Content-Type: application/json" \
     -d '{"input":"Hola mundo","model":"minishlab/potion-multilingual-128M"}'
   ```

   Expected: JSON list response with one embedding object.

5. Call the TEI-compatible endpoint:

   ```bash
   curl -X POST http://localhost:8080/embed \
     -H "Content-Type: application/json" \
     -d '{"inputs":"Hola mundo"}'
   ```

   Expected: JSON array containing one embedding vector.

## Explicit model override validation

1. Start the service with the previous default model:

   ```bash
   cargo run --release -- --model minishlab/potion-base-2M --port 8080
   ```

2. Verify the `/info` endpoint reports the explicitly supplied model:

   ```bash
   curl http://localhost:8080/info
   ```

   Expected: `{"model_id":"minishlab/potion-base-2M",...}`

3. Verify the embeddings endpoint still works as in the previous section.

## Local path override validation

1. Start the service with a local model path:

   ```bash
   cargo run --release -- --model /path/to/local/model --port 8080
   ```

2. Verify the `/info` endpoint reports the local path or its directory name:

   ```bash
   curl http://localhost:8080/info
   ```

   Expected: model identifier derived from the local path.

## Container validation

1. Build the image:

   ```bash
   docker build -t model2vec-serve:latest .
   ```

2. Run the container without specifying a model:

   ```bash
   docker run -p 8080:8080 model2vec-serve:latest
   ```

   Expected: The container downloads and loads `minishlab/potion-multilingual-128M`
   and the health endpoint becomes ready.

3. Run the same curl checks as in the local section.

## Helm validation

1. Install the chart without setting the model value:

   ```bash
   helm install model2vec-serve ./helm/model2vec-serve \
     --set apiKey=secret-key \
     --set replicaCount=1
   ```

2. Wait for pods to become ready:

   ```bash
   kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=model2vec-serve
   ```

3. Port-forward and test:

   ```bash
   kubectl port-forward svc/model2vec-serve 8080:80
   curl http://localhost:8080/health
   curl http://localhost:8080/info
   ```

   Expected: `/info` reports `minishlab/potion-multilingual-128M`.

## Documentation consistency validation

1. Search for remaining default references to the old model:

   ```bash
   grep -R "potion-base-2M" README.md AGENTS.md helm/model2vec-serve/README.md \
     helm/model2vec-serve/values.yaml docs/ specs/001-model2vec-embedding-api/
   ```

   Expected: Only references that intentionally demonstrate a non-default model
   remain, or none at all.

2. Verify the new default appears in the expected locations:

   ```bash
   grep -R "potion-multilingual-128M" README.md AGENTS.md \
     helm/model2vec-serve/values.yaml helm/model2vec-serve/README.md docs/ \
     src/config.rs
   ```

   Expected: The new default is present in CLI defaults, Helm defaults, and
   quick-start examples.

## Test validation

Run the full test suite and static checks:

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Expected: all tests pass and the new default behavior is covered by a
configuration test.

## Spec drift validation

1. Review the updated `specs/001-model2vec-embedding-api/research.md` and confirm
   the Hugging Face Hub integration note mentions the direct test/benchmark usage
   of the `hf-hub` v1.x API.

2. Confirm no statement in the existing spec or research documents contradicts
   the current model loading path or test fixture download code.

3. If drift was found and fixed, verify the changes are committed to the feature
   branch.
