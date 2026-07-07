# Feature Specification: model2vec-embedding-api

**Feature Branch**: `001-model2vec-embedding-api`

**Created**: 2026-07-07

**Status**: Draft

**Input**: User description: "Build an API backend that can serve lightweight model2vec static embedding models on OpenAI and Text Embedding Inference compatible endpoints and can be used as a lightweigt container. It should support ops features like observability, api key and health endpoints. A helm chart should also be there"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - OpenAI-compatible embeddings (Priority: P1)

As an application developer, I want to request embeddings through an OpenAI-compatible endpoint so that I can replace the OpenAI API with a self-hosted model2vec model without changing my client code.

**Why this priority**: OpenAI compatibility is the primary integration contract and unlocks drop-in replacement for the largest ecosystem of existing clients.

**Independent Test**: A standard OpenAI embeddings client pointed at the service returns a response with the same shape and status code as the OpenAI API.

**Acceptance Scenarios**:

1. **Given** the service is running with a loaded model, **When** a client sends a POST request to the OpenAI-compatible embeddings endpoint with a single string and optional model name, **Then** the service returns a JSON response containing a list with one embedding object.
2. **Given** the service is running with a loaded model, **When** a client sends a batch of strings to the same endpoint, **Then** the service returns one embedding object per input in the same order.
3. **Given** a client sends an unsupported input format (e.g., token arrays), **When** the request is received, **Then** the service returns a clear error response indicating the input is not supported.

---

### User Story 2 - Text Embedding Inference compatibility (Priority: P1)

As an infrastructure engineer, I want to call Text Embedding Inference (TEI) compatible endpoints so that I can reuse existing TEI client libraries and tooling.

**Why this priority**: TEI compatibility allows the service to integrate with the Hugging Face ecosystem and existing Kubernetes serving patterns.

**Independent Test**: A TEI client configured with the service base URL successfully retrieves embeddings and model information.

**Acceptance Scenarios**:

1. **Given** the service is running, **When** a client sends a POST request to the TEI-compatible embed endpoint with a string or list of strings, **Then** the service returns embeddings in the expected TEI response shape.
2. **Given** the service is running, **When** a client sends a GET request to the TEI-compatible model information endpoint, **Then** the service returns metadata about the loaded model (e.g., id, max input length).

---

### User Story 3 - Operational health checks (Priority: P1)

As an operator, I want health and readiness endpoints so that I can determine whether the service is alive and able to serve requests.

**Why this priority**: Health checks are required for container orchestration, load balancer membership, and incident response.

**Independent Test**: A monitoring probe calls the health endpoint and receives a success response when the model is loaded, and a non-success response when the model is not ready.

**Acceptance Scenarios**:

1. **Given** the service has loaded the model successfully, **When** a probe calls the health endpoint, **Then** the service returns a success status within 1 second.
2. **Given** the model failed to load, **When** a probe calls the health endpoint, **Then** the service returns a non-success status that indicates the service is not ready.

---

### User Story 4 - API key authentication (Priority: P1)

As an operator, I want API key authentication so that I can control which clients are allowed to use the embeddings API.

**Why this priority**: Exposing an embeddings service without access control creates abuse and cost risks.

**Independent Test**: A request without or with an invalid API key is rejected, while a request with a valid API key succeeds.

**Acceptance Scenarios**:

1. **Given** authentication is enabled, **When** a request arrives without an API key, **Then** the service returns an unauthorized error.
2. **Given** authentication is enabled, **When** a request arrives with a valid API key, **Then** the service processes it normally.
3. **Given** authentication is disabled, **When** a request arrives without an API key, **Then** the service processes it normally.

---

### User Story 5 - Kubernetes deployment via Helm (Priority: P2)

As a platform engineer, I want a Helm chart so that I can deploy the service to Kubernetes with configurable resources, model source, and secrets.

**Why this priority**: A Helm chart provides a repeatable, versioned, and configurable way to run the containerized service in production.

**Independent Test**: A Helm install command with a model name and API key secret produces a running service accessible inside the cluster.

**Acceptance Scenarios**:

1. **Given** a Helm chart for the service, **When** an operator runs `helm install` with values for model source, replica count, and API key secret, **Then** the chart creates the corresponding Kubernetes deployments, services, and configuration.
2. **Given** a deployed Helm release, **When** the pod starts, **Then** the health endpoint becomes ready after the model is loaded.

---

### User Story 6 - Observability (Priority: P2)

As an operator, I want request metrics and structured logs so that I can monitor request volume, latency, errors, and debug incidents.

**Why this priority**: Observability is required to operate the service at scale and to meet on-call responsibilities.

**Independent Test**: After sending requests, an operator can retrieve metrics and logs that expose request count, latency distribution, and error rate.

**Acceptance Scenarios**:

1. **Given** the service is receiving traffic, **When** an operator calls the metrics endpoint, **Then** the response includes counters and histograms for request count, latency, and error rate.
2. **Given** the service handles a failed request, **When** the logs are inspected, **Then** each log line is structured and includes a request correlation identifier.

---

### Edge Cases

- Empty input list submitted to the embeddings endpoint.
- Batch size exceeds a configured maximum.
- Model name provided in the request does not match the loaded model.
- Request body is malformed JSON.
- API key is present but whitespace-trimmed or malformed.
- Model download or load fails during startup.
- Very large input text exceeds the model's maximum sequence length.
- Encoding format requested is not supported (e.g., base64 vs. float).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST expose an OpenAI-compatible `POST /v1/embeddings` endpoint that accepts a single string or a list of strings.
- **FR-002**: The system MUST expose TEI-compatible endpoints for embeddings and model information.
- **FR-003**: The system MUST load a static model2vec model from a configurable model name or local path at startup.
- **FR-004**: The system MUST support configurable API key authentication that can be enabled or disabled.
- **FR-005**: The system MUST expose a health endpoint that reports whether the model is loaded and the service is ready.
- **FR-006**: The system MUST expose a metrics endpoint that reports request count, latency, and error rate.
- **FR-007**: The system MUST produce structured logs with a correlation identifier per request.
- **FR-008**: The system MUST provide a Helm chart that deploys the containerized service to Kubernetes.
- **FR-009**: The Helm chart MUST allow configuration of model source, replica count, resource limits, and API key secret.
- **FR-010**: The system MUST return clear, documented error responses for invalid input, missing authentication, and model errors.

### Key Entities *(include if feature involves data)*

- **EmbeddingRequest**: Contains input text (single string or list), model identifier, and optional encoding format.
- **EmbeddingResponse**: Contains the list of embedding vectors, model identifier, and usage metadata.
- **ModelInfo**: Contains model identifier, supported input types, and maximum input length.
- **HealthStatus**: Contains overall readiness and a message indicating the current state.
- **APIKey**: A shared secret used to authenticate client requests.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An OpenAI embeddings client can be pointed at the service with only base URL and API key changes and successfully retrieve embeddings.
- **SC-002**: A TEI client can retrieve embeddings and model information from the service without client-side workarounds.
- **SC-003**: The health endpoint returns a success response within 1 second when the model is loaded.
- **SC-004**: The metrics endpoint exposes request count, latency distribution, and error rate in a format consumable by standard monitoring tools.
- **SC-005**: A Helm install with configurable values produces a running service whose health endpoint becomes ready.
- **SC-006**: Requests without a valid API key are rejected when authentication is enabled, and valid requests are processed normally.
- **SC-007**: The container image remains lightweight enough to start and serve requests with modest resource limits (e.g., 1 CPU and 1 GiB memory).

## Assumptions

- The target deployment platform is Kubernetes, and the Helm chart is the primary installation method.
- Static model files or model names are provided at deployment time; the service does not train models.
- API key authentication uses a shared-secret model stored via environment variables or Kubernetes secrets.
- The observability stack consumes metrics in an open standard format and structured logs in JSON.
- Token-array input is out of scope for the initial release and will return a clear error.
