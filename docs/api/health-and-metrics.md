# Health and Metrics

Operational endpoints are unauthenticated so probes and monitoring stacks can
reach them directly.

## `GET /health`

Returns the overall health of the service.

### Response

**Status**: `200 OK` when the model is loaded; non-2xx when not ready.

```json
{
  "status": "healthy",
  "ready": true,
  "message": "model loaded and serving requests"
}
```

### Field descriptions

| Field | Type | Description |
|-------|------|-------------|
| `status` | `string` | `"healthy"` or `"unhealthy"` |
| `ready` | `boolean` | Whether the service can serve requests |
| `message` | `string` | Human-readable state description |

## `GET /ready`

Alias for `/health` intended for Kubernetes readiness probes.

### Response

Same as `/health`.

## `GET /metrics`

Returns Prometheus-compatible metrics.

### Response

**Status**: `200 OK`

**Content-Type**: `text/plain; version=0.0.4`

### Example output

```text
# HELP http_requests_total Total HTTP requests
# TYPE http_requests_total counter
http_requests_total{method="POST",path="/v1/embeddings",status="200"} 42

# HELP http_request_duration_seconds HTTP request latency
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_bucket{le="0.05"} 12
...

# HELP http_errors_total Total HTTP 5xx errors
# TYPE http_errors_total counter
http_errors_total{status="500"} 0
```

### Exposed metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `http_requests_total` | counter | `method`, `path`, `status` | Total request count |
| `http_request_duration_seconds` | histogram | `method`, `path` | Request latency |
| `http_errors_total` | counter | `status` | 5xx error count |

### Request correlation

Every request is assigned a correlation ID. The ID is returned in the
`x-request-id` response header and included in structured logs, but it is not
exposed in response bodies or metrics labels.
