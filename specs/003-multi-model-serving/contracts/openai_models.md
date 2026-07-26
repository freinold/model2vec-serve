# Contract: OpenAI-compatible Models List

## Endpoint

`GET /v1/models`

## Request

**Headers**:
- `Authorization: Bearer <api_key>` (when authentication is enabled)

**Body**: None

## Response

**Status**: `200 OK`

**Body**:

```json
{
  "object": "list",
  "data": [
    {
      "id": "minishlab/potion-multilingual-128M",
      "object": "model",
      "created": 1686935002,
      "owned_by": "minishlab"
    },
    {
      "id": "minishlab/potion-code-16M-v2",
      "object": "model",
      "created": 1686935002,
      "owned_by": "minishlab"
    }
  ]
}
```

**Fields**:
- `object` (string) — always `"list"`.
- `data` (array) — one entry per loaded model.
  - `id` (string) — model identifier.
  - `object` (string) — always `"model"`.
  - `created` (integer) — Unix timestamp in seconds.
  - `owned_by` (string) — model publisher or configured owner.

## Errors

- `401 Unauthorized` — missing or invalid API key.
- `503 Service Unavailable` — no models are loaded or ready.

See [errors.md](./errors.md) for error body shape.
