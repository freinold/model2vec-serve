# Text Embedding Inference (TEI) Compatibility

`model2vec-serve` also exposes TEI-compatible endpoints so existing Hugging Face
ecosystem clients can reuse it.

**Breaking change in 0.5.0**: the `?model=` query parameter on `/embed` and
`/info` was removed. Models are now selected through the per-model endpoints
below; requests carrying the retired `model` query parameter receive
`400 invalid_request`.

## `POST /embed`

Returns embeddings for one or more input strings using the configured default
model. Takes no query parameters.

### Headers

- `Content-Type: application/json`
- `Authorization: Bearer <api_key>` (when authentication is enabled)

### Body

```json
{
  "inputs": "Hello world"
}
```

`inputs` may be a single string or a list of strings.

### Response

**Status**: `200 OK`

```json
[
  [0.0123, -0.0456, "..."]
]
```

When `inputs` is a list, the response is a list of embedding vectors in the
same order.

### Validation rules

- `inputs` must be non-empty.
- Batch size must not exceed `--max-batch-size`.
- Token arrays are not supported and return `400 Bad Request`.
- The retired `model` query parameter is rejected with `400 invalid_request`.

## `GET /info`

Returns metadata for the configured default model. Takes no query parameters.

### Response

**Status**: `200 OK`

```json
{
  "model_id": "minishlab/potion-multilingual-128M",
  "max_input_length": 512,
  "embedding_dimension": 384,
  "pooling": "mean"
}
```

### Field descriptions

| Field | Type | Description |
|-------|------|-------------|
| `model_id` | `string` | Canonical model identifier |
| `max_input_length` | `number` | Maximum tokens accepted per input |
| `embedding_dimension` | `number` | Size of each embedding vector |
| `pooling` | `string` | Pooling method used by the model |

## `POST /tei/{model_id}/embed`

Returns embeddings for one or more input strings using the model addressed by
the path. The request never carries a model qualifier.

`{model_id}` is the model's **path identifier**: the operator-configured alias
(`--model-alias KEY=ALIAS` / `MODEL_ALIAS`) or, otherwise, the last segment of
the model identifier (e.g. `minishlab/potion-multilingual-128M` →
`potion-multilingual-128M`). Response bodies and `/v1/models` always report the
canonical model id.

### Headers

- `Content-Type: application/json`
- `Authorization: Bearer <api_key>` (when authentication is enabled)

### Body

Identical to [`POST /embed`](#post-embed): a single string or a list of
strings.

### Response

**Status**: `200 OK`

```json
[
  [0.0123, -0.0456, "..."]
]
```

When `inputs` is a list, the response is a list of embedding vectors in the
same order.

### Errors

- `404 not_found` when `{model_id}` matches no loaded model's path identifier.
  There is no fallback to the default model.

## `GET /tei/{model_id}/info`

Returns TEI info for exactly the model addressed by the path.

### Response

**Status**: `200 OK`

```json
{
  "model_id": "minishlab/potion-multilingual-128M",
  "max_input_length": 512,
  "embedding_dimension": 384,
  "pooling": "mean"
}
```

`model_id` is the canonical identifier even when the path used an alias.

## Errors

| Status | `error` code | Cause |
|--------|--------------|-------|
| `400` | `invalid_request` | Invalid input, unsupported batch size, token-array input, or the retired `model` query parameter present |
| `401` | `unauthorized` | Missing or invalid API key |
| `404` | `not_found` | `{model_id}` path segment matches no loaded model |
| `500` | `internal_error` | Inference failure |

See [Errors](./errors.md) for the error body shape.

## Example with curl

```bash
curl -X POST http://localhost:8080/embed \
  -H "Content-Type: application/json" \
  -d '{"inputs":["Hello","World"]}'

curl -X POST http://localhost:8080/tei/potion-code-16M-v2/embed \
  -H "Content-Type: application/json" \
  -d '{"inputs":"def hello(): pass"}'

curl http://localhost:8080/info
curl http://localhost:8080/tei/potion-code-16M-v2/info
```
