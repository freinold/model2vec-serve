---

description: "Task list for default model update and spec drift check"

---

# Tasks: Default Model Update and Spec Drift Check

**Input**: Design documents from `/specs/002-default-model-multilingual/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Tests are mandatory. Every behavior-changing task must include at least one test, written first and failing before implementation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Prepare the feature branch and verify the environment.

- [x] T001 Create and switch to feature branch `002-default-model-multilingual`
- [x] T002 [P] Verify Rust toolchain (1.85+) and `cargo` are available
- [x] T003 [P] Verify Helm 3 and Docker are available for container validation

---

## Phase 2: Foundational

**Purpose**: Apply the core default model change that all other work depends on.

**⚠️ CRITICAL**: No user story documentation sweep can be considered complete until the code and Helm defaults are updated.

- [x] T004 [P] Add `default_value = "minishlab/potion-multilingual-128M"` to the `--model` argument in `src/config.rs`
- [x] T005 [P] Update `model` default value to `minishlab/potion-multilingual-128M` in `helm/model2vec-serve/values.yaml`
- [x] T006 Update `tests/config_unit.rs` to assert the new default model identifier (or add a dedicated default-value test)

**Checkpoint**: Foundation ready — the CLI and Helm now default to the multilingual model, and the default value is covered by a unit test.

---

## Phase 3: User Story 1 - Multilingual Model as Default (Priority: P1) 🎯 MVP

**Goal**: When the user starts the service without a model identifier, it loads `minishlab/potion-multilingual-128M` and serves embeddings.

**Independent Test**: `cargo run --release -- --port 8080` and `curl http://localhost:8080/info` returns `model_id: minishlab/potion-multilingual-128M`.

### Tests for User Story 1

- [x] T007 [P] [US1] Add unit test in `tests/config_unit.rs` that parses empty CLI args and verifies the default `model` value
- [x] T008 [P] [US1] Add integration test in `tests/openai_integration.rs` (or new test) that confirms the default model is loaded and non-English input returns embeddings

### Implementation for User Story 1

- [x] T009 [US1] Verify `src/config.rs` default value is correctly wired into `AppState::new` via `state.rs`
- [x] T010 [US1] Run local validation from `quickstart.md` (start without `--model`, verify `/info` and `/health`)

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently.

---

## Phase 4: User Story 2 - Consistent Documentation Examples (Priority: P1)

**Goal**: All public-facing documentation examples that present the default model reference `minishlab/potion-multilingual-128M`.

**Independent Test**: `grep -R "potion-base-2M" README.md AGENTS.md helm/model2vec-serve/README.md helm/model2vec-serve/values.yaml docs/ specs/001-model2vec-embedding-api/` shows only intentional non-default examples.

### Tests for User Story 2

- [x] T011 [P] [US2] Add documentation consistency check script (or manual checklist) to verify old default references are gone or explicitly labeled as non-default

### Implementation for User Story 2

- [x] T012 [P] [US2] Update quick-start examples in `README.md` to use `minishlab/potion-multilingual-128M`
- [x] T013 [P] [US2] Update local run and Docker examples in `AGENTS.md` to use `minishlab/potion-multilingual-128M`
- [x] T014 [P] [US2] Update default value table and examples in `helm/model2vec-serve/README.md`
- [x] T015 [P] [US2] Update examples in `docs/getting-started.md` to use `minishlab/potion-multilingual-128M`
- [x] T016 [P] [US2] Update examples in `docs/configuration.md` to use `minishlab/potion-multilingual-128M`
- [x] T017 [P] [US2] Update examples in `docs/deployment/docker.md` to use `minishlab/potion-multilingual-128M`
- [x] T018 [P] [US2] Update examples in `docs/deployment/helm.md` to use `minishlab/potion-multilingual-128M`
- [x] T019 [P] [US2] Update examples in `docs/api/openai.md` to use `minishlab/potion-multilingual-128M`
- [x] T020 [P] [US2] Update examples in `docs/api/tei.md` to use `minishlab/potion-multilingual-128M`
- [x] T021 [US2] Run the grep consistency check and fix any remaining unintentional `potion-base-2M` default references

**Checkpoint**: At this point, User Stories 1 and 2 should both work independently.

---

## Phase 5: User Story 3 - Accurate Specification of Hugging Face Hub Integration (Priority: P2)

**Goal**: The existing specification and research documents accurately describe the current Hugging Face Hub integration, including the direct `hf-hub` v1.x usage in test fixtures.

**Independent Test**: `specs/001-model2vec-embedding-api/research.md` contains no statements that contradict the current `hf-hub` v1.x API usage in `tests/common/mod.rs` and `benches/embeddings.rs`.

### Tests for User Story 3

- [x] T022 [P] [US3] Verify `tests/common/mod.rs` and `benches/embeddings.rs` both use the current `hf-hub` v1.x API (`HFClientSync`, `snapshot_download()`, `.send()`)

### Implementation for User Story 3

- [x] T023 [P] [US3] Add a Hugging Face Hub integration note to `specs/001-model2vec-embedding-api/research.md` explaining the direct `hf-hub` v1.x usage in test fixtures and benchmarks
- [x] T024 [P] [US3] Update `specs/001-model2vec-embedding-api/quickstart.md` examples to use the new default model
- [x] T025 [P] [US3] Update example model identifiers in `specs/001-model2vec-embedding-api/contracts/openai_embeddings.md` to use the new default where they imply a default
- [x] T026 [P] [US3] Update example model identifiers in `specs/001-model2vec-embedding-api/contracts/tei.md` to use the new default where they imply a default
- [x] T027 [US3] Review all `specs/001-model2vec-embedding-api/` documents and confirm no drift remains against the current model loading path and HF Hub integration

**Checkpoint**: All user stories should now be independently functional.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Run the full validation suite and ensure the repository remains clean.

- [x] T028 [P] Run `cargo fmt -- --check` and fix any formatting issues
- [x] T029 [P] Run `cargo clippy --all-targets --all-features -- -D warnings` and fix any warnings
- [x] T030 [P] Run `cargo test` and ensure all tests pass
- [x] T031 [P] Run the manual validation steps from `specs/002-default-model-multilingual/quickstart.md`
- [x] T032 [P] Verify that `tests/common/mod.rs` still uses `minishlab/potion-base-2M` as the test fixture and did not accidentally switch to the larger default model
- [x] T033 Verify that `benches/embeddings.rs` still uses `minishlab/potion-base-2M` as the benchmark fixture
- [x] T034 Review the final diff to ensure no secrets, temporary files, or unrelated changes were committed

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately.
- **Foundational (Phase 2)**: Depends on Setup completion. Blocks documentation sweep because the code and Helm defaults must match the documented examples.
- **User Story 1 (Phase 3)**: Depends on Foundational phase completion.
- **User Story 2 (Phase 4)**: Depends on Foundational phase completion; can run in parallel with User Story 1 (different files).
- **User Story 3 (Phase 5)**: Depends on Foundational phase completion; can run in parallel with User Stories 1 and 2 (different files).
- **Polish (Phase 6)**: Depends on all user stories being complete.

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational phase. No dependencies on other stories.
- **User Story 2 (P1)**: Can start after Foundational phase. No dependencies on other stories; only documentation changes.
- **User Story 3 (P2)**: Can start after Foundational phase. No dependencies on other stories; only spec/research updates.

### Within Each User Story

- Tests (if included) must be written and fail before implementation.
- Documentation updates within User Story 2 can happen in any order because they touch different files.
- Spec updates within User Story 3 can happen in any order because they touch different files.

### Parallel Opportunities

- All Phase 1 setup tasks can run in parallel.
- All Phase 2 foundational tasks marked [P] can run in parallel.
- Once Foundational phase completes, User Stories 1, 2, and 3 can run in parallel.
- All documentation updates within User Story 2 can run in parallel.
- All spec updates within User Story 3 can run in parallel.
- All Phase 6 polish tasks can run in parallel after the user stories are complete.

---

## Parallel Example: User Story 2

```bash
# Update all documentation examples in parallel (different files):
Task: "Update README.md quick-start examples"
Task: "Update AGENTS.md local run and Docker examples"
Task: "Update helm/model2vec-serve/README.md examples and default table"
Task: "Update docs/getting-started.md examples"
Task: "Update docs/configuration.md examples"
Task: "Update docs/deployment/docker.md examples"
Task: "Update docs/deployment/helm.md examples"
Task: "Update docs/api/openai.md examples"
Task: "Update docs/api/tei.md examples"

# Then run the consistency check:
Task: "Run grep consistency check and fix remaining unintentional references"
```

---

## Implementation Strategy

### MVP First (User Story 1 + Foundational)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational — update `src/config.rs` and `helm/model2vec-serve/values.yaml`, update the config unit test.
3. Complete Phase 3: User Story 1 — verify the default model loads end-to-end.
4. **STOP and VALIDATE**: Run `cargo test` and the quickstart default-model validation.

### Incremental Delivery

1. Complete Setup + Foundational.
2. Add User Story 1 → Test independently.
3. Add User Story 2 → Test documentation consistency.
4. Add User Story 3 → Test spec/research accuracy.
5. Complete Phase 6 polish → full test suite and quickstart validation.

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together.
2. Once Foundational is done:
   - Developer A: User Story 1 (default model validation)
   - Developer B: User Story 2 (documentation sweep)
   - Developer C: User Story 3 (spec drift audit)
3. Regroup for Phase 6 polish and full validation.

---

## Notes

- [P] tasks = different files, no dependencies.
- [Story] label maps task to specific user story for traceability.
- Each user story should be independently completable and testable.
- Verify tests fail before implementing.
- Commit after each task or logical group.
- Stop at any checkpoint to validate a story independently.
- Avoid vague tasks, same-file conflicts, and cross-story dependencies that break independence.
