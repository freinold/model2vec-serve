# Research: TEI-Explicit Per-Model Endpoints

**Feature**: 005-tei-explicit-model-paths | **Date**: 2026-08-29

Resolves all decision points for the implementation plan. Each decision lists
the choice, rationale, and rejected alternatives.

## D1: Path identifier derivation (alias → last segment)

**Decision**: Each model's per-model path identifier is computed once at
registry load: the operator-configured alias if one matches the model's
canonical id, otherwise the substring after the final `/` of the canonical id
(for local directories this is already the directory name, matching
`derive_model_id` behavior). Path identifiers are guaranteed unique within a
process.

**Rationale**: Matches clarification Q1 (Option D with C fallback). Keeps URLs
slash-free so they survive reverse proxies and require no percent-encoding.
Uniqueness is validated at startup so routing is unambiguous by construction.

**Alternatives considered**:
- Full hierarchical id in path (`/tei/minishlab%2Fpotion-.../embed`) —
  rejected: `%2F` is frequently decoded before routing by proxies/ingresses,
  producing broken paths; URLs are error-prone to configure.
- Short name only, no override — rejected: two models from different
  publishers sharing a name silently collide.
- Request-time ambiguity error — rejected by clarification: operators get
  feedback at deploy time, not from failing client traffic.

## D2: Alias configuration surface

**Decision**: New repeatable CLI flag `--model-alias KEY=ALIAS` (env
`MODEL_ALIAS`, comma-separated), parsed by clap's `value_parser` into
`Vec<(String, String)>`. `KEY` is matched against the model's canonical
identifier (the exact string configured via `--model`, i.e., the HF id or
local path as given, compared after `derive_model_id` normalization). Aliases
are single URL path segments.

**Rationale**: Extends the existing clap/env pattern (`--model` is already
repeatable with comma delimiter); zero new dependencies (Constitution IV).
A flat pair list keeps parsing, validation, and docs trivial.

**Alternatives considered**:
- JSON/YAML model config file — rejected: overkill for a handful of models;
  adds a config format to document and validate.
- Deriving alias from a new `--model id=alias` syntax — rejected: overloads
  an existing flag's meaning and breaks positional compatibility.

## D3: Startup conflict validation

**Decision**: After models load, `ModelRegistry::load` builds a
`path_identifier -> model` index. On duplicate path identifiers it returns an
error that names all conflicting canonical ids, their shared path identifier,
and the hint "configure a distinct alias via --model-alias". Startup aborts
(`AppState::new` already propagates `anyhow::Result`).

**Rationale**: Fail-fast per clarification Q1; mirrors the existing
"duplicate model identifier" startup error style. Message is
operator-facing and secret-free.

**Alternatives considered**: warning log + first-wins — rejected: silently
hides an unreachable model (regression of the same class as issue #105).

## D4: Unknown model in per-model path → 404

**Decision**: New `AppError` variant (e.g., `ModelRouteNotFound(String)`)
mapping to HTTP 404 with error code `not_found` and message "no model is
served at '/tei/{id}'". The existing `ModelNotFound` (400, `model_not_found`)
remains for OpenAI body-specified models.

**Rationale**: Per spec assumption — the addressed resource (that model's
endpoint) does not exist, so 404 is the honest status. `not_found` is already
a documented code in the errors contract; no new code needed.

**Alternatives considered**: 400 `model_not_found` — rejected: implies the
request was malformed rather than the route missing; misleads generic HTTP
clients.

## D5: Retired `?model=` qualifier handling

**Decision**: Delete `TeiModelQuery` and the `Query` extractor from TEI
handlers. Both root (`/embed`, `/info`) and per-model endpoints reject any
request carrying a `model` query parameter with `400 invalid_request` and a
message naming the retired parameter and pointing to `/tei/{model_id}/...`.
Detection: reject when the `model` key is present in the raw query string
(any value, including empty).

**Rationale**: Clarification Q2 (Option A) — path is authoritative; explicit
rejection prevents a client believing it selected a different model.
`invalid_request` is an existing documented code.

**Alternatives considered**: silently ignore — rejected: masks
misconfiguration. Tolerate-when-matching (Option C) — rejected by user:
inconsistent and harder to test/document.

## D6: Routing and handler structure

**Decision**: axum 0.8 routes `POST /tei/{model_id}/embed` and
`GET /tei/{model_id}/info` with `Path<String>` extraction, registered inside
the existing protected `api` router (so API-key auth applies identically).
Per-model handlers reuse the same validation (empty inputs, batch cap) and
encoding call as the root handlers; shared validation moves to a small
helper to avoid duplication. utoipa `#[utoipa::path]` annotations added for
both new routes; OpenAPI doc version string synced to the crate version.

**Rationale**: `{param}` is the axum 0.8 syntax; single-segment params match
the slash-free identifier design. Placing routes in the existing `api`
router reuses the auth layer with no middleware changes. `RequestModelId`
extension keeps metrics/log attribution (FR-012).

**Alternatives considered**: `Router::nest` per model built at startup —
rejected: generates one route per model instead of a parametric route,
complicating OpenAPI docs and startup for no benefit. Separate
`tei_per_model.rs` module — rejected: handlers are small variants of
existing TEI handlers; same-module placement keeps contracts together.

## D7: Registry lookup changes

**Decision**: `ModelRegistry` gains a `path_index: HashMap<String, String>`
(path identifier → canonical id) built at load; a `get_by_path(&str)` lookup
resolves `LoadedModel` in O(1). `iter()`, `resolve()`, health statuses are
unchanged; `/v1/models` continues to report canonical ids.

**Rationale**: No behavioral change to OpenAI endpoints (spec assumption);
one extra index is the minimal structure enabling the feature.

**Alternatives considered**: renaming registry keys to path identifiers —
rejected: canonical ids are needed for `model_id` fields, metrics labels, and
OpenAI responses; two namespaces must stay distinguishable.

## D8: Versioning and release

**Decision**: Bump crate version 0.3.0 → 0.5.0 (breaking major per user
direction; 0.4.x line continues Helm-chart work). Sync the hardcoded utoipa
info version. README, VitePress docs (`docs/`), TEI contract docs, and the
errors contract gain the migration note: root endpoints keep working
qualifier-free for the default model; multi-model TEI clients must switch to
`/tei/{model_id}/...`; `?model=` now returns 400.

**Rationale**: Constitution III requires a major bump and migration note for
breaking changes; user confirmed 0.5 as the release vehicle.

**Alternatives considered**: deprecation window (410 + header) — rejected by
clarification: immediate removal with explicit 400.

## D9: Observability parity

**Decision**: Per-model handlers set `RequestModelId` to the canonical model
id exactly like existing handlers; existing `http_requests_total` /
`http_request_duration_seconds` labels automatically carry the model label.
The route label uses the parametric path (`/tei/{model_id}/embed`) to avoid
cardinality explosion per model.

**Rationale**: FR-012 requires equal attribution; reusing the extension keeps
telemetry code untouched and cardinality bounded.

**Alternatives considered**: label by path identifier — rejected: aliases are
operator-chosen and less stable than canonical ids for dashboards.
