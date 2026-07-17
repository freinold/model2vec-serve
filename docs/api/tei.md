# Text Embedding Inference (TEI) Compatibility

`model2vec-serve` also exposes TEI-compatible endpoints so existing Hugging Face
ecosystem clients can reuse it.

## `POST /embed`

Returns embeddings for one or more input strings.

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

## `GET /info`

Returns metadata about the loaded model.

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
| `model_id` | `string` | Loaded model identifier |
| `max_input_length` | `number` | Maximum tokens accepted per input |
| `embedding_dimension` | `number` | Size of each embedding vector |
| `pooling` | `string` | Pooling method used by the model |

## Errors

| Status | `error` code | Cause |
|--------|--------------|-------|
| `400` | `invalid_request` | Invalid input, unsupported batch size, token-array input |
| `401` | `unauthorized` | Missing or invalid API key |
| `500` | `internal_error` | Inference failure |

See [Errors](./errors.md) for the error body shape.

## Example with curl

```bash
curl -X POST http://localhost:8080/embed \
  -H "Content-Type: application/json" \
  -d '{"inputs":["Hello","World"]}'

curl http://localhost:8080/info
```
