# Feature Specification: TEI-Explicit Per-Model Endpoints

**Feature Branch**: `005-tei-explicit-model-paths`

**Created**: 2026-08-29

**Status**: Draft

**Input**: User description: "in relation to issue https://github.com/freinold/model2vec-serve/issues/105, we need to change the TEI implementation to support requests without optional model qualifier for TEI clients. Instead, multiple models should be served TEI compatible on paths /tei/{model_id}/embed and /tei/{model_id}/info. this way we skip the hidden optional model id and make the model choice verbose via api path."

## Clarifications

### Session 2026-08-29

- Q: Which forms of the model identifier should be accepted in the per-model path segment `{model_id}`? → A: An operator-configured path alias; when no alias is configured, the identifier's last segment (after the final `/`) is used. If two models resolve to the same path identifier, the program must refuse to start and notify the operator (naming the conflict) with a hint to configure an alias.
- Q: When a request hits a per-model endpoint and still includes the retired `model` query parameter, should the server reject or ignore it? → A: Always reject with the same explicit "retired parameter" client error used on the root endpoints; the breaking change ships as the 0.5 major release, with the docs clearly flagging the broken contract and migration path.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Embed Text with an Explicitly Chosen Model (Priority: P1)

An operator serves multiple embedding models from a single deployment. A
TEI-compatible client that never sends a model qualifier wants embeddings from
one specific model. The client configures its base URL to include that model's
dedicated path segment (for example, `.../tei/{model_id}`) and sends a standard
TEI embedding request. The model choice is fully visible in the URL, not hidden
in a query parameter or request body.

**Why this priority**: Embedding generation is the service's primary function.
Making model selection explicit in the path is the core change that lets
TEI-compatible clients use any served model without protocol extensions.

**Independent Test**: Can be fully tested by starting the service with two
configured models, sending a standard TEI embedding request (string or list of
strings, no model qualifier) to each model's per-model path, and verifying each
response is a valid TEI embedding response reflecting the selected model.

**Acceptance Scenarios**:

1. **Given** two models are configured and loaded, **When** a client sends a
   TEI embedding request containing only an `inputs` field to model A's
   per-model embedding path, **Then** the response is the TEI embedding shape
   (a list of float vectors, one per input, in input order) computed by model A.
2. **Given** the same deployment, **When** the identical request is sent to
   model B's per-model embedding path, **Then** the response is computed by
   model B, demonstrating both models are independently reachable.
3. **Given** a request body with a single string input, **When** sent to a
   per-model embedding path, **Then** the response contains exactly one
   embedding vector.

---

### User Story 2 - Retrieve Metadata for Every Served Model (Priority: P2)

A user of a multi-model deployment wants to know each model's maximum input
length, embedding dimension, and pooling mode. Today only the default model's
metadata is exposed on the TEI info endpoint (issue #105). With per-model info
paths, every served model has its own info endpoint that describes exactly that
model, so no model's metadata is hidden behind the default.

**Why this priority**: Resolves the reported defect (#105) where multi-model
deployments expose metadata for only one model; metadata discovery is essential
for clients to configure dimensions and input limits, but it depends on the
per-model routing introduced in User Story 1.

**Independent Test**: Can be fully tested by starting the service with at least
two models, requesting each model's per-model info path, and verifying each
response matches the TEI info shape and reports that model's own identifier,
max input length, embedding dimension, and pooling mode.

**Acceptance Scenarios**:

1. **Given** two models are configured, **When** a client requests model A's
   per-model info path, **Then** the response returns model A's identifier,
   max input length, embedding dimension, and pooling mode in the TEI info
   shape.
2. **Given** the same deployment, **When** a client requests model B's
   per-model info path, **Then** the response returns model B's metadata —
   both models' metadata is discoverable, not just the default's.
3. **Given** a single-model deployment, **When** a client requests that
   model's per-model info path, **Then** the response shape is identical to the
   existing TEI info response.

---

### User Story 3 - Unambiguous Retirement of the Hidden Model Qualifier (Priority: P3)

The current TEI endpoints accept a hidden, optional model qualifier that
selects a model without revealing the choice in the URL. This qualifier is
retired: requests that still include it receive a clear, explicit client error
that names the retired parameter and points to the path-based alternative.
Existing single-model TEI clients that never send a qualifier keep working
unchanged against the root endpoints. The change and its migration path are
published with the release.

**Why this priority**: Prevents two competing selection mechanisms and silent
misconfiguration, but it only affects clients that used the hidden qualifier;
the primary value is delivered by User Stories 1 and 2.

**Independent Test**: Can be fully tested by sending embedding and info
requests that include the retired model qualifier and verifying each receives
the documented explicit error response; and by sending qualifier-free requests
to the root endpoints and verifying they still serve the default model.

**Acceptance Scenarios**:

1. **Given** the retired qualifier is included on a root embedding or info
   request, **When** the request is processed, **Then** the server responds
   with a documented client error naming the retired parameter and the
   path-based alternative, rather than silently serving a model.
2. **Given** a single-model TEI client configured against the service root,
   **When** it sends standard qualifier-free requests to `/embed` and `/info`,
   **Then** both continue to work and serve the configured default model.
3. **Given** the release notes and API documentation, **When** an operator
   reads them, **Then** the removal of the hidden qualifier, the new
   per-model paths, and a migration example are documented.

---

### Edge Cases

- What happens when the model identifier contains hierarchical parts
  (slashes)? Only the last segment is used in paths unless an operator has
  configured an alias, keeping per-model URLs slash-free and URL-safe.
- How does the system handle an identifier in the path that matches no loaded
  model? It must return a documented client error and must not fall back to
  the default model.
- What happens when two loaded models would resolve to the same path
  identifier (same last segment and no distinguishing aliases)? Startup must
  fail with an operator-facing error naming the conflicting models and a hint
  to configure an alias, instead of serving ambiguous routes.
- What happens when requests use otherwise-invalid TEI bodies (token-array
  inputs, oversized batches, missing `inputs`)? Error semantics must match the
  existing TEI embed contract.
- What happens when a request to a per-model endpoint includes the retired
  model qualifier? It must be rejected with the same explicit client error as
  on the root endpoints — the path alone selects the model.
- How does the system treat the root `/info` endpoint in a multi-model
  deployment? It continues to describe the default model only, preserving the
  single-object TEI shape; per-model metadata is obtained from each model's
  own info path.
- What happens when authentication is enabled? Per-model embedding and info
  paths must be protected exactly like the existing embedding endpoints, while
  health, readiness, and metrics remain public.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST expose a dedicated TEI-compatible embedding
  endpoint for every loaded model, whose path explicitly names the model
  (e.g., `/tei/{model_id}/embed`).
- **FR-002**: The system MUST expose a dedicated TEI-compatible info endpoint
  for every loaded model, whose path explicitly names the model (e.g.,
  `/tei/{model_id}/info`).
- **FR-003**: Per-model embedding endpoints MUST accept the same request
  bodies as the existing TEI embed endpoint (a single string or a list of
  strings) and MUST return embeddings in the same shape and order as the
  inputs.
- **FR-004**: Per-model info endpoints MUST return the TEI info shape
  (model id, max input length, embedding dimension, pooling) describing
  exactly the model named in the path.
- **FR-005**: Model selection on per-model endpoints MUST be determined solely
  by the path; such requests MUST NOT require or use any model qualifier in
  query parameters or request bodies. Requests that include the retired
  qualifier MUST be rejected with the same explicit "retired parameter"
  client error used on the root endpoints.
- **FR-006**: The root `/embed` and `/info` endpoints MUST continue to operate
  without any model qualifier, serving the configured default model, so that
  existing single-model TEI client setups remain compatible.
- **FR-007**: The hidden optional model qualifier currently supported on
  `/embed` and `/info` MUST be removed; requests that include it MUST receive
  an explicit client error that identifies the retired parameter and points to
  the path-based per-model endpoints.
- **FR-008**: Each model's per-model path identifier MUST be its
  operator-configured alias or, when no alias is configured, the last segment
  of its model identifier (after the final `/`). The system MUST refuse to
  start — notifying the operator of the conflict and hinting to configure an
  alias — when two loaded models resolve to the same path identifier.
- **FR-009**: Requests that name an unknown or not-loaded model in the path
  MUST receive a documented client error and MUST NOT fall back to the
  default model.
- **FR-010**: In multi-model deployments, EVERY configured and loaded model
  MUST be reachable through its per-model embedding and info endpoints; no
  model may be reachable only through the default.
- **FR-011**: Per-model embedding and info endpoints MUST be subject to the
  same authentication rules as the existing embedding endpoints when API-key
  authentication is enabled.
- **FR-012**: Requests to per-model endpoints MUST be attributable to the
  selected model in the service's operational outputs (metrics and request
  logs) to the same degree as existing multi-model endpoints.
- **FR-013**: The interactive API documentation and published documentation
  MUST describe the per-model endpoints and the retirement of the hidden
  qualifier, including a migration note and example; because this is a
  breaking contract change, the change MUST ship as a breaking (major)
  release — 0.5 — and the documentation MUST clearly flag the broken contract
  and the migration path.

### Key Entities *(include if feature involves data)*

- **Served Model**: A model loaded for inference. Attributes include its full
  identifier, a path identifier (configured alias, or the identifier's last
  segment when no alias is configured), maximum input length,
  embedding dimension, and pooling mode. Exactly one loaded model is the
  configured default. Each Served Model maps to exactly one per-model path
  prefix serving its embedding and info endpoints.
- **Default Model**: The Served Model used by the root `/embed` and `/info`
  endpoints and by OpenAI-compatible requests that omit a model.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A TEI-compatible client can use any served model by changing
  only its base URL — with zero changes to request bodies, headers, or query
  parameters.
- **SC-002**: With N configured models (verified for at least N = 3), 100% of
  models respond correctly on their per-model embedding and info endpoints.
- **SC-003**: Per-model endpoint responses are schema-identical to the
  existing TEI endpoints, so existing client tooling works without
  modification.
- **SC-004**: Per-model embedding endpoints deliver latency and throughput
  within 10% of the existing default embedding endpoint under equivalent load.
- **SC-005**: 100% of requests carrying the retired model qualifier — on root
  and per-model endpoints — receive the documented explicit error (no silent
  fallback to a default), and the 0.5 release publishes a migration note
  clearly flagging the broken contract.

## Assumptions

- The root `/embed` and `/info` endpoints remain, bound to the configured
  default model; this preserves the classic single-model TEI deployment shape
  and is how TEI clients configured against the service root keep working.
- An unknown model in a per-model path yields a `404 Not Found` style client
  error with the standard error body (the requested resource — that model's
  endpoint — does not exist), rather than the `400` used for unknown models
  selected via the now-retired qualifier.
- Per-model paths use the alias-or-last-segment path identifier only; the
  full hierarchical model identifier is not accepted in per-model paths, and
  path identifier conflicts are caught at startup rather than at request time.
- The per-model info endpoints are the resolution for issue #105: every
  model's metadata becomes discoverable, while root `/info` intentionally
  keeps the single-object TEI shape describing the default model.
- The OpenAI-compatible endpoints (`/v1/embeddings`, `/v1/models`) are
  unchanged by this feature.
- Model loading and registry behavior (which models are served, how they are
  configured) is unchanged; this feature changes only the TEI API surface.
