# Tasks: Helm Chart Enhancements

**Input**: Design documents from `/specs/004-helm-chart-enhancements/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/ (values.md, publishing.md), quickstart.md

**Tests**: Included — the project constitution (II. Test Coverage) and FR-013/FR-018 require template-rendering assertions that fail before and pass after each chart change.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1–US4 from spec.md)
- All paths are relative to the repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Tooling configuration files at the repository root, needed by the publishing and chart-CI stories

- [x] T001 [P] Create `cr.yaml` at repository root with `generate-release-notes: true` (chart-releaser config per contracts/publishing.md)
- [x] T002 [P] Create `ct.yaml` at repository root with `chart-dirs: [helm]` (chart-testing discovery config per research.md)
- [x] T003 [P] Create `helm/model2vec-serve/ci/test-values.yaml` containing `model: minishlab/potion-base-2M` so `ct install` runs a fast smoke install (research.md: Fast install-test values)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Chart metadata readiness — the version bump, `appVersion` alignment, maintainers entry, and pullable default image that publishing (US1) and chart linting (US4) both depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Add failing template assertions to `tests/helm/template_test.sh`: the default render's labels contain `model2vec-serve-0.2.0` and `app.kubernetes.io/version: "0.3.0"`, and the rendered image is `ghcr.io/freinold/model2vec-serve:0.3.0` (run script, confirm it FAILS)
- [x] T005 [P] Update `helm/model2vec-serve/Chart.yaml`: `version: 0.2.0`, `appVersion: "0.3.0"`, add a `maintainers` entry (required by `ct lint` default validation)
- [x] T006 [P] Change `image.repository` to `ghcr.io/freinold/model2vec-serve` in `helm/model2vec-serve/values.yaml` (FR-005)
- [x] T007 Run `bash tests/helm/lint_test.sh && bash tests/helm/template_test.sh` and confirm both PASS (T004 assertions now green)

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Install the Chart from a Published Registry (Priority: P1) 🎯 MVP

**Goal**: Chart publishes automatically to ghcr.io as an OCI artifact plus a GitHub Release asset whenever a version-bumped chart change lands on main

**Independent Test**: Merge a chart change with a bumped version → workflow `helm-release` completes → `helm install model2vec-serve oci://ghcr.io/freinold/model2vec-serve/model2vec-serve --version 0.2.0` succeeds on a fresh cluster (quickstart.md §6)

### Implementation for User Story 1

- [x] T008 [US1] Create `.github/workflows/helm-release.yml` per contracts/publishing.md: trigger on push to `main` with `paths: helm/**` plus `workflow_dispatch`; permissions `contents: write` + `packages: write`; steps: checkout (`fetch-depth: 0`) → `fregante/setup-git-user@v2` → `azure/setup-helm@v5` → `helm/chart-releaser-action@v1.7.0` (`charts_dir: helm`, `config: cr.yaml`, `CR_TOKEN: ${{ github.token }}`) → `docker/login-action@v4` (ghcr.io) → loop `helm push .cr-release-packages/* oci://ghcr.io/freinold/model2vec-serve`
- [x] T009 [US1] Validate the release flow locally without publishing: `helm package helm/model2vec-serve -d /tmp/cr-test` produces `model2vec-serve-0.2.0.tgz`, and `helm show chart /tmp/cr-test/model2vec-serve-0.2.0.tgz` reports version 0.2.0 / appVersion 0.3.0; confirm workflow YAML parses (e.g. `python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/helm-release.yml'))"`)

**Checkpoint**: Publishing pipeline exists and the chart packages cleanly; first real publish happens on merge to main

---

## Phase 4: User Story 2 - Persist Model Files on a Volume (Priority: P2)

**Goal**: Optional `persistence` block creates/wires a PVC and redirects the Hugging Face download cache onto it via `HOME` injection (research.md: hf-hub 0.4.3 ignores `HF_HOME`)

**Independent Test**: `helm template` with `persistence.enabled=true` renders PVC + volume + volumeMount + `HOME`; a kind install with persistence survives pod deletion without re-download (quickstart.md §3)

### Tests for User Story 2 ⚠️

> **NOTE: Write these assertions FIRST, ensure they FAIL before implementation**

- [x] T010 [US2] Add failing persistence assertions to `tests/helm/template_test.sh`: (a) default render contains NO `PersistentVolumeClaim`, NO `HOME`, NO volume named `models`; (b) `--set persistence.enabled=true` renders `kind: PersistentVolumeClaim` named `<release>-models` with `5Gi` and `ReadWriteOnce`, plus a `models` volume with matching `claimName`, a volumeMount at `/models`, and env `HOME` value `/models`; (c) `--set persistence.enabled=true --set persistence.existingClaim=my-models` renders NO PVC but the volume references `my-models` (run script, confirm new assertions FAIL)

### Implementation for User Story 2

- [x] T011 [P] [US2] Add the `persistence` block (`enabled`, `existingClaim`, `storageClass`, `accessModes`, `size`, `mountPath`, `annotations`) with defaults from contracts/values.md to `helm/model2vec-serve/values.yaml`
- [x] T012 [P] [US2] Create `helm/model2vec-serve/templates/pvc.yaml`: render a `PersistentVolumeClaim` named `{{ include "model2vec-serve.fullname" . }}-models` only when `persistence.enabled` and `persistence.existingClaim` is empty; include standard labels via `model2vec-serve.labels`, optional storage class (omitted when empty), access modes, size, annotations
- [x] T013 [P] [US2] Wire persistence into `helm/model2vec-serve/templates/deployment.yaml`: when `persistence.enabled` add env `HOME={{ .Values.persistence.mountPath }}` BEFORE the `.Values.env` loop (operator env wins, FR-009), a `models` volumeMount at `mountPath`, and a `models` volume whose `claimName` is `existingClaim` or `<fullname>-models`
- [x] T014 [US2] Run `bash tests/helm/lint_test.sh && bash tests/helm/template_test.sh` and confirm all assertions PASS, including the default-render no-PVC assertions (SC-005)

**Checkpoint**: Persistence works end-to-end in template rendering; default installs unchanged

---

## Phase 5: User Story 4 - Automated Chart Linting and Install Testing (Priority: P2)

**Goal**: PRs touching `helm/**` are automatically linted (with chart version-increment enforcement) and install-tested in an ephemeral kind cluster; non-chart PRs skip these checks

**Independent Test**: Open a PR with a chart change and no version bump → `ct lint` fails; with a version bump → lint and kind install pass; a PR with no `helm/**` changes runs neither (quickstart.md §5)

### Implementation for User Story 4

- [x] T015 [US4] Extend the `helm` job in `.github/workflows/ci.yml`: checkout with `fetch-depth: 0`, add `helm/chart-testing-action@v2.8.0` setup, add `ct list-changed --target-branch ${{ github.event.repository.default_branch }}` gate step exposing a `changed` output, and add `ct lint --target-branch ${{ github.event.repository.default_branch }}` guarded by that output (FR-015, FR-017)
- [x] T016 [US4] Add kind install smoke test to the same job in `.github/workflows/ci.yml`: `helm/kind-action@v1` step and `ct install --target-branch ${{ github.event.repository.default_branch }} --debug` step, both guarded by the `changed` output and by a commit-message `[skip install]` opt-out check (FR-016); keep existing `lint_test.sh`/`template_test.sh` steps running unconditionally (FR-018)
- [x] T017 [US4] Validate locally: `ct list-changed --config ct.yaml --target-branch main` detects the current branch's chart change, `ct lint --config ct.yaml --target-branch main` passes with the bumped version, and the updated workflow YAML parses

**Checkpoint**: Chart quality gates active; broken or unversioned chart changes are blocked (SC-006)

---

## Phase 6: User Story 3 - Expose the Service Through an Ingress (Priority: P3)

**Goal**: Optional `ingress` block renders a standard `networking.k8s.io/v1` Ingress with hosts, paths, TLS, annotations, and `extraLabels` merged into the standard chart labels

**Independent Test**: `helm template` with `ingress.enabled=true` renders an Ingress routing the configured host to the service `http` port with merged labels; default render contains no Ingress (quickstart.md §4)

### Tests for User Story 3 ⚠️

> **NOTE: Write these assertions FIRST, ensure they FAIL before implementation**

- [x] T018 [US3] Add failing ingress assertions to `tests/helm/template_test.sh`: (a) default render contains NO `kind: Ingress`; (b) `--set ingress.enabled=true --set ingress.hosts[0].host=embeddings.example.com --set ingress.extraLabels.environment=staging` renders `kind: Ingress` containing `environment: staging` next to the standard labels, a rule for `embeddings.example.com`, and backend service `<release>` port `http`; (c) a `--set ingress.tls[0]...` variant renders the TLS block (run script, confirm new assertions FAIL)

### Implementation for User Story 3

- [x] T019 [P] [US3] Add the `ingress` block (`enabled`, `className`, `annotations`, `extraLabels`, `hosts`, `tls`) with defaults from contracts/values.md to `helm/model2vec-serve/values.yaml`
- [x] T020 [P] [US3] Create `helm/model2vec-serve/templates/ingress.yaml`: render `networking.k8s.io/v1` Ingress when `ingress.enabled`; metadata labels = `model2vec-serve.labels` merged with `ingress.extraLabels`; optional `ingressClassName`; per-host rules routing each path to `{{ include "model2vec-serve.fullname" . }}` service port `http`; `tls` block only when non-empty (FR-010…FR-012)
- [x] T021 [P] [US3] Update `helm/model2vec-serve/templates/NOTES.txt` to print the first configured ingress host URL when `ingress.enabled`
- [x] T022 [US3] Run `bash tests/helm/lint_test.sh && bash tests/helm/template_test.sh` and confirm all assertions PASS, including the default-render no-Ingress assertion (SC-005)

**Checkpoint**: Ingress works end-to-end in template rendering; default installs unchanged

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Documentation and final validation across all stories

- [x] T023 [P] Update `helm/model2vec-serve/README.md`: add `persistence.*` and `ingress.*` rows to the configuration table, a persistence example (HF cache on PVC), an ingress example with `extraLabels`, and the OCI install command from contracts/publishing.md
- [x] T024 [P] Update `docs/deployment/helm.md`: same values-table rows and examples as T023, plus a "Installing from the registry" section with the OCI install command (replaces local-path-only instructions)
- [x] T025 [P] Update the Helm section of `README.md`: add the OCI registry install command alongside the local-path install
- [x] T026 [P] Update `AGENTS.md` Helm section: add `persistence` and `ingress` to the key values list and document the `helm-release.yml` publishing flow (chart version bump required)
- [x] T027 Run the full quickstart.md validation: `tests/helm` scripts, backward-compat render diff (§2), persistence scenarios (§3), ingress scenarios (§4), and `ct list-changed`/`ct lint` (§5; kind install only if a local kind cluster is available)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories (Chart.yaml version/maintainers required by US1 publishing and US4 ct lint)
- **User Stories (Phase 3-6)**: All depend on Foundational completion
  - US1 (P1), US2 (P2), US4 (P2), US3 (P3) touch mostly disjoint files and can proceed in parallel
  - **Delivery note**: although US1 is P1 and independently implementable, the first published chart 0.2.0 SHOULD include US2 and US3, and US4 SHOULD be active before further chart changes land (it enforces the version bump US1's immutability relies on)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Depends on Foundational (Chart.yaml/values) + T001 `cr.yaml`; no dependency on other stories
- **User Story 2 (P2)**: Depends on Foundational only; chart files disjoint from US3
- **User Story 4 (P2)**: Depends on Foundational (ct lint needs bumped version + maintainers) + T002 `ct.yaml` + T003 `ci/test-values.yaml`
- **User Story 3 (P3)**: Depends on Foundational only; chart files disjoint from US2

### Within Each User Story

- Failing template assertions MUST be written before template changes (US2: T010 → T011-T013; US3: T018 → T019-T021)
- Verification task (lint + template tests) closes each story phase
- Story complete before moving to next priority

### Parallel Opportunities

- Phase 1: T001, T002, T003 are independent files → all parallel
- Phase 2: T005, T006 → parallel after failing test T004
- US2: T011, T012, T013 → parallel (values.yaml / pvc.yaml / deployment.yaml) after T010
- US3: T019, T020, T021 → parallel (values.yaml / ingress.yaml / NOTES.txt) after T018
- Polish: T023, T024, T025, T026 → all parallel (different files)
- US2 and US3 conflict only on `values.yaml` and `template_test.sh` — sequence those two stories if working serially, or coordinate merges

---

## Parallel Example: User Story 2

```bash
# Write failing assertions first:
Task: "Add failing persistence assertions to tests/helm/template_test.sh"

# Then implement in parallel (different files):
Task: "Add persistence block to helm/model2vec-serve/values.yaml"
Task: "Create helm/model2vec-serve/templates/pvc.yaml"
Task: "Wire persistence into helm/model2vec-serve/templates/deployment.yaml"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (`cr.yaml`, `ct.yaml`, `ci/test-values.yaml`)
2. Complete Phase 2: Foundational (chart metadata + image repository, test-first)
3. Complete Phase 3: User Story 1 (helm-release workflow)
4. **STOP and VALIDATE**: `helm package` locally; on merge, the first OCI chart publishes
5. Chart is consumable from ghcr.io (MVP!)

### Incremental Delivery

1. Setup + Foundational → chart metadata ready
2. User Story 1 → publishing pipeline live (MVP)
3. User Story 2 → persistence shipped in next chart version
4. User Story 4 → quality gates enforce versions for all later chart PRs
5. User Story 3 → ingress shipped in next chart version
6. Polish → docs tell operators how to use all of it

### Parallel Team Strategy

1. One pass through Setup + Foundational together
2. Then in parallel: Developer A: US1 (workflow), Developer B: US2 (persistence), Developer C: US4 (CI), Developer D: US3 (ingress)
3. US2/US3 coordinate the shared `values.yaml` / `template_test.sh` edits at merge time
