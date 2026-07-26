# Feature Specification: Multi-Model Serving

**Feature Branch**: `003-multi-model-serving`

**Created**: 2026-07-19

**Status**: Draft

**Input**: User description: "Extend the model serving so multiple models can be served in parallel. Check for compatibility to the different APIs and implement accordingly. E.g. i want to serve a potion code embedding model next to a multilang model (check available models for example)"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Serve Multiple Models Simultaneously (Priority: P1)

As an operator deploying the embedding service, I want to configure and load more than one model at startup so that a single service instance can serve embeddings for different use cases (e.g., multilingual text and code) without running separate containers.

**Why this priority**: Running a single container per model increases operational overhead, memory duplication across infrastructure, and deployment complexity. Multi-model serving reduces these costs while keeping the service lightweight.

**Independent Test**: Start the service with two different model identifiers and verify that both can produce embeddings through the OpenAI-compatible endpoint.

**Acceptance Scenarios**:

1. **Given** the service is configured with two valid model identifiers, **When** it initializes, **Then** both models are loaded and the readiness endpoint reports the service is ready.
2. **Given** the service is configured with a multilingual model and a code model, **When** text suitable for each model is submitted, **Then** embeddings from the correct model are returned.
3. **Given** one configured model fails to load, **When** the service starts, **Then** the failure is reported clearly and the remaining healthy models continue to serve requests.

---

### User Story 2 - OpenAI-Compatible Model Selection (Priority: P1)

As an application developer using an OpenAI-compatible client, I want to list available models and select one by name in the embeddings request so that I can switch between models without changing the service URL or client configuration.

**Why this priority**: OpenAI-compatible clients expect `/v1/models` and the `model` field in `/v1/embeddings` to control which model is used. Supporting these standard fields makes the service a drop-in replacement.

**Independent Test**: A standard OpenAI client calls `/v1/models`, selects one of the listed identifiers, and sends it in a `/v1/embeddings` request to receive embeddings from the selected model.

**Acceptance Scenarios**:

1. **Given** multiple models are loaded, **When** a client sends a GET request to `/v1/models`, **Then** the response lists all loaded models with their identifiers and basic metadata.
2. **Given** multiple models are loaded, **When** a client sends a `POST /v1/embeddings` request with a valid `model` value, **Then** the response is produced by the requested model.
3. **Given** a client sends a `POST /v1/embeddings` request without specifying a model, **When** the request is processed, **Then** a configured default model is used.

---

### User Story 3 - Text Embedding Inference (TEI) Compatibility with Multiple Models (Priority: P2)

As an infrastructure engineer using existing TEI client tooling, I want the TEI-compatible endpoints to remain usable when multiple models are loaded so that existing integrations are not broken by the multi-model change.

**Why this priority**: TEI clients often expect a single model per endpoint. The multi-model extension must either preserve that expectation or provide a clear, documented path for selecting a model.

**Independent Test**: A TEI client that worked with the previous single-model version can still retrieve embeddings and model information, or the documented migration path is no more than one configuration change.

**Acceptance Scenarios**:

1. **Given** multiple models are loaded, **When** a TEI client calls the existing embed or info endpoint, **Then** the service either uses a configured default model or returns a clear response indicating how to select a model.
2. **Given** a client follows the documented TEI path for model selection, **When** it requests embeddings, **Then** it receives the correct response for the chosen model.
3. **Given** the chosen TEI compatibility approach requires a breaking change, **When** the change is documented, **Then** a migration note is provided for existing clients.

---

### User Story 4 - Configure and Deploy Multiple Models (Priority: P2)

As a platform engineer, I want to declare multiple models in the service configuration and Helm chart so that I can deploy a multi-model instance in Kubernetes with the same packaging used for single-model deployments.

**Why this priority**: The Helm chart is the primary production deployment method. Multi-model support must be configurable through the same deployment interface or operators will not adopt it.

**Independent Test**: A Helm install with a list of model identifiers and a default model produces a service that exposes all configured models on the OpenAI-compatible endpoints.

**Acceptance Scenarios**:

1. **Given** the service configuration accepts multiple models, **When** an operator supplies two or more model identifiers, **Then** the service loads all of them.
2. **Given** the Helm chart is configured with multiple models, **When** a release is installed, **Then** the deployment passes those models to the service and the readiness endpoint becomes ready.
3. **Given** an operator supplies a single model identifier, **When** the service initializes, **Then** behavior remains backward-compatible with the single-model deployment.

---

### User Story 5 - Per-Model Observability and Errors (Priority: P2)

As an operator, I want metrics and errors to be attributable to individual models so that I can monitor load, latency, and failure rates per model and troubleshoot issues without guessing which model is affected.

**Why this priority**: With multiple models sharing one process, aggregated metrics hide per-model performance and failures. Per-model labels are essential for operational clarity.

**Independent Test**: After sending requests to different models, the metrics endpoint shows distinct counters or histograms for each model, and errors for an unknown model are clearly labeled as such.

**Acceptance Scenarios**:

1. **Given** requests are sent to two different models, **When** the metrics endpoint is called, **Then** request counts and latency distributions are distinguishable by model identifier.
2. **Given** a client requests a model identifier that is not loaded, **When** the request is processed, **Then** the service returns a clear, documented error indicating the model is unknown or unavailable.
3. **Given** a model fails during inference, **When** the error is logged, **Then** the log includes the model identifier and a structured error message without exposing internal paths or secrets.

---

### Edge Cases

- A request references a model identifier that is not configured or has failed to load.
- The service is configured with duplicate model identifiers.
- No model is configured at all.
- The service is configured with a single model (backward-compatibility scenario).
- A model parameter is omitted from a request and no default model is configured.
- Multiple models exhaust available memory or other resources during startup.
- A model loads slowly while others are already ready; readiness should reflect the overall state.
- TEI endpoints are called without specifying a model when multiple models are loaded.
- OpenAI `/v1/models` is called while one model is still loading.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST be configurable with one or more model identifiers at startup.
- **FR-002**: The system MUST load all configured models and make them available for inference.
- **FR-003**: The system MUST expose an OpenAI-compatible `GET /v1/models` endpoint that lists all loaded models with identifiers and basic metadata.
- **FR-004**: The system MUST accept a `model` field in `POST /v1/embeddings` requests and route the request to the corresponding loaded model.
- **FR-005**: The system MUST use a configured default model when the `model` field is omitted from a request.
- **FR-006**: The system MUST return a clear, documented error when a request references an unknown or unavailable model.
- **FR-007**: The system MUST preserve TEI-compatible endpoints or provide a documented migration path when multiple models are loaded.
- **FR-008**: The system MUST expose metrics that are distinguishable by model identifier.
- **FR-009**: The system MUST report readiness that reflects whether the configured models are loaded and healthy.
- **FR-010**: The system MUST keep the Helm chart compatible with both single-model and multi-model configurations.
- **FR-011**: The system MUST support model identifiers such as `minishlab/potion-multilingual-128M` and `minishlab/potion-code-16M-v2` and MUST validate the multi-model feature against these two models.

### Key Entities *(include if feature involves data)*

- **ModelRegistry**: The set of loaded models that are available for inference, keyed by model identifier.
- **ModelIdentifier**: A string that uniquely references a model, such as a Hugging Face Hub repository name or a local path.
- **DefaultModel**: The model identifier that is used when a request does not specify one.
- **EmbeddingRequest**: A request containing input text and an optional model identifier.
- **ModelInfo**: Metadata about a loaded model, including its identifier, maximum input length, and embedding dimension.
- **ModelStatus**: The load/health state of a model (e.g., loading, ready, failed).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A client can list all loaded models via the OpenAI-compatible `/v1/models` endpoint and receive a response in under 1 second.
- **SC-002**: A client can request embeddings from a specific model by including its identifier in the request, and at least 99% of such requests are routed correctly.
- **SC-003**: When no model is specified in the request, the configured default model is used, and the response is indistinguishable from an explicit default-model request to the client.
- **SC-004**: Requests for an unknown or unavailable model return a clear error within 1 second without exposing internal paths or secrets.
- **SC-005**: The metrics endpoint exposes per-model request counts and latency distributions that can be filtered by model identifier.
- **SC-006**: The Helm chart can be configured with multiple models and produces a ready deployment whose health endpoint succeeds.
- **SC-007**: TEI-compatible clients either work without changes or have a documented one-step migration path after multi-model support is enabled.

## Assumptions

- Model identifiers are unique within a single service instance; duplicates are rejected or de-duplicated at startup.
- At least one configured model must be loaded successfully for the service to report ready.
- A default model is either explicitly configured or derived from the first successfully loaded model.
- The service does not dynamically load or unload models at runtime; all models are declared at startup.
- The target model families are static model2vec embedding models, such as the potion multilingual and potion code families.
- API key authentication, when enabled, applies to all models uniformly; per-model authentication is out of scope.
- Memory and CPU resources for multiple models are sized by the operator; the service does not enforce resource limits.
