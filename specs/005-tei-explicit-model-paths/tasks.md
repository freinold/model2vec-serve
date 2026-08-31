---

description: "Task list for TEI-Explicit Per-Model Endpoints"
---

# Tasks: TEI-Explicit Per-Model Endpoints

**Input**: Design documents from `/specs/005-tei-explicit-model-paths/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included — Constitution Principle II mandates tests for every API change; write test tasks first and confirm they FAIL before implementation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root (matches plan.md structure)

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Version bump and new configuration surface

- [x] T001 [P] Bump crate version 0.3.0 → 0.5.0 in Cargo.toml and sync the utoipa `info(version)` string in src/routes/mod.rs
- [x] T002 [P] Add `--model-alias KEY=ALIAS` configuration (repeatable, comma-delimited, env `MODEL_ALIAS`) storing `Vec<(String, String)>` in src/config.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Registry path-identifier machinery and 404 error mapping that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T003 [P] Write failing tests for `MODEL_ALIAS` parsing in tests/config_unit.rs: valid pairs, malformed pair (missing `=`), empty alias, duplicate keys, comma-separated list via env
- [x] T004 [P] Write failing integration tests in tests/multi_model_integration.rs for path identifier derivation: last-segment fallback (e.g., `minishlab/potion-base-32M` → `potion-base-32M`), alias override, and startup abort with error naming both conflicting models plus the `--model-alias` hint when two models share a path identifier, plus unmatched alias key error
- [x] T005 Implement path identifier derivation, `path_index: HashMap<String, String>`, `get_by_path()`, startup conflict validation (duplicate path identifier → error naming conflicting canonical ids and hinting to configure distinct aliases), unmatched-alias-key validation, and alias path-segment validity (non-empty, slash-free) in src/model/mod.rs (depends on T002; makes T003, T004 pass)
- [x] T006 [P] Add `AppError::ModelRouteNotFound(String)` variant mapping to HTTP 404 with error code `not_found` and message "no model is served at '/tei/{id}'" in src/errors.rs

**Checkpoint**: Foundation ready — registry resolves path identifiers uniquely at startup; user story implementation can begin

---

## Phase 3: User Story 1 — Embed Text with an Explicitly Chosen Model (Priority: P1) 🎯 MVP

**Goal**: TEI clients get embeddings from any loaded model by pointing at `/tei/{model_id}/embed` with no model qualifier.

**Independent Test**: Start the service with two models, send qualifier-free TEI embed requests to each model's per-model path, and verify each response is the TEI embedding shape (`Vec<Vec<f32>>`, input order) computed by the addressed model.

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T007 [P] [US1] Contract tests for `POST /tei/{model_id}/embed` in tests/tei_contract.rs: 200 shape and input-order preservation for string and list inputs, single-string input yields one vector, 404 `not_found` body for unknown `{model_id}` (no default fallback), 400 for empty inputs and oversized batch matching root endpoint semantics
- [x] T008 [P] [US1] Integration tests in tests/multi_model_integration.rs: with three configured models each model is independently reachable via its own path identifier and returns model-distinct embeddings; auth integration in tests/auth_integration.rs asserting `/tei/{id}/embed` returns 401 without Bearer and 200 with a valid key when `--api-key` is set
- [x] T009 [US1] Implement `POST /tei/{model_id}/embed` handler in src/routes/tei.rs: extract `Path` model identifier, resolve via `registry.get_by_path()` (unknown → `AppError::ModelRouteNotFound`), reuse shared input-validation helper (empty inputs, max batch) factored out of the existing root handler, call `model.encode`, set `RequestModelId` extension to the canonical model id
- [x] T010 [US1] Register the route inside the auth-protected `api` router and add the utoipa `#[utoipa::path]` annotation for `/tei/{model_id}/embed` in src/routes/mod.rs (depends on T009)

**Checkpoint**: User Story 1 fully functional and testable independently (MVP)

---

## Phase 4: User Story 2 — Retrieve Metadata for Every Served Model (Priority: P2)

**Goal**: Every model's metadata is discoverable via `/tei/{model_id}/info` (resolves issue #105).

**Independent Test**: With at least two models, request each model's per-model info path and verify each response is the TEI info shape reporting that model's own canonical id, max input length, embedding dimension, and pooling.

### Tests for User Story 2 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T011 [P] [US2] Contract tests for `GET /tei/{model_id}/info` in tests/tei_contract.rs: 200 TEI info shape per model, `model_id` reports the canonical identifier even when addressed via alias, 404 `not_found` for unknown `{model_id}`
- [x] T012 [P] [US2] Integration tests in tests/multi_model_integration.rs: with two+ models each model's info returns its own metadata (not the default's), and in a single-model deployment the per-model info shape is identical to the root `/info` response

- [x] T013 [US2] Implement `GET /tei/{model_id}/info` handler in src/routes/tei.rs (resolve via `get_by_path`, map to `ModelInfo` with canonical `model_id`) and register route + utoipa annotation inside the auth-protected `api` router in src/routes/mod.rs

**Checkpoint**: User Stories 1 AND 2 both work independently

---

## Phase 5: User Story 3 — Unambiguous Retirement of the Hidden Model Qualifier (Priority: P3)

**Goal**: `?model=` qualifier is rejected with an explicit 400 on all TEI endpoints; qualifier-free root endpoints keep serving the default model; migration documented.

**Independent Test**: Send embed/info requests including the retired qualifier (root and per-model) and verify each returns the documented 400 `invalid_request` naming the retired parameter and the per-model alternative; send qualifier-free root requests and verify default-model behavior is unchanged.

### Tests for User Story 3 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T014 [P] [US3] Contract tests in tests/tei_contract.rs: `/embed?model=x`, `/info?model=x`, `/tei/{id}/embed?model=x`, and `/tei/{id}/info?model=x` each return 400 `invalid_request` with a message naming the retired `model` parameter and pointing to `/tei/{model_id}/...` (including empty-value case)
- [x] T015 [P] [US3] Integration tests in tests/tei_integration.rs: qualifier-free root `/embed` and `/info` continue to serve the configured default model (single-model TEI client setup unchanged), and tests/default_model_integration.rs assertions still hold

- [x] T016 [US3] Remove `TeiModelQuery` and `Query` extractor usage from src/routes/tei.rs; reject any request whose query string contains a `model` key (any value) on all four TEI endpoints with 400 `invalid_request` naming the retired parameter and the `/tei/{model_id}/...` alternative (depends on T009, T013)
- [x] T017 [US3] Update utoipa annotations and handler doc comments in src/routes/tei.rs and src/routes/mod.rs to drop the query parameter from the documented requests and add 400 responses for the retired qualifier

**Checkpoint**: All user stories independently functional

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, Helm, benchmarks, and full validation

- [x] T018 [P] Update documentation for the 0.5.0 breaking change: VitePress docs TEI and configuration pages plus README.md with per-model paths, `--model-alias` usage, and the migration table from contracts/tei-per-model.md
- [x] T019 [P] Add `modelAliases` value to the Helm chart: map to `MODEL_ALIAS` env in helm/model2vec-serve/templates deployment and document in helm/model2vec-serve/README.md and docs/deployment/helm.md
- [x] T020 [P] Extend criterion benchmarks in benches/ to cover the per-model embed route and record invocation commands; verify no regression beyond 10% versus the root `/embed` route (SC-004)
- [x] T021 Run full validation: `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo fmt -- --check`, and execute all quickstart.md scenarios (per-model embed/info, retired qualifier 400, unknown model 404, startup conflict, auth, alias)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 (T005 needs T002) — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2; first deliverable (MVP)
- **US2 (Phase 4)**: Depends on Phase 2 only (uses `get_by_path` from T005); independent of US1's handler code except shared file src/routes/tei.rs — sequence after US1 to avoid conflicts
- **US3 (Phase 5)**: Depends on T009 and T013 (edits the same handlers to remove the qualifier)
- **Polish (Phase 6)**: Depends on all user stories complete

### User Story Dependencies

- **US1 (P1)**: Starts after Foundational — no dependencies on other stories
- **US2 (P2)**: Starts after Foundational — independently testable; shares src/routes/tei.rs with US1 so implement sequentially (US1 → US2)
- **US3 (P3)**: Requires US1 + US2 handlers in place (removes the qualifier from them)

### Within Each User Story

- Tests written first and confirmed FAILING (TDD per Constitution II)
- Handler implementation after its tests
- Route registration + utoipa annotation with the handler
- Story checkpoint validation before next story

### Parallel Opportunities

- T001 + T002 (Setup, different files)
- T003 + T004 + T006 (Foundational, different files)
- Contract + integration test tasks within each story (T007+T008, T011+T012, T014+T015)
- T018 + T019 + T020 (Polish, different files)

## Parallel Example: User Story 1

```bash
# Launch US1 test tasks together:
Task: "Contract tests for POST /tei/{model_id}/embed in tests/tei_contract.rs"
Task: "Integration + auth tests in tests/multi_model_integration.rs and tests/auth_integration.rs"

# Then implementation sequentially:
Task: "Implement per-model embed handler in src/routes/tei.rs"
Task: "Register route + utoipa annotation in src/routes/mod.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: run `cargo test --test tei_contract --test multi_model_integration` and the quickstart per-model embed scenario
5. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → foundation ready
2. US1 → validate independently (MVP: per-model embeddings)
3. US2 → validate independently (issue #105 fix: per-model info)
4. US3 → validate independently (breaking change complete: qualifier retired)
5. Polish → docs, Helm, benchmarks, full gate run → release 0.5.0

### Single-Developer Strategy

Sequential execution T001 → T021 respects shared-file conflicts
(src/routes/tei.rs and src/routes/mod.rs are touched by US1, US2, US3 —
never parallelize across stories). Within a story, batch the [P] test tasks
in one pass.

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Verify tests fail before implementing (write-first ordering above)
- Commit after each task or logical group
- Stop at any checkpoint to validate a story independently
- Avoid: vague tasks, same-file conflicts (src/routes/tei.rs tasks are strictly sequential), cross-story dependencies that break independence
