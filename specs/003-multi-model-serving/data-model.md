# Data Model: Multi-Model Serving

## Entities

### ModelConfig

Runtime configuration for the service, extended to support multiple models.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| host | `String` | no | Bind address; defaults to `0.0.0.0`. |
| port | `u16` | no | Listen port; defaults to `8080`. |
| models | `Vec<String>` | yes | List of model identifiers (Hugging Face Hub ids or local paths) to load at startup. |
| default_model | `Option<String>` | no | Model identifier to use when a request does not specify one. If omitted, the first entry in `models` is used as the default. |
| api_key | `Option<String>` | no | Shared secret; if set, all embeddings requests require it. |
| max_batch_size | `usize` | no | Maximum strings per request; defaults to `256`. |
| log_level | `String` | no | e.g. `info`, `debug`; defaults to `info`. |
| request_timeout_seconds | `u64` | no | Per-request timeout; defaults to `30`. |

### LoadedModel

A model that has been loaded into memory and is ready for inference.

| Field | Type | Description |
|-------|------|-------------|
| model_id | `String` | Canonical identifier used for routing and in responses. |
| max_input_length | `usize` | Maximum tokens accepted per input. |
| embedding_dimension | `usize` | Size of each embedding vector. |
| pooling | `String` | Pooling method used by the model, e.g. `"mean"`. |
| model | `Arc<StaticModel>` | The underlying model2vec model instance. |

### ModelRegistry

The set of loaded models available for inference.

| Field | Type | Description |
|-------|------|-------------|
| models | `HashMap<String, LoadedModel>` | Loaded models keyed by canonical identifier. |
| default_model_id | `String` | Identifier to use when a request omits the model. |

### ModelInfo

Metadata exposed by the OpenAI-compatible `/v1/models` endpoint.

| Field | Type | Description |
|-------|------|-------------|
| id | `String` | Model identifier. |
| object | `String` | Always `"model"`. |
| created | `i64` | Unix timestamp in seconds. |
| owned_by | `String` | Model publisher or configured owner. |

### ModelInfoTEI

Metadata exposed by the TEI-compatible `/info` endpoint.

| Field | Type | Description |
|-------|------|-------------|
| model_id | `String` | Model identifier. |
| max_input_length | `usize` | Maximum tokens accepted per input. |
| embedding_dimension | `usize` | Size of each embedding vector. |
| pooling | `String` | Pooling method used by the model. |

### EmbeddingRequest

Represents a request to generate embeddings.

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| input | `String` or `Vec<String>` | yes | Must be non-empty; each string must not exceed the selected model's max length. |
| model | `String` | no | If provided, must match a loaded model identifier; otherwise the default model is used. |
| encoding_format | `String` | no | Must be `"float"` or `"base64"` if provided; defaults to `"float"`. |

### EmbeddingResponse

Represents the successful embeddings response.

| Field | Type | Description |
|-------|------|-------------|
| object | `String` | Always `"list"` for OpenAI compatibility. |
| data | `Vec<EmbeddingObject>` | One entry per input, in the same order. |
| model | `String` | Identifier of the model that produced the embeddings. |
| usage | `Usage` | Token/prompt usage metadata. |

### EmbeddingObject

A single embedding result.

| Field | Type | Description |
|-------|------|-------------|
| object | `String` | Always `"embedding"`. |
| index | `u32` | Position in the input list. |
| embedding | `Vec<f32>` or `String` | Float array or base64-encoded bytes. |

### Usage

Metadata about resource consumption.

| Field | Type | Description |
|-------|------|-------------|
| prompt_tokens | `u32` | Number of tokens in the input. |
| total_tokens | `u32` | Same as prompt_tokens for embedding requests. |

### ModelStatus

The load/health state of a model.

| Field | Type | Description |
|-------|------|-------------|
| model_id | `String` | Model identifier. |
| status | `String` | `"loading"`, `"ready"`, or `"failed"`. |
| message | `String` | Human-readable state description, especially for failures. |

### HealthStatus

Represents the result of a health probe.

| Field | Type | Description |
|-------|------|-------------|
| status | `String` | `"healthy"` or `"unhealthy"`. |
| ready | `bool` | True when at least one configured model is loaded and the service can serve requests. |
| message | `String` | Human-readable state description. |
| models | `Vec<ModelStatus>` | Per-model status for observability. |

### ErrorResponse

Standard error body returned for all failed requests.

| Field | Type | Description |
|-------|------|-------------|
| error | `String` | Short error code, e.g. `"invalid_request"` or `"model_not_found"`. |
| message | `String` | Human-readable description. |

## Relationships

- `ModelConfig` declares one or more `ModelIdentifier` values and an optional `DefaultModel`.
- At startup, each configured identifier is loaded into a `LoadedModel` and stored in the `ModelRegistry`.
- `ModelRegistry` is immutable after startup and shared across all request handlers.
- `EmbeddingRequest` selects a `LoadedModel` from the `ModelRegistry` via `model_id`.
- `EmbeddingRequest` produces one `EmbeddingResponse` containing many `EmbeddingObject` entries.
- `ModelInfo` and `ModelInfoTEI` are derived from a `LoadedModel`.
- `HealthStatus` aggregates `ModelStatus` values from the registry.

## Validation Rules

1. `models` must contain at least one identifier; an empty list is rejected at startup.
2. `default_model`, if provided, must be present in `models`.
3. Duplicate entries in `models` are de-duplicated or rejected at startup.
4. `input` must not be empty; empty lists return HTTP 400 with `ErrorResponse`.
5. `encoding_format` must be `"float"` or `"base64"`; other values return HTTP 400.
6. If `model` is supplied in the request, it must match a loaded model identifier; otherwise return HTTP 400 with code `model_not_found` or `invalid_request`.
7. Batch size must not exceed `max_batch_size`; larger batches return HTTP 400.
8. Tokenized input length must not exceed the selected model's `max_input_length`; truncate or return HTTP 400 depending on endpoint behavior.
9. Token arrays are not supported and return HTTP 400 with a clear message.
10. If a model fails to load but at least one model loads successfully, the service remains ready for the healthy models and reports the failed model in `HealthStatus`.
