# Feature Specification: Docker Compose Support

**Feature Branch**: `006-docker-compose-support`

**Created**: 2026-08-31

**Status**: Draft

**Input**: User description: "Add fully fledged docker compose support with docs, a compose file with 2 models (multilingual and code v2) and volume mounting and reference in the readme"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Launch the service locally with two models using one command (Priority: P1)

A developer who wants to try or demonstrate the embedding service runs a single
launch command and gets a running service that serves both the multilingual
model (`minishlab/potion-multilingual-128M`) and the code model
(`minishlab/potion-code-16M-v2`) from the published container image. The
developer does not need to build anything, write configuration files, or read
the Helm documentation.

**Why this priority**: One-command startup is the core value of this feature.
Without a working launch path that serves both models, nothing else in this
feature matters.

**Independent Test**: Can be fully tested by running the provided launch
command from a clean checkout on a machine with the container runtime
installed, then querying the service for its model list and embeddings; it
delivers a two-model embedding service without any manual steps.

**Acceptance Scenarios**:

1. **Given** a clean environment with the container runtime installed, **When**
   the developer runs the provided compose launch command, **Then** the service
   starts, downloads both models, and becomes ready without any additional
   configuration.
2. **Given** a running compose deployment, **When** the developer requests the
   model list, **Then** both the multilingual model and the code v2 model are
   listed.
3. **Given** a running compose deployment, **When** the developer requests
   embeddings while naming either model, **Then** valid embeddings are returned
   for the requested model.
4. **Given** the compose deployment is running, **When** the developer checks
   the operational endpoints (health, readiness, metrics), **Then** all respond
   successfully without authentication.

---

### User Story 2 - Persist downloaded models across restarts via volume mounting (Priority: P2)

A developer restarting the compose deployment (or rebooting their machine) does
not want to re-download the models every time. A host directory is mounted into
the container so downloaded model artifacts survive container recreation, and
subsequent startups are fast and work offline.

**Why this priority**: Repeated downloads waste time and bandwidth and make the
setup unusable offline; persistence is the second most visible quality-of-life
improvement and was explicitly requested.

**Independent Test**: Can be fully tested by starting the deployment once,
noting the download time, tearing it down, starting it again with the network
disconnected, and verifying the service still becomes ready quickly; it
delivers restart persistence and offline start capability.

**Acceptance Scenarios**:

1. **Given** a first-time launch, **When** the deployment is torn down and
   started again, **Then** the models are not re-downloaded from the network.
2. **Given** a host directory mounted per the documented configuration, **When**
   the developer inspects that directory after the first launch, **Then** the
   downloaded model artifacts are visible on the host.
3. **Given** the deployment has been started once, **When** the developer
   starts it again with no network access, **Then** the service still becomes
   ready and serves both models.
4. **Given** a developer who prefers to manage model storage themselves, **When**
   they change the documented storage setting to a different host path (or a
   named volume), **Then** the service uses that location without other changes.

---

### User Story 3 - Customize the deployment without editing the compose file (Priority: P3)

A developer adapting the compose setup for their environment (different host
port, restricted memory, API-key protection, a different default model) can do
so through the documented environment variables and compose mechanisms, without
editing the compose file itself.

**Why this priority**: Customization makes the setup "fully fledged" rather
than a fixed demo, but the default path already works without it.

**Independent Test**: Can be fully tested by overriding each documented setting
one at a time (port, API key, model selection) and verifying the service
reflects each override; it delivers an adaptable local deployment.

**Acceptance Scenarios**:

1. **Given** the documented port override is set, **When** the deployment
   starts, **Then** the service is reachable on the overridden port instead of
   the default.
2. **Given** an API key is configured via the documented mechanism, **When** a
   client calls an embedding endpoint without the key, **Then** the request is
   rejected, and with the correct key it succeeds; operational endpoints
   (health, readiness, metrics) remain accessible without the key.
3. **Given** a developer changes the model selection via the documented
   mechanism, **When** the deployment starts, **Then** the service serves the
   selected model set and the configured default model answers
   requests that omit a model name.

---

### User Story 4 - Discover and follow the compose documentation (Priority: P3)

A developer arriving at the project's README finds a pointer to Docker Compose
usage, and the documentation site contains a dedicated page that explains
prerequisites, the launch command, the two served models, volume mounting,
customization options, and how this setup relates to the existing Kubernetes
deployment path.

**Why this priority**: Documentation was explicitly requested and is what makes
the feature approachable, but it has value only once the compose setup itself
works.

**Independent Test**: Can be fully tested by reading the README from a fresh
visitor's perspective, following the link to the compose documentation, and
executing every documented command successfully; it delivers a self-service
onboarding path.

**Acceptance Scenarios**:

1. **Given** a fresh visitor reads the project README, **When** they look for a
   local deployment option, **Then** they find a reference to Docker Compose
   with a link to the detailed documentation.
2. **Given** the documentation site, **When** the developer opens the compose
   documentation page, **Then** it covers prerequisites, launch, both models,
   volume mounting, customization, and tear-down with copy-pasteable commands.
3. **Given** the documented commands on the documentation page, **When** the
   developer executes them in order, **Then** each succeeds as described.

---

### Edge Cases

- What happens when the model download fails at startup (e.g., no network, Hub
  unreachable, model name misspelled)? The container must fail with a clear log
  message rather than serve an incomplete model set.
- What happens when the mounted storage location is not writable by the
  container? The failure must be surfaced clearly in logs at startup instead of
  silently re-downloading or crashing later.
- What happens when a developer starts the deployment while the host port is
  already in use? The failure must be immediate and understandable.
- What happens when the container is stopped? The service must shut down
  gracefully (in-flight requests finish or the runtime's stop timeout applies),
  and restarts must be automatic after a crash or host reboot when configured
  to do so.
- What happens when a developer pulls a newer version of the image? The
  deployment must continue to work with the persisted model storage, and the
  documentation must state any caveats about reusing cached models across
  versions.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The repository MUST include a ready-to-use compose deployment
  file that, by default, serves exactly two models: the multilingual model
  (`minishlab/potion-multilingual-128M`) and the code model
  (`minishlab/potion-code-16M-v2`).
- **FR-002**: The compose deployment MUST run the published service container
  image (not a locally built one) so a fresh checkout needs no build step.
- **FR-003**: The compose deployment MUST mount a host directory into the
  container for model storage, so downloaded model artifacts persist across
  container recreation and restarts.
- **FR-004**: The compose deployment MUST allow the model storage location to
  be changed (different host path or named volume) through documented settings.
- **FR-005**: The compose deployment MUST expose the service on a documented
  default host port, overridable without editing the compose file.
- **FR-006**: The compose deployment MUST support enabling API-key protection
  for the embedding endpoints via a documented setting, keeping health,
  readiness, and metrics endpoints public, consistent with existing
  authentication behavior.
- **FR-007**: The compose deployment MUST select the multilingual model as the
  default model, so requests that omit a model name are served by it.
- **FR-008**: The compose deployment MUST define a health check so the runtime
  can report service health, and a restart policy so the service recovers from
  crashes and host reboots without manual intervention.
- **FR-009**: The compose deployment MUST forward all existing service
  configuration options relevant to local use (models, default model, port,
  API key, request limits) through documented environment variables or
  equivalent settings.
- **FR-010**: The repository documentation (README) MUST reference the compose
  option for local deployment with a pointer to the detailed documentation page.
- **FR-011**: The documentation site MUST include a dedicated compose page
  covering prerequisites, launch, the two served models, volume mounting,
  customization, tear-down, and the relationship to the Kubernetes deployment
  path.
- **FR-012**: Every command shown in the compose documentation MUST be
  executable as written on a machine that satisfies the documented
  prerequisites.
- **FR-013**: The compose deployment MUST stop cleanly in response to the
  runtime's stop signal, without corrupting the persisted model storage.
- **FR-014**: The compose setup MUST be kept consistent with the service's
  existing error responses and API contracts; no new endpoints or response
  shapes are introduced by this feature.

### Key Entities *(include if feature involves data)*

- **Compose deployment definition**: The single source of truth describing how
  the service container is launched locally — image reference, served model
  set, default model, host port mapping, model storage mount, health check,
  restart policy, and customization settings.
- **Model storage location**: The host-side directory (or named volume) that
  holds downloaded model artifacts; persists across restarts; created
  automatically on first launch.
- **Served model set**: The two models exposed by the default deployment —
  `minishlab/potion-multilingual-128M` (default) and
  `minishlab/potion-code-16M-v2` — matching the service's multi-model
  capability.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer with the container runtime installed can go from
  cloning the repository to receiving embeddings for both models in under 5
  minutes, excluding model download time.
- **SC-002**: A second launch of the same deployment completes startup at least
  3× faster than the first launch (no model downloads) and succeeds with no
  network access.
- **SC-003**: 100% of the commands in the compose documentation succeed when
  executed verbatim on a clean machine that meets the documented prerequisites.
- **SC-004**: A first-time reader locates the compose deployment option in the
  README in under 30 seconds and can reach the detailed documentation in one
  click.
- **SC-005**: Overriding each documented customization setting (port, API key,
  model selection, storage location) works on the first attempt without editing
  the compose file, verified for every documented setting.

## Assumptions

- The target audience is developers running the service on a local machine or
  single host for evaluation, demos, and development; production-like
  deployments remain the domain of the existing Helm chart.
- The published container image from the project's existing registry is used;
  building a local image is out of scope for the default path (though the
  documentation may mention how to substitute a locally built image).
- The two default models match those already used in the project's docs and
  tests: `minishlab/potion-multilingual-128M` (default) and
  `minishlab/potion-code-16M-v2`.
- The service's existing behavior (multi-model serving, optional API key,
  public operational endpoints, error contract) is reused unchanged; this
  feature only packages and documents a deployment method.
- The model storage default is a host directory inside the repository checkout
  (so it is obvious and cleanable), overridable to any path or named volume.
- Compose file format versioning follows current tooling conventions (no
  obsolete top-level version key requirements).
