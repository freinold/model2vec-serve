# Implementation Plan: Helm Chart Enhancements

**Branch**: `004-helm-chart-enhancements` | **Date**: 2026-08-19 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/004-helm-chart-enhancements/spec.md`

## Summary

Make the model2vec-serve Helm chart a first-class, publishable artifact. Three operator-facing capabilities are added: (1) automated chart publishing to the GitHub Container Registry as OCI artifacts plus GitHub Release assets, mirroring the it-at-m/helm-charts release flow (chart-releaser + `helm push`); (2) an optional, auto-wired persistence block that mounts a PVC at the Hugging Face cache location so model downloads survive pod restarts; (3) an optional, standard Kubernetes Ingress template with configurable hosts, TLS, annotations, and extra labels. Chart changes are guarded by chart-testing (`ct lint` with version-increment enforcement, `ct install` in an ephemeral kind cluster) in addition to the existing bash lint/template tests. All new options are disabled by default so existing installs render unchanged resources.

## Technical Context

**Language/Version**: Helm chart templates (chart `apiVersion: v2`), GitHub Actions workflow YAML, Bash test scripts. No Rust source changes.

**Primary Dependencies**: `helm/chart-releaser-action` v1.7.0 (chart packaging + GitHub Releases), `helm/chart-testing-action` v2.8.0 (`ct lint` / `ct install`), `helm/kind-action` v1 (ephemeral cluster), `azure/setup-helm` v5, `docker/login-action` v4 (ghcr.io credentials reused by `helm push`), `fregante/setup-git-user` v2. Target: Helm 3, Kubernetes ≥ 1.19 (`networking.k8s.io/v1` Ingress).

**Storage**: Optional PVC for the Hugging Face download cache. `model2vec-rs` 0.2.1 uses `hf-hub` 0.4.3 `Api::new()`, which resolves the cache to `$HOME/.cache/huggingface/hub` and does **not** read `HF_HOME` (verified in crate sources); the chart therefore sets the `HOME` environment variable to the configured mount path when persistence is enabled.

**Testing**: Existing bash scripts (`tests/helm/lint_test.sh`, `tests/helm/template_test.sh`) kept and extended; added `ct lint` (version-increment + schema) and `ct install` (kind smoke test) on PRs that touch `helm/**`.

**Target Platform**: Kubernetes clusters (any CNCF distribution) via Helm; CI on GitHub Actions `ubuntu-latest`.

**Project Type**: Helm chart / CI packaging (no runtime service changes).

**Performance Goals**: No runtime performance impact (chart-only feature). CI budgets: chart quality checks ≤ 10 min added PR feedback time and skipped entirely when `helm/**` is untouched (SC-007); publish flow completes within 5 minutes of merge (SC-004); pod restart with a warm PVC skips the model download entirely (SC-002).

**Constraints**:
- Default values MUST render byte-identical resources to the current chart (SC-005, backward compatibility).
- Chart releases are immutable: no overwriting published versions (FR-003).
- Publishing uses the existing ghcr.io namespace; no new external services.
- OCI chart artifact path: `oci://ghcr.io/freinold/model2vec-serve` (mirrors the it-at-m reference; chart name gives the final segment).
- The GitHub Pages site remains dedicated to the VitePress docs; the chart-releaser `gh-pages` index branch is maintained for the classic repository format but is not web-served.

**Scale/Scope**: One chart; two new templates (`pvc.yaml`, `ingress.yaml`); modifications to `values.yaml`, `Chart.yaml`, `deployment.yaml`, `NOTES.txt`; one new release workflow; one extended CI workflow; one `cr.yaml` + one `ct.yaml`; `ci/test-values.yaml` for fast install tests; docs updates.

**Research Resolution**: All unknowns resolved in `research.md`:
1. Chart publishing mechanism: chart-releaser-action (packages + GitHub Release) followed by `helm push` to ghcr.io OCI, mirroring it-at-m/helm-charts.
2. Cache redirection mechanism: `HOME` env injection (hf-hub 0.4.3 ignores `HF_HOME` in the code path used by model2vec-rs).
3. Chart CI tooling: chart-testing with `ct lint` + `ct install` in kind; existing bash tests retained.
4. Fast install tests: `ci/test-values.yaml` with a tiny model (`minishlab/potion-base-2M`).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | Pass | Chart changes are validated by `helm lint`, `ct lint`, and template tests before merge. No application code is touched. |
| II. Test Coverage | Pass with action | Every new template and values option must have template-rendering assertions that fail before and pass after; `ct install` provides the end-to-end smoke test. |
| III. API Conformity | Pass | No REST API surface changes. The OpenAI/TEI contracts are untouched. Chart values are additive and backward-compatible. |
| IV. Simplicity Over Complexity | Pass with action | New tooling (chart-testing) and new workflow must be justified; rejected simpler alternatives recorded in Complexity Tracking. New values blocks solve concrete operator needs (persistence, ingress) with no speculative options. |
| V. Performance Focus | Pass | No runtime hot-path impact. CI time budgets are defined in SC-004/SC-007; install tests use a tiny model to stay within budget. |

### Post-Design Re-Check

After completing Phase 1 design artifacts (`data-model.md`, `contracts/`, `quickstart.md`):

- **I. Code Quality**: still Pass — all templates go through lint + render tests; the release workflow mirrors a proven reference implementation.
- **II. Test Coverage**: still Pass with action — `template_test.sh` gains PVC/Ingress/persistence assertions; `ct lint` enforces version bumps; `ct install` covers real deployment readiness.
- **III. API Conformity**: still Pass — chart-only feature; the default `image.repository` change points at the published ghcr.io image without altering the runtime API.
- **IV. Simplicity Over Complexity**: still Pass with action — `HF_HOME` was rejected after source verification in favor of a single `HOME` env injection; chart-testing adoption is justified in Complexity Tracking.
- **V. Performance Focus**: still Pass — `ci/test-values.yaml` keeps the kind install test fast; budgets recorded above.

## Project Structure

### Documentation (this feature)

```text
specs/004-helm-chart-enhancements/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── values.md        # Chart values contract (persistence, ingress)
│   └── publishing.md    # Release flow + artifact locations
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
helm/model2vec-serve/
├── Chart.yaml                  # version 0.2.0, appVersion 0.3.0, maintainers
├── values.yaml                 # + persistence.*, ingress.* blocks; image.repository → ghcr.io
├── ci/
│   └── test-values.yaml        # tiny model for fast ct install
└── templates/
    ├── deployment.yaml         # + persistence volume/mount + HOME env
    ├── ingress.yaml            # NEW: optional Ingress with extraLabels
    ├── pvc.yaml                # NEW: optional PersistentVolumeClaim
    └── NOTES.txt               # + ingress URL hint

.github/workflows/
├── ci.yml                      # helm job: + ct lint / ct install in kind
└── helm-release.yml            # NEW: chart-releaser + OCI push on helm/** changes

cr.yaml                         # NEW: chart-releaser config
ct.yaml                         # NEW: chart-testing config
tests/helm/
├── lint_test.sh                # unchanged
└── template_test.sh            # + persistence/ingress assertions

docs/deployment/helm.md         # new values + OCI install instructions
helm/model2vec-serve/README.md  # new values + OCI install instructions
AGENTS.md                       # key values list + helm-release workflow
```

**Structure Decision**: Single chart under `helm/model2vec-serve` (unchanged location; `cr.yaml`/`ct.yaml` at repo root point chart tooling at the `helm/` chart directory). CI changes extend the existing `ci.yml` helm job; publishing gets its own `helm-release.yml` workflow mirroring the it-at-m reference layout.

## Complexity Tracking

| Addition | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| chart-testing (`ct lint` + kind `ct install`) | Enforces chart version increments (required by the immutable-release publishing flow) and catches install-time failures that static rendering cannot (Spec US4) | Extending the existing bash scripts only — no version-bump enforcement, no real-cluster smoke test |
| chart-releaser-action + OCI push | Produces both GitHub Release assets and OCI artifacts from one trigger, mirroring the proven it-at-m/helm-charts flow | Plain `helm package` + `helm push` only — loses GitHub Release assets, generated release notes, and the classic index |
| `HOME` env injection for cache persistence | hf-hub 0.4.3 (`Api::new()` path used by model2vec-rs 0.2.1) resolves the cache as `$HOME/.cache/huggingface/hub` and ignores `HF_HOME`; verified in vendored crate sources | `HF_HOME` env injection — silently ineffective with the current dependency stack (would violate FR-008) |
| `ci/test-values.yaml` with tiny model | Keeps the kind install test within the CI time budget (SC-007) | Installing default values (potion-multilingual-128M, hundreds of MB) — slow and flaky in CI |
