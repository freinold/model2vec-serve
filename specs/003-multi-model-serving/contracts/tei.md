# Contract: Text Embedding Inference (TEI) Compatibility (Multi-Model)

## Endpoints

### `POST /embed`

Returns embeddings for one or more input strings using the configured default model, or the model selected by the `model` query parameter.

**Headers**:
- `Content-Type: application/json`
- `Authorization: Bearer <api_key>` (when authentication is enabled)

**Query Parameters**:
- `model` (string, optional) — must match a loaded model identifier. If omitted, the configured default model is used.

**Body**:

```json
{
  "inputs": "Hello world"
}
```

`inputs` may be a single string or a list of strings.

**Response**:

**Status**: `200 OK`

```json
[
  [0.0123, -0.0456, ...]
]
```

When `inputs` is a list, the response is a list of embedding vectors in the same order.

### `GET /info`

Returns metadata about the configured default model, or the model selected by the `model` query parameter.

**Query Parameters**:
- `model` (string, optional) — must match a loaded model identifier. If omitted, the configured default model is used.

**Response**:

**Status**: `200 OK`

```json
{
  "model_id": "minishlab/potion-multilingual-128M",
  "max_input_length": 512,
  "embedding_dimension": 384,
  "pooling": "mean"
}
```

## Errors

- `400 Bad Request` — invalid input, unsupported batch size, token-array input, or unknown model.
- `401 Unauthorized` — missing or invalid API key.
- `500 Internal Server Error` — inference failure.
- `503 Service Unavailable` — the requested model is not loaded or ready.

See [errors.md](./errors.md) for error body shape.
