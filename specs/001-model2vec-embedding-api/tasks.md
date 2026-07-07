---

description: "Task list template for feature implementation"
---

# Tasks: model2vec-embedding-api

**Input**: Design documents from `/specs/001-model2vec-embedding-api/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are MANDATORY unless explicitly exempted. Every behavior-changing task MUST include at least one test, written first and failing before implementation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- This feature uses a single Rust binary crate with modules under `src/` and a Helm chart under `helm/model2vec-serve/`

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Create Rust project files `Cargo.toml`, `src/main.rs`, and workspace layout
- [x] T002 Add runtime dependencies to `Cargo.toml` (`axum`, `utoipa`, `utoipa-scalar`, `model2vec-rs`, `tokio`, `serde`, `clap`, `tracing`, `tracing-subscriber`, `metrics`, `metrics-exporter-prometheus`, `tower-http`)
- [x] T003 [P] Configure linting and formatting with `rustfmt.toml` and `clippy` CI checks
- [x] T004 Create multi-stage `Dockerfile` for the service
- [x] T005 Create directory layout (`src/routes/`, `src/model/`, `tests/contract/`, `tests/integration/`, `tests/unit/`, `helm/model2vec-serve/`)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T006 Define `Config` struct and CLI argument parsing in `src/config.rs`
- [x] T007 Implement `AppError` enum and `IntoResponse` mapping in `src/errors.rs`
- [x] T008 Set up structured logging, request correlation IDs, and tracing subscriber in `src/telemetry.rs`
- [x] T009 Create shared `AppState` in `src/state.rs`
- [x] T010 [P] Create model2vec inference wrapper in `src/model/embedding.rs`
- [x] T011 Implement model loading at startup in `src/main.rs`
- [x] T012 Set up axum router skeleton and Tower middleware layers in `src/routes/mod.rs`
- [x] T013 [P] Create shared request/response DTOs in `src/routes/dto.rs`

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - OpenAI-compatible embeddings (Priority: P1) 🎯 MVP

**Goal**: Expose a `POST /v1/embeddings` endpoint that returns OpenAI-compatible embeddings for single strings or batches.

**Independent Test**: A standard OpenAI client pointed at the service retrieves embeddings with the expected response shape.

### Tests for User Story 1 (MANDATORY) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T014 [P] [US1] Contract test for `/v1/embeddings` response shape in `tests/contract/openai.rs`
- [x] T015 [P] [US1] Integration test for single and batch embedding requests in `tests/integration/openai.rs`

### Implementation for User Story 1

- [x] T016 [P] [US1] Create OpenAI request/response types in `src/routes/dto.rs`
- [x] T017 [US1] Implement `/v1/embeddings` handler in `src/routes/embeddings.rs`
- [x] T018 [US1] Add input validation (empty input, encoding format, model name) in `src/routes/embeddings.rs`
- [x] T019 [US1] Wire OpenAI route into `src/routes/mod.rs`

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Text Embedding Inference compatibility (Priority: P1)

**Goal**: Expose TEI-compatible `/embed` and `/info` endpoints.

**Independent Test**: A TEI client configured with the service base URL retrieves embeddings and model metadata.

### Tests for User Story 2 (MANDATORY) ⚠️

- [x] T020 [P] [US2] Contract test for TEI `/embed` and `/info` in `tests/contract/tei.rs`
- [x] T021 [P] [US2] Integration test for TEI endpoints in `tests/integration/tei.rs`

### Implementation for User Story 2

- [x] T022 [P] [US2] Create TEI request/response types in `src/routes/tei.rs`
- [x] T023 [US2] Implement TEI `/embed` handler in `src/routes/tei.rs`
- [x] T024 [US2] Implement TEI `/info` handler in `src/routes/tei.rs`
- [x] T025 [US2] Wire TEI routes into `src/routes/mod.rs`

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Operational health checks (Priority: P1)

**Goal**: Expose `/health` and `/ready` endpoints that reflect model load status.

**Independent Test**: A monitoring probe receives success when the model is loaded and non-success when it is not.

### Tests for User Story 3 (MANDATORY) ⚠️

- [x] T026 [P] [US3] Contract test for `/health` response in `tests/contract/health.rs`
- [x] T027 [P] [US3] Integration test for `/health` and `/ready` in `tests/integration/health.rs`

### Implementation for User Story 3

- [x] T028 [US3] Implement `HealthStatus` response and readiness logic in `src/routes/health.rs`
- [x] T029 [US3] Wire health routes into `src/routes/mod.rs`

**Checkpoint**: At this point, User Stories 1, 2, AND 3 should all work independently

---

## Phase 6: User Story 4 - API key authentication (Priority: P1)

**Goal**: Protect embeddings endpoints with configurable API key authentication.

**Independent Test**: Requests without or with an invalid API key are rejected; valid requests succeed.

### Tests for User Story 4 (MANDATORY) ⚠️

- [x] T030 [P] [US4] Contract test for unauthorized requests in `tests/contract/auth.rs`
- [x] T031 [P] [US4] Integration test for API key middleware in `tests/integration/auth.rs`

### Implementation for User Story 4

- [x] T032 [US4] Implement API key Tower layer in `src/auth.rs`
- [x] T033 [US4] Apply auth layer to embeddings endpoints in `src/routes/mod.rs`

**Checkpoint**: At this point, User Stories 1 through 4 should all work independently

---

## Phase 7: User Story 5 - Kubernetes deployment via Helm (Priority: P2)

**Goal**: Provide a configurable Helm chart for Kubernetes deployment.

**Independent Test**: `helm install` produces a running service with configurable model source and API key secret.

### Tests for User Story 5 (MANDATORY) ⚠️

- [ ] T034 [P] [US5] Helm template validation tests in `helm/model2vec-serve/tests/` or `tests/helm/template_test.sh`
- [ ] T035 [US5] Helm lint check for the chart

### Implementation for User Story 5

- [x] T036 [P] [US5] Create Helm chart structure (`Chart.yaml`, `values.yaml`, `templates/`) in `helm/model2vec-serve/`
- [x] T037 [US5] Implement `Deployment` template with CLI arg mapping in `helm/model2vec-serve/templates/deployment.yaml`
- [x] T038 [US5] Implement `Service` template in `helm/model2vec-serve/templates/service.yaml`
- [x] T039 [US5] Implement `Secret` and `ConfigMap` templates in `helm/model2vec-serve/templates/`
- [x] T040 [US5] Add `values.yaml` with documented defaults for model source, API key, replicas, resources, and volume mounts
- [x] T041 [US5] Add `README.md` and `NOTES.txt` to `helm/model2vec-serve/`

**Checkpoint**: At this point, the Helm chart can deploy the service independently

---

## Phase 8: User Story 6 - Observability (Priority: P2)

**Goal**: Expose Prometheus metrics and structured request logs with correlation IDs.

**Independent Test**: Metrics endpoint returns request count, latency, and error rate; logs contain correlation IDs.

### Tests for User Story 6 (MANDATORY) ⚠️

- [x] T042 [P] [US6] Contract test for `/metrics` format in `tests/contract/metrics.rs`
- [x] T043 [P] [US6] Integration test for metrics and logs in `tests/integration/observability.rs`

### Implementation for User Story 6

- [x] T044 [US6] Add Prometheus metrics recorder in `src/telemetry.rs`
- [x] T045 [US6] Implement `/metrics` handler in `src/routes/metrics.rs`
- [x] T046 [US6] Add request logging layer with correlation IDs in `src/telemetry.rs`
- [x] T047 [US6] Wire metrics route into `src/routes/mod.rs`

**Checkpoint**: All user stories should now be independently functional

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T048 [P] Update `README.md` and `docs/` with usage and deployment instructions
- [x] T049 Code cleanup and clippy fixes across `src/`
- [x] T050 Performance benchmark for embeddings endpoint in `benches/embeddings.rs`
- [x] T051 [P] Additional unit tests for config and validation in `tests/unit/`
- [x] T052 Security review of secret handling and input validation
- [x] T053 Run `quickstart.md` validation scenarios
- [x] T054 Build and smoke-test container image

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P1)**: Can start after Foundational (Phase 2) - Independent of US1
- **User Story 3 (P1)**: Can start after Foundational (Phase 2) - Independent of US1/US2
- **User Story 4 (P1)**: Can start after Foundational (Phase 2) - Builds on router from foundation
- **User Story 5 (P2)**: Can start after Foundational (Phase 2) - Packaging only, can be done in parallel with story implementation
- **User Story 6 (P2)**: Can start after Foundational (Phase 2) - Cross-cutting telemetry

### Within Each User Story

- Tests (if included) MUST be written and FAIL before implementation
- DTOs/types before handlers
- Handlers before route wiring
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Contract test for /v1/embeddings response shape in tests/contract/openai.rs"
Task: "Integration test for single and batch embedding requests in tests/integration/openai.rs"

# Launch DTO and handler implementation together:
Task: "Create OpenAI request/response types in src/routes/dto.rs"
Task: "Implement /v1/embeddings handler in src/routes/embeddings.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently with an OpenAI client
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Add User Story 4 → Test independently → Deploy/Demo
6. Add User Story 5 (Helm) → Test independently → Deploy/Demo
7. Add User Story 6 (Observability) → Test independently → Deploy/Demo
8. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (OpenAI)
   - Developer B: User Story 2 (TEI)
   - Developer C: User Story 3 (Health) + User Story 4 (Auth)
   - Developer D: User Story 5 (Helm) + User Story 6 (Observability)
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
