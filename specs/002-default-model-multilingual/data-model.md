# Data Model: Default Model Update and Spec Drift Check

## Entities

### Config

Runtime configuration for the service. The only change for this feature is the
default value of the `model` field.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| host | `String` | no | `0.0.0.0` | Bind address. |
| port | `u16` | no | `8080` | Listen port. |
| model | `String` | no | `minishlab/potion-multilingual-128M` | Hugging Face id or local path of the model2vec model. |
| api_key | `Option<String>` | no | `None` | Shared secret; if set, all embeddings requests require it. |
| max_batch_size | `usize` | no | `256` | Maximum strings per request. |
| max_input_length | `usize` | no | `512` | Maximum tokens per input string. |
| log_level | `String` | no | `info` | Log verbosity. |
| request_timeout_seconds | `u64` | no | `30` | Per-request timeout. |

### DefaultModel

The model identifier used when the user does not supply one.

| Field | Type | Description |
|-------|------|-------------|
| identifier | `String` | `minishlab/potion-multilingual-128M` after this feature. |

### ModelIdentifier

A user-supplied model reference, independent of the default.

| Field | Type | Description |
|-------|------|-------------|
| value | `String` | Either a Hugging Face Hub id (`namespace/repo`) or a local filesystem path. |

### TestFixtureModel

A small model used only by tests and benchmarks to keep CI fast.

| Field | Type | Description |
|-------|------|-------------|
| identifier | `String` | `minishlab/potion-base-2M` (unchanged). |
| purpose | `String` | Download once and reuse for unit, contract, integration tests, and benchmarks. |

### EmbeddingRequest

Unchanged from feature 001. See `specs/001-model2vec-embedding-api/data-model.md`.

### EmbeddingResponse

Unchanged from feature 001. See `specs/001-model2vec-embedding-api/data-model.md`.

### ModelInfo

Unchanged from feature 001. The `model_id` field will reflect the loaded model,
which is the default model when no identifier is supplied.

## Relationships

- `Config` is loaded once at startup and shared across all request handlers via
  application state.
- `Config.model` falls back to `DefaultModel.identifier` when the user does not
  provide a value.
- `ModelIdentifier` overrides `DefaultModel` whenever the user supplies a value.
- `TestFixtureModel` is used only inside the test suite and benchmarks; it is
  never exposed to production deployments.
- `ModelInfo.model_id` is derived from the loaded model, which is either the
  user-supplied `ModelIdentifier` or the `DefaultModel`.

## Validation Rules

1. `Config.model` must be a non-empty string; the default value satisfies this.
2. If the model identifier is a Hugging Face Hub id, the model must be reachable
   at startup; otherwise the service exits with a clear error.
3. If the model identifier is a local path, the path must exist and contain the
   expected model files; otherwise the service exits with a clear error.
4. The `DefaultModel` identifier must be valid for the `model2vec-rs` loading
   path.
5. `TestFixtureModel` must remain a small, fast model to keep CI times low.
