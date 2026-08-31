# Contract: Errors (TEI Per-Model Endpoints)

Error bodies keep the standard shape:

```json
{
  "error": "error_code",
  "message": "Human-readable description of what went wrong."
}
```

Codes and statuses for the new behaviors (all other codes per
`specs/003-multi-model-serving/contracts/errors.md`):

| HTTP Status | error code | Trigger |
|-------------|------------|---------|
| 400 | `invalid_request` | Retired `model` query parameter present on `/embed`, `/info`, `/tei/{model_id}/embed`, or `/tei/{model_id}/info` (any value, including empty). Message names the retired parameter and points to `/tei/{model_id}/...`. |
| 400 | `invalid_request` | Empty `inputs`, oversized batch, or other invalid TEI body (unchanged semantics). |
| 404 | `not_found` | `{model_id}` path segment matches no loaded model's path identifier. No fallback to the default model. |
| 401 | `unauthorized` | API key required and missing/invalid on per-model paths (same rule as existing embedding endpoints). |
| 500 | `internal_error` | Inference failure. |

## Notes

- The 404 for unknown path models is deliberate: the addressed resource (that
  model's endpoint) does not exist. The existing 400 `model_not_found`
  (OpenAI body-specified models) is unchanged.
- Messages MUST NOT leak internal paths or secrets; startup conflict errors
  are operator-facing (stderr/logs), never HTTP responses.
