# Contract: Errors

All error responses use the following JSON body shape:

```json
{
  "error": "error_code",
  "message": "Human-readable description of what went wrong."
}
```

## Error Codes

| HTTP Status | error code | Meaning |
|-------------|------------|---------|
| 400 | `invalid_request` | Request body or parameters are invalid. |
| 400 | `model_not_found` | Requested model identifier is not loaded or unavailable. |
| 401 | `unauthorized` | API key missing, malformed, or incorrect. |
| 403 | `forbidden` | API key valid but request not permitted. |
| 404 | `not_found` | Requested endpoint does not exist. |
| 422 | `unprocessable_entity` | Malformed JSON or invalid field type. |
| 429 | `rate_limited` | Request rejected due to rate limiting. |
| 500 | `internal_error` | Unexpected server-side failure. |
| 503 | `service_unavailable` | Model not loaded or service not ready. |

## Notes

- Error bodies MUST be valid JSON.
- The `message` field SHOULD be safe to display to client developers but MUST not leak internal paths or secrets.
- Request correlation IDs attached to logs are not exposed in error bodies.
- When a request omits the `model` parameter and no default model is configured, the service returns `400 invalid_request` with a clear message.
- When a request references a model that failed to load or does not exist, the service returns `400 model_not_found` or `503 service_unavailable` depending on whether the model was configured but failed.
