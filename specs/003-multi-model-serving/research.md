# Research: Multi-Model Serving

## Decision: Validation models

- **Decision**: Use `minishlab/potion-code-16M-v2` as the code embedding validation model alongside `minishlab/potion-multilingual-128M`.
- **Rationale**: The repository exists on Hugging Face Hub at `https://huggingface.co/minishlab/potion-code-16M-v2`, is public, and contains the native Model2Vec layout (`config.json`, `model.safetensors`, `tokenizer.json`). It is a 16.2M-parameter, 256-dimensional static code-embedding model and is the successor to `minishlab/potion-code-16M`. `model2vec-rs` 0.2.x supports f16 safetensors and the native layout, so it loads correctly.
- **Alternatives considered**: `minishlab/potion-code-16M` — the predecessor, still available but explicitly superseded by v2 with worse CoIR benchmark scores.

## Decision: Multi-model loading in model2vec-rs

- **Decision**: Multiple independent `StaticModel` instances can be safely loaded and used in the same process.
- **Rationale**: `StaticModel` in `model2vec-rs` is a self-contained struct that owns its tokenizer, embeddings, weights, and token mapping. It has no static fields, no interior mutability, and no global registry. `StaticModel::from_pretrained` constructs a fresh `hf_hub::Api` per call and reads its own files into memory. Encoding methods take `&self`, so models can be shared across threads (e.g., via `Arc<StaticModel>`) and called concurrently. The only shared global side effect is a temporary `HF_HUB_TOKEN` environment variable set during authenticated downloads, which is irrelevant for local paths or identical tokens. Parallel loading is possible by issuing multiple `from_pretrained` calls concurrently from different threads (e.g., `tokio::task::spawn_blocking`), because the crate exposes only a synchronous loader.
- **Alternatives considered**: Running each model in a separate OS process — unnecessary for correctness given the crate's instance-per-model design, and it would reintroduce the operational overhead the spec is trying to avoid.

## Decision: OpenAI /v1/models response shape

- **Decision**: `GET /v1/models` returns the standard OpenAI list envelope:

  ```json
  {
    "object": "list",
    "data": [
      {
        "id": "minishlab/potion-multilingual-128M",
        "object": "model",
        "created": 1686935002,
        "owned_by": "minishlab"
      }
    ]
  }
  ```

- **Rationale**: The OpenAI "List models" API reference defines this exact shape. Keeping `model2vec-serve` compatible lets existing OpenAI clients consume `/v1/models` without modification. For static model2vec models, `created` can be a fixed timestamp and `owned_by` can reflect the model publisher (`minishlab` or a configured value).
- **Alternatives considered**: Adding embedding-specific fields (max_input_length, embedding_dimension) to the model object — rejected because it deviates from the OpenAI contract and would break clients that expect the standard shape.

## Decision: TEI multi-model compatibility strategy

- **Decision**: Keep the existing base paths `/embed` and `/info` and route them to a configured **default model**. Allow model selection with an optional `model` query parameter (e.g., `/embed?model=minishlab/potion-code-16M-v2` and `/info?model=minishlab/potion-code-16M-v2`).
- **Rationale**: The official TEI OpenAPI spec defines `/embed` and `/info` as single-model endpoints with no model selector in the request body or path. Existing TEI clients (e.g., LlamaIndex, LangChain, and custom callers) hard-code the base URL plus `/embed` and do not send a model identifier. A default model fallback makes those clients work unchanged. A query parameter is the smallest, non-breaking extension: clients that omit it get the default, and clients that want another model can add it without changing the request body or HTTP method.
- **Alternatives considered**:
  - Per-model path prefixes (`/models/{id}/embed`, `/models/{id}/info`) — RESTful but breaks existing TEI clients that expect `/embed` at the root.
  - Header-based selection (`X-Model-Id`) — works but is invisible to OpenAPI/Scalar docs and awkward for generic HTTP clients.
  - Request body `model` field on `/embed` — TEI embed bodies only contain `inputs`; adding a body field is backward-compatible only if clients ignore extra fields, and `/info` has no body.
  - One TEI instance per model — fully backward compatible but gives up the single multi-model endpoint goal.

## Decision: Multi-model performance goals

- **Decision**:
  - **p99 latency < 20 ms** for a single (batch-1) embedding request under light load.
  - **Throughput ≥ 2,000 batch-1 requests/sec per model** on a single instance.
  - **Peak RSS < 2 GB** for the Rust process with both validation models loaded.
  - **Cold-start model loading < 3 s** from local disk.
- **Rationale**: Verified model sizes on Hugging Face Hub are `model.safetensors` 512 MB + tokenizer 18.6 MB for `potion-multilingual-128M`, and `model.safetensors` 32.5 MB + tokenizer 1.0 MB for `potion-code-16M-v2`. Measured in a release Rust process using `model2vec-rs` through the service wrapper, batch-1 inference latency is ~10–32 µs, batch-64 throughput is ~77–121 k samples/sec, loading both models from the local HF cache takes ~1.7 s, and peak resident set size with both models loaded is ~1.66 GB. The 20 ms p99 target covers pure inference plus HTTP/JSON serialization, tokenization jitter, and scheduler overhead. The 2,000 RPS target is a measurable HTTP-stack floor because the model itself can sustain >40 k batch-1 inferences/sec. The 2 GB memory budget covers the measured 1.66 GB peak with headroom, and the 3 s cold-start budget covers the measured ~1.7 s combined load from local disk.
- **Alternatives considered**:
  - <5 ms p99 latency — rejected because the benchmark only measured pure model inference, not full HTTP request handling.
  - <1 GB memory budget — rejected because the measured peak RSS is already 1.66 GB.
  - ≥10,000 RPS per model — rejected because single-process HTTP throughput was not verified and 2,000 RPS is a safer, measurable floor.

## Decision: Model registry abstraction

- **Decision**: Implement a lightweight in-memory model registry keyed by model identifier, holding the loaded `StaticModel` plus derived metadata (max input length, embedding dimension, pooling).
- **Rationale**: A registry is the smallest abstraction that satisfies OpenAI `/v1/models` listing and per-model routing. Because `model2vec-rs` instances are independent and thread-safe when wrapped in `Arc`, the registry can be immutable after startup and shared across async request handlers without locks on the hot path.
- **Alternatives considered**:
  - Direct hash map lookups inside each handler with raw `Arc<StaticModel>` values — more duplication and harder to test.
  - Generic trait-based model provider — adds speculative generality not required by the spec.

## Decision: Configuration format

- **Decision**: Extend the existing CLI/env configuration to accept a list of model identifiers plus an optional default model. For backward compatibility, a single `--model` value must continue to work and be treated as both the only model and the default.
- **Rationale**: `clap` supports `Vec<String>` via repeated arguments or comma-separated values. Helm can map a YAML list to container arguments. Keeping `--model` working ensures existing deployments and documentation do not break.
- **Alternatives considered**:
  - Replace `--model` with a required `--models` list — rejected because it breaks backward compatibility.
  - JSON/YAML configuration file — adds complexity and a new file path to manage; rejected in favor of extending the existing CLI/Helm pattern.

## Decision: Observability labels

- **Decision**: Add a `model` label to existing HTTP request metrics and per-model inference counters, without changing the metric names used in the previous release.
- **Rationale**: This satisfies the spec's per-model observability requirement with the smallest change to the existing metrics contract. Existing dashboards that aggregate by method/path/status continue to work; dashboards that want per-model breakdown can filter by the new label.
- **Alternatives considered**:
  - Separate metric families per model — would explode cardinality and break existing dashboards.
  - No per-model labels — would hide per-model behavior and fail the spec.
