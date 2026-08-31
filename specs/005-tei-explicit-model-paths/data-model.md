# Data Model: TEI-Explicit Per-Model Endpoints

**Feature**: 005-tei-explicit-model-paths | **Date**: 2026-08-29

## Entities

### ServedModel (extends existing `LoadedModel`)

A model loaded for inference, now carrying a per-model path identifier.

| Field | Type | Source | Rules |
|-------|------|--------|-------|
| `model_id` | `String` | derived from `--model` value (full HF id; local dir name for paths) | Immutable after load; canonical identity for responses, metrics, OpenAI list |
| `path_identifier` | `String` | configured alias matching `model_id`, else substring after final `/` of `model_id` | Single URL path segment (no `/`, non-empty, no whitespace); unique across registry; immutable after load |
| `max_input_length` | `usize` | config | Unchanged |
| `embedding_dimension` | `usize` | model | Unchanged |
| `pooling` | `&'static str` | constant `"mean"` | Unchanged |
| `model` | `Arc<EmbeddingModel>` | loader | Unchanged |

Validation rules (startup):
- `path_identifier` MUST be non-empty and slash-free; a configured alias
  violating this is a startup error.
- Two loaded models MUST NOT share a `path_identifier`; violation aborts
  startup with an error naming the conflicting canonical ids and hinting to
  configure distinct aliases via `--model-alias`.

### ModelRegistry (modified)

| Member | Change | Rules |
|--------|--------|-------|
| `models: HashMap<String, LoadedModel>` | unchanged | keyed by canonical `model_id` |
| `path_index: HashMap<String, String>` | NEW | `path_identifier -> model_id`; built once in `load()`; lookup via `get_by_path()` |
| `default_model_id: String` | unchanged | root `/embed`, `/info`, and OpenAI default resolution |
| `failed_models: Vec<(String, anyhow::Error)>` | unchanged | health reporting only; failed models get no path routes |

### ModelAlias (new config input)

| Field | Type | Source | Rules |
|-------|------|--------|-------|
| `key` | `String` | `--model-alias KEY=ALIAS` / `MODEL_ALIAS` (repeatable, comma-separated) | Matches a canonical `model_id` (after `derive_model_id` normalization); unknown keys are a startup error naming the unmatched key |
| `alias` | `String` | same | Must be a valid single path segment |

### ErrorResponse (unchanged shape, new usage)

`{ "error": "<code>", "message": "<text>" }` — see
[contracts/errors.md](./contracts/errors.md) for the new 404 mapping and the
retired-qualifier 400.

### TeiEmbedRequest / ModelInfo (unchanged)

Per-model endpoints reuse the existing TEI DTOs; no field additions.

## Relationships

- One `ModelRegistry` owns N `ServedModel`s; exactly one is the
  `default_model_id` (root TEI endpoints + OpenAI fallback).
- Each `ServedModel` maps to exactly one `path_identifier` (1:1 via
  `path_index`); N models never share one.
- `path_identifier` and `model_id` are both stable for the process lifetime;
  no runtime mutation (models load only at startup).

## State Transitions

Startup (only lifecycle):

1. Parse `--model-alias` pairs → validate `KEY=ALIAS` shape and alias
   segment validity.
2. Load models concurrently (existing behavior).
3. Derive `path_identifier` per loaded model (alias match → last segment).
4. Build `path_index`; on duplicate → abort startup with operator-facing
   error (conflicting ids + alias hint).
5. Validate alias keys all matched a loaded model; unmatched key → abort
   with error naming the key.
6. Serve: `/tei/{path_identifier}/embed|info` resolve via `path_index`;
   unknown identifier → 404 at request time (no fallback).

Request-time is stateless: every resolution is a lookup; no per-request
state changes.
