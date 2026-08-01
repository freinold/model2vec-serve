# Contract: OpenAI-compatible Embeddings (Multi-Model)

## Endpoint

`POST /v1/embeddings`

## Request

**Headers**:
- `Content-Type: application/json`
- `Authorization: Bearer <api_key>` (when authentication is enabled)

**Body**:

```json
{
  "input": "Hello world",
  "model": "minishlab/potion-multilingual-128M",
  "encoding_format": "float"
}
```

**Fields**:
- `input` (string or list of strings, required)
- `model` (string, optional) — must match a loaded model identifier. If omitted, the configured default model is used.
- `encoding_format` (string, optional) — `"float"` or `"base64"`, defaults to `"float"`

## Response

**Status**: `200 OK`

**Body**:

```json
{
  "object": "list",
  "data": [
    {
      "object": "embedding",
      "index": 0,
      "embedding": [0.0123, -0.0456, ...]
    }
  ],
  "model": "minishlab/potion-multilingual-128M",
  "usage": {
    "prompt_tokens": 2,
    "total_tokens": 2
  }
}
```

The `model` field in the response is the identifier of the model that actually produced the embeddings, which may be the default model if the request did not specify one.

## Errors

- `400 Bad Request` — invalid input format, unsupported encoding, mismatched model, or `model_not_found` (requested model is not loaded or unavailable).
- `401 Unauthorized` — missing or invalid API key.
- `422 Unprocessable Entity` — malformed JSON.
- `500 Internal Server Error` — model inference failure.

See [errors.md](./errors.md) for error body shape.
