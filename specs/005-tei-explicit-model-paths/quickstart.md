# Quickstart: TEI-Explicit Per-Model Endpoints

End-to-end validation for `/speckit.implement` and reviewers. Contracts:
[tei-per-model.md](./contracts/tei-per-model.md), [errors.md](./contracts/errors.md).
Data model: [data-model.md](./data-model.md).

## Prerequisites

- Rust 1.85+ (`cargo --version`).
- Models available locally for offline testing (or Hugging Face access):
  e.g., `minishlab/potion-base-32M` and `minishlab/potion-multilingual-128M`.

## Start the service (multi-model, one alias)

```bash
cargo run --release -- \
  --model minishlab/potion-base-32M \
  --model minishlab/potion-multilingual-128M \
  --model-alias "minishlab/potion-multilingual-128M=potion-multi" \
  --port 8080
```

Expected startup log: two models loaded; no conflict errors. Last-segment
identifiers: `potion-base-32M`; alias: `potion-multi`.

## Validate per-model embedding (P1)

```bash
curl -s http://localhost:8080/tei/potion-base-32M/embed \
  -H 'Content-Type: application/json' -d '{"inputs": "hello"}'
curl -s http://localhost:8080/tei/potion-multi/embed \
  -H 'Content-Type: application/json' -d '{"inputs": ["hello", "world"]}'
```

Expected: `[[...]]` — one float vector for the single string; two vectors in
order for the list. Vectors from the two models differ (different models).

## Validate per-model info — issue #105 (P2)

```bash
curl -s http://localhost:8080/tei/potion-base-32M/info
curl -s http://localhost:8080/tei/potion-multi/info
```

Expected: each returns `{model_id, max_input_length, embedding_dimension,
pooling}` describing **its own** model; `model_id` reports the canonical id
(`minishlab/...`) even via alias.

## Validate retired qualifier (P3)

```bash
curl -s -o /dev/null -w '%{http_code}\n' \
  'http://localhost:8080/embed?model=potion-base-32M' \
  -H 'Content-Type: application/json' -d '{"inputs": "hello"}'
curl -s -o /dev/null -w '%{http_code}\n' \
  'http://localhost:8080/tei/potion-base-32M/embed?model=potion-base-32M' \
  -H 'Content-Type: application/json' -d '{"inputs": "hello"}'
```

Expected: `400` both times; body `{"error":"invalid_request",...}` naming the
retired parameter and the `/tei/{model_id}/...` alternative.

Qualifier-free root endpoints still serve the default model:

```bash
curl -s http://localhost:8080/info   # → default model info
curl -s -o /dev/null -w '%{http_code}\n' http://localhost:8080/embed \
  -H 'Content-Type: application/json' -d '{"inputs": "hello"}'   # → 200
```

## Validate unknown path model

```bash
curl -s -o /dev/null -w '%{http_code}\n' \
  http://localhost:8080/tei/no-such-model/embed \
  -H 'Content-Type: application/json' -d '{"inputs": "hello"}'
```

Expected: `404` with `{"error":"not_found",...}`; the response is **not** a
default-model embedding.

## Validate startup conflict detection

```bash
cargo run --release -- \
  --model minishlab/potion-base-32M \
  --model minishlab/potion-multilingual-128M \
  --model-alias "minishlab/potion-base-32M=same" \
  --model-alias "minishlab/potion-multilingual-128M=same"
```

Both models resolve to the same path identifier `same`. Expected: process
exits non-zero with an error naming both conflicting models and the hint to
configure distinct `--model-alias` values.

## Validate auth on per-model paths

```bash
cargo run --release -- --api-key secret --model minishlab/potion-base-32M --port 8081
curl -s -o /dev/null -w '%{http_code}\n' http://localhost:8081/tei/potion-base-32M/info  # → 401
curl -s -o /dev/null -w '%{http_code}\n' -H 'Authorization: Bearer secret' \
  http://localhost:8081/tei/potion-base-32M/info                                          # → 200
```

## Automated test suite

```bash
cargo test                      # full suite
cargo test --test tei_contract  # per-model contract shapes
cargo test --test multi_model_integration
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt -- --check
```

Expected: all pass with zero warnings; OpenAI contract tests unchanged.
