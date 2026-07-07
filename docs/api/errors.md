# Errors

All error responses share a single JSON body shape.

## Error body

```json
{
  "error": "error_code",
  "message": "Human-readable description of what went wrong."
}
```

## Error codes

| HTTP Status | `error` code | Meaning |
|-------------|--------------|---------|
| `400` | `invalid_request` | Request body or parameters are invalid |
| `401` | `unauthorized` | API key missing, malformed, or incorrect |
| `403` | `forbidden` | API key valid but request not permitted |
| `404` | `not_found` | Requested endpoint does not exist |
| `422` | `unprocessable_entity` | Malformed JSON or invalid field type |
| `429` | `rate_limited` | Request rejected due to rate limiting |
| `500` | `internal_error` | Unexpected server-side failure |
| `503` | `service_unavailable` | Model not loaded or service not ready |

## Notes

- Error bodies are always valid JSON.
- The `message` field is safe to display to client developers but must not leak
  internal paths, secrets, or stack traces.
- Request correlation IDs attached to logs are not exposed in error bodies.

## Example

```bash
curl -X POST http://localhost:8080/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{"input":[]}'
```

Response:

```json
{
  "error": "invalid_request",
  "message": "input cannot be empty"
}
```
