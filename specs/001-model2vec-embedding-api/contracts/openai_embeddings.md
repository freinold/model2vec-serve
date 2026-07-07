# Contract: OpenAI-compatible Embeddings

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
  "model": "minishlab/potion-base-32M",
  "encoding_format": "float"
}
```

**Fields**:
- `input` (string or list of strings, required)
- `model` (string, optional) — must match the loaded model id
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
  "model": "minishlab/potion-base-32M",
  "usage": {
    "prompt_tokens": 2,
    "total_tokens": 2
  }
}
```

## Errors

- `400 Bad Request` — invalid input format, unsupported encoding, mismatched model
- `401 Unauthorized` — missing or invalid API key
- `422 Unprocessable Entity` — malformed JSON
- `500 Internal Server Error` — model inference failure

See [errors.md](./errors.md) for error body shape.
