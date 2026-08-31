# Contract: TEI Per-Model Endpoints

Multi-model TEI compatibility via explicit paths. Root `/embed` and `/info`
remain qualifier-free and serve the default model; the `?model=` qualifier is
retired (see [errors.md](./errors.md)).

## Path identifier

`{model_id}` in the paths below is the model's **path identifier**:

- the operator-configured alias (`--model-alias KEY=ALIAS` / `MODEL_ALIAS`)
  for that model, or
- otherwise the last segment of the model identifier (after the final `/`).

Path identifiers are slash-free, unique per process, and validated at startup
(two models sharing an identifier abort startup with an operator-facing
error). A path identifier is not necessarily the canonical model id; response
bodies, `/v1/models`, and metrics always report the canonical id.

## `POST /tei/{model_id}/embed`

Returns embeddings for one or more input strings using the model addressed by
the path. The request never carries a model qualifier.

**Headers**:
- `Content-Type: application/json`
- `Authorization: Bearer <api_key>` (when authentication is enabled)

**Body** (identical to TEI `/embed`):

```json
{
  "inputs": "Hello world"
}
```

`inputs` may be a single string or a list of strings.

**Response**:

**Status**: `200 OK`

```json
[
  [0.0123, -0.0456, ...]
]
```

When `inputs` is a list, the response is a list of embedding vectors in the
same order.

**Errors**: `400 invalid_request` (empty/oversized inputs, retired `model`
qualifier present), `401 unauthorized`, `404 not_found` (unknown
`{model_id}`), `500 internal_error`. See [errors.md](./errors.md).

## `GET /tei/{model_id}/info`

Returns TEI info for exactly the model addressed by the path.

**Response**:

**Status**: `200 OK`

```json
{
  "model_id": "minishlab/potion-multilingual-128M",
  "max_input_length": 512,
  "embedding_dimension": 384,
  "pooling": "mean"
}
```

`model_id` is the canonical identifier even when the path used an alias.

**Errors**: `401 unauthorized`, `404 not_found` (unknown `{model_id}`), `500
internal_error`.

## Root endpoints (breaking change)

- `POST /embed`, `GET /info`: no query parameters. Requests containing the
  retired `model` query parameter receive `400 invalid_request` naming the
  retired parameter and the per-model path alternative.
- Multi-model TEI clients must point their base URL at
  `/tei/{model_id}`; single-model clients configured at the root keep
  working against the default model.

## Migration note (release 0.5.0)

| Before (≤ 0.3.x) | After (0.5.0) |
|------------------|----------------|
| `POST /embed?model=<id>` | `POST /tei/{model_id}/embed` |
| `GET /info?model=<id>` | `GET /tei/{model_id}/info` |
| `POST /embed` (default) | `POST /embed` (default, unchanged) |
| `?model=` present | `400 invalid_request` |
