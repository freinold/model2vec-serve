# Contract: Health and Metrics

## Health / Readiness

### `GET /health`

Returns the overall health of the service and the status of each configured model.

**Response**:

**Status**: `200 OK` when at least one configured model is loaded; non-2xx when no model is ready.

```json
{
  "status": "healthy",
  "ready": true,
  "message": "2 models ready, 0 failed",
  "models": [
    {
      "model_id": "minishlab/potion-multilingual-128M",
      "status": "ready",
      "message": "model loaded"
    },
    {
      "model_id": "minishlab/potion-code-16M-v2",
      "status": "ready",
      "message": "model loaded"
    }
  ]
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
http_requests_total{method="POST",path="/v1/embeddings",status="200",model="minishlab/potion-multilingual-128M"} 42
http_requests_total{method="POST",path="/v1/embeddings",status="200",model="minishlab/potion-code-16M-v2"} 7

# HELP http_request_duration_seconds HTTP request latency
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_bucket{method="POST",path="/v1/embeddings",model="minishlab/potion-multilingual-128M",le="0.05"} 12
...
```

Metrics MUST include:
- Total request count by method/path/status with a `model` label where applicable.
- Request latency histogram with a `model` label where applicable.
- Error rate counter.

Existing dashboards that aggregate without the `model` label continue to work because the metric names and base labels are unchanged.
