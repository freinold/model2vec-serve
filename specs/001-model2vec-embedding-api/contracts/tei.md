# Contract: Text Embedding Inference (TEI) Compatibility

## Endpoints

### `POST /embed`

Returns embeddings for one or more input strings.

**Headers**:
- `Content-Type: application/json`
- `Authorization: Bearer <api_key>` (when authentication is enabled)

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

When `inputs` is a list, the response is a list of embedding vectors in the same
order.

### `GET /info`

Returns metadata about the loaded model.

**Response**:

**Status**: `200 OK`

```json
{
  "model_id": "minishlab/potion-base-32M",
  "max_input_length": 512,
  "embedding_dimension": 384,
  "pooling": "mean"
}
```

## Errors

- `400 Bad Request` — invalid input, unsupported batch size, token-array input
- `401 Unauthorized` — missing or invalid API key
- `500 Internal Server Error` — inference failure

See [errors.md](./errors.md) for error body shape.
