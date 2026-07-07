# Contract: Health and Metrics

## Health / Readiness

### `GET /health`

Returns the overall health of the service.

**Response**:

**Status**: `200 OK` when the model is loaded; non-2xx when not ready.

```json
{
  "status": "healthy",
  "ready": true,
  "message": "model loaded and serving requests"
}
```

### `GET /ready`

Alias for `/health` intended for Kubernetes readiness probes.

**Response**: Same as `/health`.

## Metrics

### `GET /metrics`

Returns Prometheus-compatible metrics.

**Response**:

**Status**: `200 OK`

**Content-Type**: `text/plain; version=0.0.4`

**Example body**:

```text
# HELP http_requests_total Total HTTP requests
# TYPE http_requests_total counter
http_requests_total{method="POST",path="/v1/embeddings",status="200"} 42

# HELP http_request_duration_seconds HTTP request latency
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_bucket{le="0.05"} 12
...
```

Metrics MUST include:
- Total request count by method/path/status
- Request latency histogram
- Error rate counter
