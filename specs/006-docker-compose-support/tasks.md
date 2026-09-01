---

description: "Task list for Docker Compose Support"
---

# Tasks: Docker Compose Support

**Input**: Design documents from `/specs/006-docker-compose-support/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/compose.md, contracts/docs.md, quickstart.md

**Tests**: Included. The constitution (Principle II) and plan.md require automated validation: `tests/compose/compose_config_test.sh` grows per story (test-first — assertions are added and confirmed failing before the compose-file change they validate).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Deployment-artifact feature at repository root (per plan.md Project Structure):
`docker-compose.yml`, `Dockerfile`, `.env.example`, `tests/compose/`,
`docs/deployment/`, `.github/workflows/`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare the repository for compose artifacts

- [X] T001 Add `models/` and `.env` entries to `.gitignore`
  - `.gitignore` currently has no compose-related entries; add a `# Docker Compose` block with `models/` (host model cache, per data-model.md → *Model Storage Location*) and `.env` (local overrides; only `.env.example` is committed — contracts/compose.md variable rules).
  - Verify with `git status --short` that neither path would be tracked.

**Checkpoint**: Repository ignores the model cache and local env file.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Infrastructure every user story depends on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T002 Add `curl` and a `HEALTHCHECK` to the runtime stage in `Dockerfile`
  - In the **runtime stage only** (after the existing `ca-certificates` layer): install `curl` in the same `apt-get install` run; do NOT touch the builder stage, `ENTRYPOINT`, or `EXPOSE 8080` (contracts/compose.md Dockerfile rules 1, 3).
  - Add: `HEALTHCHECK --interval=30s --timeout=5s --start-period=300s --retries=3 CMD curl -fsS http://127.0.0.1:8080/health || exit 1` (start period ≥ 5 min covers first-launch model downloads — research.md, health check decision).
  - Validate: `docker build` succeeds (or at minimum `docker build --check .` passes lint rules if a full build is not feasible locally) and the stage diff is limited to the two described changes.

- [X] T003 Create test harness skeleton in `tests/compose/compose_config_test.sh`
  - Mirror `tests/helm/lint_test.sh` conventions: `#!/usr/bin/env bash`, `set -euo pipefail`, `REPO_ROOT="$(git rev-parse --show-toplevel)"`, executable bit set.
  - Shared helpers (reused by per-story assertion blocks added in T004/T006/T008): render the config once into a temp file via `docker compose -f "$REPO_ROOT/docker-compose.yml" config > "$tmp"`; fail with a clear message when `docker-compose.yml` is missing (this is what makes per-story assertions FAIL before their implementation).
  - Assert the global invariants already decided in contracts/compose.md: no obsolete top-level `version:` key; `restart: unless-stopped` present; `stop_grace_period: 30s` present.
  - Confirm the script exits non-zero now (`docker compose config` fails without a compose file) — test-first per constitution Principle II.

**Checkpoint**: Dockerfile carries the health check; the offline test harness exists and fails for the right reason (no compose file yet).

---

## Phase 3: User Story 1 — Launch the service locally with two models using one command (Priority: P1) 🎯 MVP

**Goal**: `docker compose up -d` from a fresh checkout starts the published image serving exactly the multilingual and code-v2 models, with the multilingual model as default.

**Independent Test**: Run `docker compose up -d`, wait for healthy, then `GET /v1/models` lists both models and per-model embedding requests succeed (quickstart.md Scenario 1). No docs, volumes, or `.env` needed.

### Tests for User Story 1 (write first, confirm FAIL) ⚠️

- [X] T004 [P] [US1] Add US1 assertions to `tests/compose/compose_config_test.sh`
  - Rendered-config assertions (contracts/compose.md → Rendered-config invariants): `MODEL` contains exactly `minishlab/potion-multilingual-128M` and `minishlab/potion-code-16M-v2`; effective `DEFAULT_MODEL` is `minishlab/potion-multilingual-128M`; image is `ghcr.io/freinold/model2vec-serve:latest` by default; port mapping defaults to `8080:8080` and honors `MODEL2VEC_PORT` when set (re-render with the variable set); service is named `model2vec-serve`.
  - Run the script and confirm failure (no compose file yet).

### Implementation for User Story 1

- [X] T005 [US1] Create `docker-compose.yml` with the two-model default stack
  - Single service `model2vec-serve`, `container_name: model2vec-serve`, image `${MODEL2VEC_IMAGE:-ghcr.io/freinold/model2vec-serve:latest}` (variable-ized from the start per research.md D2 — avoids rework in US3).
  - `environment:` mapping form: `MODEL: ${MODEL:-minishlab/potion-multilingual-128M,minishlab/potion-code-16M-v2}` and `DEFAULT_MODEL: ${DEFAULT_MODEL:-minishlab/potion-multilingual-128M}` (FR-001, FR-007; comma-split is handled by `--model`'s `value_delimiter`).
  - `ports: ["${MODEL2VEC_PORT:-8080}:8080"]`, `restart: unless-stopped`, `stop_grace_period: 30s` (FR-005, FR-008, FR-013).
  - NO top-level `version:` key; NO health check in this file (inherited from image per contracts/compose.md rule 8); NO volumes yet (US2).
  - Validate: `tests/compose/compose_config_test.sh` passes its US1 + global assertions; then run quickstart.md Scenario 1 end-to-end (`docker compose up -d`, both models answer).

**Checkpoint**: `docker compose up -d` delivers the two-model service — spec US1 acceptance scenarios 1–4 verifiable. MVP is functional.

---

## Phase 4: User Story 2 — Persist downloaded models across restarts via volume mounting (Priority: P2)

**Goal**: Model downloads land in a host directory (`./models` by default) and survive container recreation, including offline restarts.

**Independent Test**: Start once, tear down, restart offline — service still becomes ready without re-downloading and `models/.cache/huggingface/hub` is visible on the host (quickstart.md Scenario 2).

### Tests for User Story 2 (write first, confirm FAIL) ⚠️

- [X] T006 [P] [US2] Add US2 assertions to `tests/compose/compose_config_test.sh`
  - Rendered-config assertions: volume source defaults to `./models` with target `/models` and honors `MODEL2VEC_CACHE_DIR` when set (re-render with the variable set); container env `HOME: /models` present; `HF_HOME` **absent** (research.md D3 — HF_HOME is ineffective with hf-hub 0.4.3 and its presence would be misleading).
  - Run the script and confirm the new assertions fail against the current compose file.

### Implementation for User Story 2

- [X] T007 [US2] Add model-cache volume and `HOME` redirection to `docker-compose.yml`
  - `volumes: ["${MODEL2VEC_CACHE_DIR:-./models}:/models"]` and add `HOME: /models` to the `environment:` mapping (fixed value, NOT variable-sourced — data-model.md validation rule; Helm parity per research.md D3).
  - Validate: `tests/compose/compose_config_test.sh` fully passes; run quickstart.md Scenario 2 (host-side artifacts visible, warm restart fast, offline restart succeeds, `MODEL2VEC_CACHE_DIR` override works).

**Checkpoint**: Persistence works — spec US2 acceptance scenarios 1–4 verifiable.

---

## Phase 5: User Story 3 — Customize the deployment without editing the compose file (Priority: P3)

**Goal**: Port, API key, models, and image are overridable via environment/`.env`; optional service env vars pass through without the empty-string trap.

**Independent Test**: Set each documented variable one at a time (port, API key, model selection, image pin) and verify the running service reflects each override (quickstart.md Scenario 3).

### Tests for User Story 3 (write first, confirm FAIL) ⚠️

- [X] T008 [P] [US3] Add US3 assertions to `tests/compose/compose_config_test.sh`
  - Optional-var semantics (contracts/compose.md rule 6 — the empty-string trap): with nothing set, `API_KEY`, `MODEL_OWNER`, `MODEL_ALIAS`, `MAX_BATCH_SIZE`, `MAX_INPUT_LENGTH`, `LOG_LEVEL`, `REQUEST_TIMEOUT_SECONDS` are **absent** from the rendered environment (not empty strings); with a probe value set (e.g. `API_KEY=probe`), it appears verbatim.
  - `.env.example` exists and documents every variable from data-model.md → *Customization Surface* (grep for each name).
  - Run the script and confirm the new assertions fail.

### Implementation for User Story 3

- [X] T009 [P] [US3] Create `.env.example` documenting the full variable surface
  - One commented entry per variable from data-model.md → *Customization Surface*: `MODEL2VEC_IMAGE`, `MODEL2VEC_PORT`, `MODEL2VEC_CACHE_DIR`, `MODEL`, `DEFAULT_MODEL`, `API_KEY`, `MODEL_OWNER`, `MODEL_ALIAS`, `MAX_BATCH_SIZE`, `MAX_INPUT_LENGTH`, `LOG_LEVEL`, `REQUEST_TIMEOUT_SECONDS` — each with its default and effect; `API_KEY` left commented out (uncommenting enables auth).
  - Header comment: copy to `.env` (git-ignored) and `docker compose up -d` to apply.

- [X] T010 [US3] Add short-syntax optional env pass-through to `docker-compose.yml`
  - Append to `environment:` as list entries: `- DEFAULT_MODEL` (retain its mapping default? No — convert `DEFAULT_MODEL` to short syntax and rely on the service's own first-`--model` fallback? NO: FR-007 requires the explicit multilingual default → keep `DEFAULT_MODEL` in mapping form with default; short-syntax list is only for the truly optional ones: `- API_KEY`, `- MODEL_OWNER`, `- MODEL_ALIAS`, `- MAX_BATCH_SIZE`, `- MAX_INPUT_LENGTH`, `- LOG_LEVEL`, `- REQUEST_TIMEOUT_SECONDS`).
  - Compose allows mixing mapping and list forms only via YAML merge — use the list form for all entries if merging is awkward, with `DEFAULT_MODEL` expressed as `"DEFAULT_MODEL=${DEFAULT_MODEL:-minishlab/potion-multilingual-128M}"` and `MODEL` as `"MODEL=${MODEL:-...}"` (string form sets defaults while short syntax omits unset vars — verified semantics in research.md D4/D5).
  - Validate: `tests/compose/compose_config_test.sh` fully passes; run quickstart.md Scenario 3 (port, API key incl. 401-without/200-with behavior and public operational endpoints, model swap, image pin).

**Checkpoint**: Customization works without editing the file — spec US3 acceptance scenarios 1–3 verifiable.

---

## Phase 6: User Story 4 — Discover and follow the compose documentation (Priority: P3)

**Goal**: README references compose; the docs site has a complete, verbatim-executable compose page.

**Independent Test**: A fresh reader finds the README section, follows one link to the docs page, and every documented command succeeds (quickstart.md Scenario 4).

### Implementation for User Story 4

> Note: docs describe the finished behavior of US1–US3, so this phase runs after them. The four authoring tasks below touch disjoint files and can proceed in parallel.

- [X] T011 [P] [US4] Create `docs/deployment/compose.md` per contracts/docs.md section order
  - Nine sections in the contracted order: intro → prerequisites → quick start → served models → model cache/volume mounting (link to Helm persistence docs for the shared `HOME` pattern) → configuration table (mirror data-model.md *Customization Surface* exactly) → operations (logs, stop/teardown, `docker compose pull && docker compose up -d`, health-check caveat for pre-feature images) → relation to `docker run`/Helm → troubleshooting (mirror quickstart.md table).
  - Model ids and commands MUST match `docker-compose.yml` and quickstart.md exactly (contracts/docs.md quality rules).

- [X] T012 [P] [US4] Register the compose page in the VitePress sidebar in `docs/.vitepress/config.ts`
  - Add `{ text: 'Docker Compose', link: '/deployment/compose' }` between the Docker and Helm entries in the Deployment sidebar group.

- [X] T013 [P] [US4] Add the README "Docker Compose" section in `README.md`
  - Between `## Container` and `## Helm`: one-sentence value statement, `docker compose up -d`, the two default model ids with default-model note, cache location note (`./models`, survives restarts), relative link to `docs/deployment/compose.md`; plus one bullet in `## Features` (contracts/docs.md README rules 1–4; teaser only, no duplication).

- [X] T014 [P] [US4] Cross-link compose from `docs/deployment/docker.md`
  - Short pointer (one paragraph or callout) after the run sections: for a two-model local stack with persisted cache, see the Docker Compose page (contracts/docs.md docs-site rule 3).

- [X] T015 [US4] Verify the docs build with the new page and sidebar entry
  - Run the docs build from `docs/` (per `docs/package.json` scripts, e.g. `npm run docs:build`) and confirm no dead sidebar link and the page renders; then walk quickstart.md Scenario 4 (README → link → page).

**Checkpoint**: Documentation complete — spec US4 acceptance scenarios 1–3 verifiable.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: CI wiring, repo guidance, final validation

- [X] T016 Wire the compose test into CI in `.github/workflows/ci.yml`
  - Add a lightweight job (or step alongside the existing helm checks) running `tests/compose/compose_config_test.sh` on `ubuntu-latest` (Docker CLI preinstalled; `docker compose config` is offline — no image pull, per research.md CI decision). Trigger on the same paths-plus-compose-file condition style used by the helm job (`docker-compose.yml`, `.env.example`, `Dockerfile`, `tests/compose/**`).

- [X] T017 [P] Update agent/repository guidance in `AGENTS.md`
  - Repository Layout: add `docker-compose.yml` / `.env.example` / `tests/compose/` lines; Deployment Artifacts: add a short Docker Compose subsection (launch command, docs pointer) mirroring the existing Docker/Helm entries.

- [X] T018 Run the full quickstart.md validation end-to-end
  - Execute `specs/006-docker-compose-support/quickstart.md` Scenarios 1–4 plus teardown on a clean state (`rm -rf models/` first) and record results; this covers SC-001–SC-005 evidence.

- [X] T019 Final consistency sweep
  - `tests/compose/compose_config_test.sh` green; `bash -n` on all new/changed shell scripts; confirm no `helm/**` files changed (no Chart.yaml bump or ct run needed); confirm service code untouched (`git diff --stat` shows only the files listed in plan.md Project Structure); README/docs model ids match the compose file exactly.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **Foundational (Phase 2)**: Depends on Phase 1 only trivially (gitignore precedes any cache creation); T002 (Dockerfile) and T003 (test harness) are independent of each other and can run in parallel. **BLOCKS all user stories** (health check + harness must exist; assertions fail-first).
- **US1 (Phase 3)**: Depends on Phase 2. Creates `docker-compose.yml` — everything else edits it.
- **US2 (Phase 4)**: Depends on US1 (edits the compose file created there).
- **US3 (Phase 5)**: Depends on US1 (edits the compose file); independent of US2 in content but **sequential in practice** to avoid same-file merge conflicts — run US2 → US3.
- **US4 (Phase 6)**: Depends on US1–US3 (documents finished behavior). The four authoring tasks are parallelizable; T015 verifies after them.
- **Polish (Phase 7)**: T016 after the test script is final (US3); T017 any time after US1; T018/T019 last.

### User Story Dependencies

- **US1 (P1)**: Foundational only — no other-story dependencies. Independently testable (quickstart Scenario 1).
- **US2 (P2)**: Needs US1's compose file. Independently testable (quickstart Scenario 2).
- **US3 (P3)**: Needs US1's compose file. Independently testable (quickstart Scenario 3).
- **US4 (P3)**: Needs US1–US3 behavior to document. Independently testable (quickstart Scenario 4).

### Within Each User Story

- Test assertions first (T004/T006/T008), confirmed FAILING before the implementation task.
- Compose-file edits sequential within a story; test-script and `.env.example` tasks are [P] (different files).

### Parallel Opportunities

- Phase 2: T002 ∥ T003.
- US3: T009 (.env.example) ∥ T008 (test assertions) — different files; T010 afterwards (edits compose file).
- US4: T011 ∥ T012 ∥ T013 ∥ T014 (four disjoint files); T015 after.
- Polish: T017 ∥ T016 (different files).

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 → Phase 2 (harness + Dockerfile health check).
2. Phase 3: T004 → T005.
3. **STOP and VALIDATE**: quickstart.md Scenario 1 — the one-command two-model stack works end-to-end. Ship-worthy increment on its own.

### Incremental Delivery

1. MVP (US1) → validate Scenario 1.
2. + US2 → validate Scenario 2 (persistence).
3. + US3 → validate Scenario 3 (customization).
4. + US4 → validate Scenario 4 (docs); then Phase 7 polish.
5. Each increment leaves the repo consistent (tests green at every checkpoint).

### Parallel Team Strategy

With multiple contributors after Phase 2:
- Contributor A: US1 → US2 → US3 (compose file is single-writer territory).
- Contributor B: docs scaffolding (T011–T014 drafts) finalized after US3 lands; CI wiring (T016) and AGENTS.md (T017).

---

## Notes

- [P] tasks = different files, no dependencies.
- [Story] label maps task to spec user story for traceability.
- The compose file is the only multi-story shared file — that is why US2/US3 run after US1 rather than in parallel.
- All test assertions are offline (`docker compose config`); the only network-dependent validation is the quickstart, executed at checkpoints and T018.
- Commit after each task or logical group; every checkpoint leaves the tree green.
