# Feature Specification: Default Model Update and Spec Drift Check

**Feature Branch**: `002-default-model-multilingual`

**Created**: 2026-07-18

**Status**: Draft

**Input**: User description: "The default model should be potion-128m-multilingual. Also check that there is no spec drift in the recent changes for HF Hub crate and update spec if needed."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Multilingual model as default (Priority: P1)

As an operator deploying the embedding service, I want the server to default to a multilingual model so that out-of-the-box deployments can handle text in many languages without requiring me to research and configure a specific model identifier.

**Why this priority**: The previous default model was English-centric. Making the multilingual variant the default lowers the barrier to entry for international use cases and aligns the service with the most broadly useful model in the model2vec family.

**Independent Test**: Start the service without providing a model identifier and verify that it loads `minishlab/potion-multilingual-128M` and serves embeddings.

**Acceptance Scenarios**:

1. **Given** the service is started with no explicit model identifier, **When** it initializes, **Then** it loads `minishlab/potion-multilingual-128M` and becomes ready.
2. **Given** the service is deployed via Helm without setting a model value, **When** the pod starts, **Then** it loads `minishlab/potion-multilingual-128M` by default.
3. **Given** a client sends text in a non-English language to the default deployment, **When** the embeddings endpoint is called, **Then** the service returns valid embeddings.

---

### User Story 2 - Consistent documentation examples (Priority: P1)

As a new user reading the project documentation, I want all examples and defaults to reference the same model so that I can copy commands confidently without encountering conflicting information.

**Why this priority**: Inconsistent examples create confusion, increase support burden, and make the project look unmaintained.

**Independent Test**: Search the documentation and README for references to the old default model and confirm they have been updated or intentionally preserved as non-default examples.

**Acceptance Scenarios**:

1. **Given** the README and agent guide list quick-start commands, **When** a user reads them, **Then** they reference `minishlab/potion-multilingual-128M` as the default model.
2. **Given** the Helm chart documentation and `values.yaml` describe the default model, **When** an operator reviews them, **Then** they show `minishlab/potion-multilingual-128M`.
3. **Given** the project documentation site includes configuration examples, **When** a user follows them, **Then** the examples consistently use the new default model.

---

### User Story 3 - Accurate specification of Hugging Face Hub integration (Priority: P2)

As a maintainer, I want the architecture spec and research notes to match the actual Hugging Face Hub integration so that future contributors do not make decisions based on outdated assumptions.

**Why this priority**: The project recently upgraded its Hugging Face Hub integration, which changed the patterns used in tests and benchmarks. If the spec still describes the old approach, it becomes a source of drift and incorrect decisions.

**Independent Test**: Compare the current Hugging Face Hub usage in tests and benchmarks against the research/specification documents and verify they describe the same patterns.

**Acceptance Scenarios**:

1. **Given** the codebase uses the current Hugging Face Hub integration in tests and benchmarks, **When** the research document is reviewed, **Then** it accurately reflects the current integration approach or notes that direct Hugging Face Hub usage is limited to test fixtures.
2. **Given** the specification mentions model loading from Hugging Face Hub or local paths, **When** it is compared with the implementation, **Then** the described behavior matches the actual model loading path.
3. **Given** drift is found between the spec and the code, **When** the spec is updated, **Then** the corrected spec is committed and the discrepancy is resolved.

---

### Edge Cases

- A user explicitly provides the old default model identifier and expects it to keep working.
- A user deploys with a local model path, bypassing the default entirely.
- The new default model is temporarily unavailable from Hugging Face Hub; startup should produce a clear error.
- Test fixtures continue to use a small, fast model to keep CI quick even though the production default is larger.
- Documentation examples that intentionally demonstrate using a specific (non-default) model are preserved but labeled accordingly.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST use `minishlab/potion-multilingual-128M` as the default model identifier when no model is supplied by the user.
- **FR-002**: The CLI, Helm chart default values, and runtime configuration MUST treat `minishlab/potion-multilingual-128M` as the fallback model.
- **FR-003**: All public-facing documentation examples (README, AGENTS.md, VitePress docs, Helm chart README) that previously presented `minishlab/potion-base-2M` as the default MUST be updated to `minishlab/potion-multilingual-128M`.
- **FR-004**: Any documentation examples that intentionally illustrate specifying a non-default model MUST remain valid and be clearly distinguishable from default examples.
- **FR-005**: The existing specification and research documents MUST be audited for drift against the current Hugging Face Hub integration.
- **FR-006**: If the audit finds outdated descriptions of the Hugging Face Hub integration or model-loading behavior, the specification and/or research documents MUST be updated to accurately describe the current state.
- **FR-007**: The change MUST NOT break existing behavior for users who explicitly provide a model identifier or a local model path.

### Key Entities *(include if feature involves data)*

- **DefaultModel**: The model identifier used by the service when the user does not supply one. After this change it is `minishlab/potion-multilingual-128M`.
- **ModelIdentifier**: A string referencing either a Hugging Face Hub repository or a local filesystem path; configurable by the user and independent of the default.
- **TestFixtureModel**: A small, fast model used by automated tests and benchmarks to keep CI quick; distinct from the production default.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Starting the service without a model identifier results in `minishlab/potion-multilingual-128M` being loaded, verified by the readiness endpoint returning success.
- **SC-002**: 100% of quick-start examples in README, AGENTS.md, and VitePress docs that describe the default model reference `minishlab/potion-multilingual-128M`.
- **SC-003**: The Helm chart's default `model` value and its README both reference `minishlab/potion-multilingual-128M`.
- **SC-004**: The test suite passes after the default change, with no unintended use of the new larger default model in performance-sensitive test fixtures.
- **SC-005**: The specification and research documents describing Hugging Face Hub integration contain no statements that contradict the current integration patterns used in the codebase.
- **SC-006**: If spec drift is found, the affected spec/research files are updated and the audit results are recorded.

## Assumptions

- `minishlab/potion-multilingual-128M` is publicly available on Hugging Face Hub and is compatible with the current inference crate.
- The production default can be a larger model than the small fixture used in tests; tests will continue to use a fast, small model to keep CI times low.
- The Hugging Face Hub integration is stable enough that the audit only needs to reconcile existing documentation, not redesign integration.
- Users who explicitly set a model identifier or local path are unaffected by the default change.
- Documentation examples that demonstrate non-default model selection remain useful and do not need to be removed.
