# OpenAI-compatible Embeddings

`model2vec-serve` implements the OpenAI embeddings endpoint so existing clients
can switch to a self-hosted model with only a base-URL change.

## Endpoint

```http
POST /v1/embeddings
```

## Request

### Headers

- `Content-Type: application/json`
- `Authorization: Bearer <api_key>` (when authentication is enabled)

### Body

```json
{
  "input": "Hello world",
  "model": "minishlab/potion-multilingual-128M",
  "encoding_format": "float"
}
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `input` | `string` or `string[]` | yes | Text(s) to embed |
| `model` | `string` | no | Must match the loaded model id |
| `encoding_format` | `string` | no | `"float"` or `"base64"`; defaults to `"float"` |

## Response

### Status

`200 OK`

### Body

```json
{
  "object": "list",
  "data": [
    {
      "object": "embedding",
      "index": 0,
      "embedding": [0.0123, -0.0456, "..."]
    }
  ],
  "model": "minishlab/potion-multilingual-128M",
  "usage": {
    "prompt_tokens": 2,
    "total_tokens": 2
  }
}
```

When `input` is an array, `data` contains one embedding object per input in the
same order.

### Field descriptions

| Field | Type | Description |
|-------|------|-------------|
| `object` | `string` | Always `"list"` |
| `data` | `EmbeddingObject[]` | One entry per input |
| `model` | `string` | Loaded model identifier |
| `usage.prompt_tokens` | `number` | Estimated tokens in the input |
| `usage.total_tokens` | `number` | Same as `prompt_tokens` |

## Validation rules

- `input` must be non-empty.
- Batch size must not exceed `--max-batch-size`.
- `encoding_format` must be `"float"` or `"base64"`.
- If `model` is supplied, it must match the loaded model id.

## Errors

| Status | `error` code | Cause |
|--------|--------------|-------|
| `400` | `invalid_request` | Empty input, unsupported encoding, mismatched model, batch too large |
| `401` | `unauthorized` | Missing or invalid API key |
| `422` | `unprocessable_entity` | Malformed JSON |
| `500` | `internal_error` | Model inference failure |

See [Errors](./errors.md) for the error body shape.

## Example with curl

```bash
curl -X POST http://localhost:8080/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{"input":["Hello","World"],"model":"minishlab/potion-multilingual-128M"}'
```
