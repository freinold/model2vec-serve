# Feature Specification: Helm Chart Enhancements

**Feature Branch**: `004-helm-chart-enhancements`

**Created**: 2026-08-19

**Status**: Draft

**Input**: User description: "Implement open issues #96 (publish helm chart), #97 (add pvc template), and #98 (add route template → confirmed as Kubernetes Ingress with optional extraLabels) according to the agreed plan, mirroring the it-at-m/helm-charts release flow (chart-releaser + OCI push to ghcr.io)."

## Clarifications

### Session 2026-08-19

- Q: Which tooling should the new chart linting & testing story standardize on? → A: Adopt helm/chart-testing (ct lint + ct install in an ephemeral kind cluster) mirroring the it-at-m reference repository, while keeping the existing bash template tests for value-specific assertions.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Install the Chart from a Published Registry (Priority: P1)

As an operator deploying model2vec-serve on Kubernetes, I want to install the Helm chart directly from a public OCI registry without cloning the source repository, so that I can consume it in standard deployment pipelines and receive versioned chart releases.

**Why this priority**: Today the chart can only be installed from a local checkout of the repository, which blocks automated GitOps/CI consumption and external adoption. Publishing versioned chart artifacts is the enabler for every other chart improvement to reach users.

**Independent Test**: On a fresh machine without the repository, add no local files and install the chart using only an OCI registry reference; the release deploys and the pod becomes ready.

**Acceptance Scenarios**:

1. **Given** the chart has been published, **When** an operator runs an install command referencing only the OCI registry URL and a chart version, **Then** the chart installs successfully with default values and the pod becomes ready.
2. **Given** a change to the chart sources is merged to the main branch with an incremented chart version, **When** the publishing automation completes, **Then** the new chart version is available in the registry and as a downloadable release asset.
3. **Given** chart sources change without a version increment, **When** the publishing automation runs, **Then** no duplicate release is created and the run does not fail.
4. **Given** a published chart version, **When** it is installed with default values, **Then** the deployed container image reference resolves to an image that actually exists in the registry.

---

### User Story 2 - Persist Model Files on a Volume (Priority: P2)

As an operator, I want the chart to optionally create and mount a persistent volume for model files, so that downloaded models survive pod restarts and reschedules instead of being re-downloaded from the model hub every time.

**Why this priority**: Every pod restart currently re-downloads model files (hundreds of MB), causing slow cold starts, wasted bandwidth, and a hard dependency on hub availability at startup. Persistence removes all three pain points and additionally enables pre-staged or air-gapped model delivery.

**Independent Test**: Enable persistence, install the chart, let the model download, delete the pod, and verify the replacement pod reaches readiness without re-downloading the model.

**Acceptance Scenarios**:

1. **Given** persistence is enabled with a size and optional storage class, **When** the chart is installed, **Then** a persistent volume claim is created and mounted into the serving pod at the configured mount path.
2. **Given** persistence is enabled, **When** the pod starts, **Then** model downloads are directed to the mounted volume so they persist across restarts.
3. **Given** persistence is enabled with an existing claim name, **When** the chart is installed, **Then** no new claim is created and the deployment references the provided claim.
4. **Given** persistence is disabled (default), **When** the chart is installed, **Then** no claim, volume, or mount is added and the rendered resources are unchanged from before this feature.
5. **Given** persistence is enabled and the operator supplies their own cache-location environment variable, **When** the chart is installed, **Then** the operator's value takes precedence over the chart-provided default.

---

### User Story 3 - Expose the Service Through an Ingress (Priority: P3)

As an operator, I want the chart to optionally create a Kubernetes Ingress with configurable host, paths, TLS, annotations, and additional labels, so that the embedding API is reachable from outside the cluster through our standard ingress controller without hand-written manifests.

**Why this priority**: The chart currently only creates a ClusterIP service, forcing operators to maintain their own ingress manifests. A built-in, disabled-by-default ingress template covers the standard exposure pattern while staying out of the way of existing installs.

**Independent Test**: Enable the ingress with a host name, install the chart on a cluster with an ingress controller, and verify the API answers requests sent to that host.

**Acceptance Scenarios**:

1. **Given** the ingress is enabled with a host, **When** the chart is installed, **Then** an Ingress resource is created routing that host's traffic to the chart's service port.
2. **Given** the ingress is enabled with TLS configuration, **When** the chart is installed, **Then** the Ingress carries the TLS block referencing the configured secret.
3. **Given** the ingress is enabled with additional labels, **When** the chart is installed, **Then** the Ingress metadata contains both the standard chart labels and the additional labels.
4. **Given** the ingress is disabled (default), **When** the chart is installed, **Then** no Ingress resource is rendered.

---

### User Story 4 - Automated Chart Linting and Install Testing (Priority: P2)

As a maintainer reviewing chart changes, I want every pull request that touches the chart to be linted and install-tested automatically, so that broken or unversioned charts never reach the main branch or the published registry.

**Why this priority**: The publishing flow (P1) releases whatever lands on the main branch, so quality gates must exist before publishing can be trusted. Linting also enforces the version-increment rule that publishing depends on, and an install smoke test catches errors that static rendering cannot.

**Independent Test**: Open a pull request with a deliberately broken chart template and verify that the checks fail and block merge; open a pull request with a valid chart change and verify linting and an install smoke test pass.

**Acceptance Scenarios**:

1. **Given** a pull request changes chart sources, **When** checks run, **Then** the chart is linted and the lint fails if the chart version was not incremented.
2. **Given** a pull request changes chart sources, **When** checks run, **Then** the chart is installed into an ephemeral cluster and the check fails if the release does not become ready.
3. **Given** a pull request that only changes documentation or other non-chart files, **When** checks run, **Then** no chart linting or install testing is triggered.
4. **Given** a chart change that cannot be install-tested (for example a documentation-only chart change), **When** the change is marked with the documented opt-out, **Then** the install test is skipped while linting still runs.
5. **Given** the existing template-rendering assertions, **When** any chart change is made, **Then** those assertions continue to run and guard value-specific behavior.

---

### Edge Cases

- Persistence enabled with an existing claim: no claim is created; missing or misnamed claims surface as standard Kubernetes pending-pod errors, not chart errors.
- Persistence enabled with an empty storage class: the cluster's default storage class is used.
- Persistence enabled but the operator overrides the cache location via custom environment variables: the operator's value wins.
- Ingress enabled without TLS: the Ingress serves plain HTTP through the controller.
- Ingress enabled with empty additional labels: only the standard chart labels are applied.
- A chart change merged without a version increment: publishing is skipped; the previous release remains the latest. (Chart linting prevents this case from reaching the main branch.)
- A published chart version is never overwritten or mutated after release.
- A pull request that changes only non-chart files triggers no chart linting or install testing.
- A documentation-only chart change can skip the install smoke test via a documented opt-out marker, while linting still applies.
- An install test whose pod never becomes ready (for example because the referenced image cannot be pulled) fails the check instead of passing silently.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The chart MUST be published to a public OCI registry so it can be installed using only a registry reference and version, without a local copy of the repository.
- **FR-002**: Chart publishing MUST run automatically when chart sources change on the main branch and MUST also be triggerable manually.
- **FR-003**: Each chart release MUST be uniquely versioned; re-running publishing for an already-released version MUST be a no-op that does not fail or overwrite the existing release.
- **FR-004**: Each chart release MUST also produce a downloadable packaged-chart release asset alongside the OCI artifact.
- **FR-005**: The chart's default image repository and tag MUST resolve to a real, published container image so a default installation works out of the box.
- **FR-006**: The chart MUST provide an optional, disabled-by-default persistence configuration that creates a persistent volume claim with configurable size, storage class, access modes, mount path, and annotations.
- **FR-007**: The persistence configuration MUST support referencing an existing claim, in which case no new claim is created.
- **FR-008**: When persistence is enabled, the deployment MUST automatically mount the claim at the configured path and direct model downloads to that path, without requiring operators to wire extra volumes or mounts.
- **FR-009**: Operator-supplied environment variables MUST take precedence over chart-provided defaults for the model cache location.
- **FR-010**: The chart MUST provide an optional, disabled-by-default ingress configuration producing a standard Kubernetes Ingress with configurable ingress class, annotations, hosts, paths, and TLS.
- **FR-011**: The ingress configuration MUST support arbitrary additional labels that are merged with the standard chart labels on the Ingress resource.
- **FR-012**: The ingress MUST route traffic to the chart's service on the HTTP port.
- **FR-013**: All new chart behavior MUST be covered by automated lint and template-rendering tests that run in CI.
- **FR-014**: All new configuration values and the registry-based install command MUST be documented in the chart README and the documentation site.
- **FR-015**: Every pull request that changes chart sources MUST be linted automatically, and the lint MUST fail when the chart version was not incremented relative to the main branch.
- **FR-016**: Every pull request that changes chart sources MUST pass an install smoke test that deploys the chart into an ephemeral cluster and verifies the release becomes ready; a documented opt-out marker MUST be available for changes that cannot be installed.
- **FR-017**: Chart quality checks MUST NOT run for pull requests that change no chart sources.
- **FR-018**: The existing template-rendering test assertions MUST be preserved and extended to cover the new persistence and ingress resources.

### Key Entities

- **Chart Release**: A versioned, immutable package of the Helm chart, distributed as an OCI registry artifact and as a downloadable release asset; references a compatible container image version.
- **Persistence Configuration**: Operator-facing settings controlling model-file storage: enabled flag, claim size, storage class, access modes, mount path, annotations, and an optional existing claim reference.
- **Ingress Configuration**: Operator-facing settings controlling external exposure: enabled flag, ingress class, annotations, additional labels, host/path rules, and TLS settings.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator can install a working deployment on a fresh cluster with a single registry-referencing command, with no local copy of the repository, reaching ready state in one attempt.
- **SC-002**: With persistence enabled, a pod that is deleted after initial model download returns to ready state without re-downloading model files, measurably shortening restart time compared to a non-persistent pod.
- **SC-003**: With ingress enabled, 100% of API requests sent to the configured host reach the service without port-forwarding or manual networking steps.
- **SC-004**: After merging a versioned chart change, the new chart version is installable from the registry within 5 minutes of automation completion.
- **SC-005**: Existing installations that do not opt into the new options produce exactly the same rendered resources as before this feature (zero forced changes on upgrade).
- **SC-006**: 100% of pull requests with broken or unversioned chart changes are blocked from merging by automated checks before they can reach the main branch.
- **SC-007**: Chart quality checks add no more than 10 minutes to pull-request feedback time, and are skipped entirely for changes that do not touch chart sources.

## Assumptions

- The target OCI registry is the project's existing container registry namespace (ghcr.io), already used for the Docker image; chart artifacts live under the repository's registry path.
- The packaged chart is additionally attached to a GitHub Release per version, mirroring the reference flow of the it-at-m/helm-charts repository.
- GitHub Pages remains dedicated to the documentation site; a classic web-served chart repository index is not required since OCI is the primary distribution channel.
- The registry package for the chart will be marked public once by a maintainer; this one-time administrative step is outside the chart's scope.
- Chart version increments are a manual step in the pull request that changes the chart; the automation releases only on version changes.
- An ingress controller is present in the target cluster; installing and operating the controller is the operator's responsibility.
- Single-replica deployments with read-write-once storage are the default persistence shape; shared multi-replica storage is out of scope.
- Chart linting and install testing run on the standard CI infrastructure, which can provision an ephemeral cluster on demand; install tests use default chart values against publicly pullable container images.
- The chart-testing lint and install checks complement, not replace, the existing bash lint and template-rendering test scripts.
