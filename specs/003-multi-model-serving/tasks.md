# Tasks: Multi-Model Serving

**Input**: Design documents from `/specs/003-multi-model-serving/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`

**Tests**: Tests are mandatory. Every behavior-changing task MUST include at least one test, written first and failing before implementation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Verify the existing project stack can support multi-model serving without new dependencies.

- [x] T001 [P] Verified `model2vec-rs` 0.2 supports loading multiple independent `StaticModel` instances in one process.
- [x] T002 [P] Confirmed no new crates are required for the model registry.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data structures and configuration changes that MUST be complete before any user story can be implemented.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T003 Updated `Config` in `src/config.rs` to accept a list of model identifiers (`--model` repeated) and an optional `--default-model` value.
- [x] T004 Added unit tests for multi-model config parsing and default-model validation in `tests/config_unit.rs`.
- [x] T005 Created `LoadedModel` and `ModelRegistry` types in `src/model/mod.rs` keyed by model identifier.
- [x] T006 Updated `AppState` in `src/state.rs` to hold a `ModelRegistry` instead of a single model.
- [x] T007 Added `ModelNotFound` and `ModelUnavailable` error variants in `src/errors.rs`.
- [x] T008 Updated request/response DTOs in `src/routes/dto.rs` for multi-model (e.g., `ModelsListResponse`, per-model `ModelInfo`).
- [x] T009 Updated `src/main.rs` startup flow to load all configured models into the registry before binding the listener.

**Checkpoint**: Foundation ready — config parsing, registry, and error types are in place; user story implementation can now begin.

---

## Phase 3: User Story 1 - Serve Multiple Models Simultaneously (Priority: P1) 🎯 MVP

**Goal**: A single service instance loads and serves more than one model at startup.

**Independent Test**: Start the service with `minishlab/potion-multilingual-128M` and `minishlab/potion-code-16M-v2`; verify both return embeddings.

### Tests for User Story 1 (MANDATORY) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation.**

- [x] T010 [P] [US1] Add integration test for loading two models successfully in `tests/multi_model_integration.rs`.
- [x] T011 [P] [US1] Add integration test for partial startup failure (one model fails, the other remains ready) in `tests/multi_model_integration.rs`.

### Implementation for User Story 1

- [x] T012 [US1] Implement concurrent model loading in `src/model/mod.rs` and `src/state.rs` so all configured models are loaded in parallel.
- [x] T013 [US1] Implement model lookup helper on `ModelRegistry` in `src/model/mod.rs` that returns the requested model or a clear error.
- [x] T014 [US1] Update readiness logic in `src/routes/health.rs` so the service reports ready when at least one configured model loaded successfully.

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently.

---

## Phase 4: User Story 2 - OpenAI-Compatible Model Selection (Priority: P1)

**Goal**: Clients list available models and select one by name in `/v1/embeddings`.

**Independent Test**: An OpenAI client calls `/v1/models`, picks an identifier, and receives embeddings from the selected model.

### Tests for User Story 2 (MANDATORY) ⚠️

- [x] T015 [P] [US2] Add contract test for `GET /v1/models` response shape in `tests/openai_contract.rs`.
- [x] T016 [P] [US2] Add contract test for `POST /v1/embeddings` with explicit `model` field in `tests/openai_contract.rs`.
- [x] T017 [P] [US2] Add contract test for `POST /v1/embeddings` without `model` field using the default model in `tests/openai_contract.rs`.
- [x] T018 [P] [US2] Add integration test for model selection in `tests/openai_integration.rs`.

### Implementation for User Story 2

- [x] T019 [US2] Implement `GET /v1/models` handler in `src/routes/embeddings.rs` and wire it in `src/routes/mod.rs`.
- [x] T020 [US2] Update `POST /v1/embeddings` in `src/routes/embeddings.rs` to read the `model` field and route to the selected model, falling back to the default.
- [x] T021 [US2] Add `utoipa` path/response annotations for `/v1/models` and updated `/v1/embeddings` in `src/routes/embeddings.rs`.

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently.

---

## Phase 5: User Story 3 - TEI Compatibility with Multiple Models (Priority: P2)

**Goal**: Existing TEI clients continue to work through a default-model fallback, with an optional `model` query parameter for selection.

**Independent Test**: A TEI client calls `/embed` and `/info` without changes and receives the default model's results; adding `?model=` selects another model.

### Tests for User Story 3 (MANDATORY) ⚠️

- [x] T022 [P] [US3] Add contract test for `POST /embed` with default model in `tests/tei_contract.rs`.
- [x] T023 [P] [US3] Add contract test for `POST /embed?model=...` in `tests/tei_contract.rs`.
- [x] T024 [P] [US3] Add contract test for `GET /info` with default model and `?model=...` in `tests/tei_contract.rs`.
- [x] T025 [P] [US3] Add integration test for TEI model selection in `tests/tei_integration.rs`.

### Implementation for User Story 3

- [x] T026 [US3] Update `POST /embed` in `src/routes/tei.rs` to use the default model when no `model` query parameter is provided.
- [x] T027 [US3] Update `POST /embed` in `src/routes/tei.rs` to route to the model specified by the `model` query parameter.
- [x] T028 [US3] Update `GET /info` in `src/routes/tei.rs` to return metadata for the default model or the model specified by `?model=`.
- [x] T029 [US3] Document the TEI multi-model behavior and any migration notes in `docs/`.

**Checkpoint**: User Stories 1, 2, and 3 should all work independently.

---

## Phase 6: User Story 4 - Configure and Deploy Multiple Models (Priority: P2)

**Goal**: Operators can declare multiple models in the Helm chart and configuration; single-model deployments remain backward-compatible.

**Independent Test**: A Helm install with a list of models produces a ready deployment whose `/v1/models` lists all configured models.

### Tests for User Story 4 (MANDATORY) ⚠️

- [x] T030 [P] [US4] Add Helm template test for multi-model container arguments in `tests/helm/`.
- [x] T031 [P] [US4] Add Helm template test for backward-compatible single-model values in `tests/helm/`.

### Implementation for User Story 4

- [x] T032 [US4] Update `helm/model2vec-serve/values.yaml` to support a `models` list and a `defaultModel` value.
- [x] T033 [US4] Update Helm templates in `helm/model2vec-serve/templates/` to pass the model list and default model to the container.
- [x] T034 [US4] Update `helm/model2vec-serve/README.md` with multi-model installation examples.
- [x] T035 [US4] Ensure single `--model` / `MODEL` values remain backward-compatible in `src/config.rs` and the Helm chart.

**Checkpoint**: User Stories 1–4 should all work independently.

---

## Phase 7: User Story 5 - Per-Model Observability and Errors (Priority: P2)

**Goal**: Metrics, health, and errors are attributable to individual models.

**Independent Test**: After sending requests to different models, the metrics endpoint shows per-model labels and unknown-model errors are clearly identified.

### Tests for User Story 5 (MANDATORY) ⚠️

- [x] T036 [P] [US5] Add contract test for per-model labels on `/metrics` in `tests/metrics_contract.rs`.
- [x] T037 [P] [US5] Add contract test for per-model status in `/health` response in `tests/health_contract.rs`.
- [x] T038 [P] [US5] Add integration test for unknown-model error responses in `tests/openai_integration.rs` and `tests/tei_integration.rs`.

### Implementation for User Story 5

- [x] T039 [US5] Update `src/telemetry.rs` to include a `model` label on HTTP request counters and histograms where applicable.
- [x] T040 [US5] Update `src/routes/health.rs` to include a `models` array with per-model load status in the response.
- [x] T041 [US5] Ensure error responses for unknown or unavailable models use safe messages without internal paths or secrets in `src/errors.rs`.

**Checkpoint**: All user stories should now be independently functional.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, validation, and release readiness.

- [x] T042 [P] Update VitePress documentation in `docs/` for multi-model CLI args, endpoints, and Helm values.
- [x] T043 [P] Update `README.md` with multi-model quick-start examples.
- [x] T044 [P] Update `AGENTS.md` if CLI arguments or architecture change materially.
- [x] T045 Run `cargo fmt -- --check` and fix formatting issues.
- [x] T046 Run `cargo clippy --all-targets --all-features -- -D warnings` and fix all warnings.
- [x] T047 Run `cargo test` and fix failures.
- [x] T048 Run `cargo bench` and verify performance goals from `research.md` (p99 < 20 ms, ≥ 2,000 RPS/model, RSS < 2 GB, cold start < 3 s).
- [x] T049 Run the quickstart validation steps from `specs/003-multi-model-serving/quickstart.md` end-to-end.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately.
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories.
- **User Stories (Phase 3+)**: All depend on Foundational phase completion.
  - User stories can then proceed in parallel (if staffed).
  - Or sequentially in priority order (P1 → P2 → P2 → P2 → P2).
- **Polish (Final Phase)**: Depends on all desired user stories being complete.

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) — no dependencies on other stories.
- **User Story 2 (P1)**: Can start after Foundational (Phase 2) — builds on the registry from US1 but is independently testable.
- **User Story 3 (P2)**: Can start after Foundational (Phase 2) — uses registry and default model from earlier stories but is independently testable.
- **User Story 4 (P2)**: Can start after Foundational (Phase 2) — packaging work, independent of endpoint implementation.
- **User Story 5 (P2)**: Can start after Foundational (Phase 2) — observability work, independent of endpoint implementation.

### Within Each User Story

- Tests MUST be written and FAIL before implementation.
- Models/services before endpoints.
- Core implementation before integration.
- Story complete before moving to next priority.

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel.
- All Foundational tasks marked [P] can run in parallel (within Phase 2).
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows).
- All tests for a user story marked [P] can run in parallel.
- Models/services within a story marked [P] can run in parallel.
- Documentation updates in the Polish phase marked [P] can run in parallel.

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Add integration test for loading two models successfully in tests/multi_model_integration.rs"
Task: "Add integration test for partial startup failure in tests/multi_model_integration.rs"

# Launch foundational model work together:
Task: "Implement concurrent model loading in src/model/mod.rs and src/state.rs"
Task: "Implement model lookup helper on ModelRegistry in src/model/mod.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories).
3. Complete Phase 3: User Story 1.
4. **STOP and VALIDATE**: Test that two models load and serve embeddings independently.
5. Deploy/demo if ready.

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready.
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!).
3. Add User Story 2 → Test independently → Deploy/Demo.
4. Add User Story 3 → Test independently → Deploy/Demo.
5. Add User Story 4 → Test independently → Deploy/Demo.
6. Add User Story 5 → Test independently → Deploy/Demo.
7. Each story adds value without breaking previous stories.

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together.
2. Once Foundational is done:
   - Developer A: User Story 1 + User Story 2
   - Developer B: User Story 3 + User Story 5
   - Developer C: User Story 4 (Helm/packaging)
3. Stories complete and integrate independently.

---

## Notes

- `[P]` tasks = different files, no dependencies.
- `[Story]` label maps task to specific user story for traceability.
- Each user story should be independently completable and testable.
- Verify tests fail before implementing.
- Commit after each task or logical group.
- Stop at any checkpoint to validate a story independently.
- Avoid: vague tasks, same-file conflicts, cross-story dependencies that break independence.
