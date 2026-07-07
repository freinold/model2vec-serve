# Data Model: model2vec-embedding-api

## Entities

### EmbeddingRequest

Represents a request to generate embeddings.

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| input | `String` or `Vec<String>` | yes | Must be non-empty; each string must not exceed model max length. |
| model | `String` | no | If provided, must match the loaded model identifier. |
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

### ModelInfo

Metadata exposed by the TEI-compatible info endpoint.

| Field | Type | Description |
|-------|------|-------------|
| model_id | `String` | Loaded model identifier. |
| max_input_length | `u32` | Maximum tokens accepted per input. |
| embedding_dimension | `u32` | Size of each embedding vector. |
| pooling | `String` | Pooling method used by the model, e.g. `"mean"`. |

### HealthStatus

Represents the result of a health probe.

| Field | Type | Description |
|-------|------|-------------|
| status | `String` | `"healthy"` or `"unhealthy"`. |
| ready | `bool` | True when the model is loaded and the service can serve requests. |
| message | `String` | Human-readable state description. |

### ErrorResponse

Standard error body returned for all failed requests.

| Field | Type | Description |
|-------|------|-------------|
| error | `String` | Short error code, e.g. `"invalid_request"`. |
| message | `String` | Human-readable description. |

### Config

Runtime configuration for the service.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| host | `String` | no | Bind address; defaults to `0.0.0.0`. |
| port | `u16` | no | Listen port; defaults to `8080`. |
| model_path | `String` | yes | Local path or Hugging Face id of the model2vec model. |
| api_key | `Option<String>` | no | Shared secret; if set, all embeddings requests require it. |
| max_batch_size | `usize` | no | Maximum strings per request; defaults to `256`. |
| log_level | `String` | no | e.g. `info`, `debug`; defaults to `info`. |
| request_timeout_seconds | `u64` | no | Per-request timeout; defaults to `30`. |

## Relationships

- `EmbeddingRequest` → produces one `EmbeddingResponse` containing many
  `EmbeddingObject` entries.
- `Config` is loaded once at startup and shared across all request handlers via
  application state.
- `ModelInfo` is derived from the loaded model in `Config`.

## Validation Rules

1. `input` must not be empty; empty lists return HTTP 400 with `ErrorResponse`.
2. `encoding_format` must be `"float"` or `"base64"`; other values return HTTP 400.
3. If `model` is supplied in the request, it must equal the loaded model id;
   otherwise return HTTP 400.
4. Tokenized input length must not exceed `max_input_length`; truncate or return
   HTTP 400 depending on endpoint behavior.
5. Batch size must not exceed `max_batch_size`; larger batches return HTTP 400.
6. Token arrays are not supported and return HTTP 400 with a clear message.
