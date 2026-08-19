# Research: Helm Chart Enhancements

## Decision: Chart publishing mechanism

- **Decision**: Mirror the it-at-m/helm-charts release flow. A new `helm-release.yml` workflow triggers on pushes to `main` that touch `helm/**` (plus `workflow_dispatch`), runs `helm/chart-releaser-action@v1.7.0` (which packages changed charts, creates GitHub Releases named `model2vec-serve-<chart-version>`, attaches the `.tgz` assets, and maintains `index.yaml` on a `gh-pages` branch), then pushes every package in `.cr-release-packages/` to `oci://ghcr.io/freinold/model2vec-serve` via `helm push` after `docker/login-action` authenticates to ghcr.io.
- **Rationale**: Verified against the reference repository's `release.yml` and the chart-releaser-action documentation. The action's `charts_dir` input handles our non-default chart location (`helm/` instead of `charts/`). `docker/login-action` credentials are sufficient for `helm push` to ghcr.io because Helm reads the Docker client config. Re-running with an unchanged chart version is a no-op: the action detects the existing release/tag and skips it, satisfying FR-003 (immutable releases, no failure).
- **Alternatives considered**:
  - *Plain `helm package` + `helm push` on release events* — loses GitHub Release assets, generated release notes, and the classic `index.yaml`; also couples chart releases to crate releases instead of chart changes.
  - *chart-releaser with a web-served gh-pages classic repository as primary* — rejected: GitHub Pages is already consumed by the VitePress docs via the Actions-artifact flow, so a branch-served chart index would conflict; OCI is the primary channel (per spec clarification).
  - *Separate charts repository* — unnecessary operational overhead for a single chart.

## Decision: chart-releaser and chart-testing configuration

- **Decision**: Add `cr.yaml` at the repository root with `generate-release-notes: true` (mirroring the reference); pass `charts_dir: helm` as an action input. Add `ct.yaml` at the repository root with `chart-dirs: [helm]` so `ct list-changed`, `ct lint`, and `ct install` discover the chart.
- **Rationale**: The action's `charts_dir` input is the documented way to point at a non-`charts/` directory; `cr.yaml` only carries `cr` binary options (kebab-case), and `generate-release-notes` is the only behavior flag the reference repo sets. For ct, `chart-dirs` lists directories that *contain* charts, so `helm` is correct. `ct lint` validates the chart version increment by default (`--check-version-increment`), which enforces the version-bump rule the publishing flow depends on (FR-015).
- **Alternatives considered**: Moving the chart to a top-level `charts/` directory — rejected as churn with no functional gain; it would break existing documentation and scripts that reference `helm/model2vec-serve`.

## Decision: Model download cache redirection (persistence)

- **Decision**: When `persistence.enabled`, the deployment mounts the claim at `persistence.mountPath` (default `/models`) and sets the container's `HOME` environment variable to that mount path. Model downloads then land in `<mountPath>/.cache/huggingface/hub`. Operator-supplied `env` entries render *after* the injected `HOME`, so a user-provided `HOME` overrides the chart default (Kubernetes applies the last duplicate env entry), satisfying FR-009.
- **Rationale**: Verified in the vendored crate sources: `model2vec-rs` 0.2.1 loads remote models via `hf_hub::api::sync::Api::new()` (hf-hub **0.4.3**), which builds its cache with `Cache::default()` → `dirs::home_dir()/.cache/huggingface/hub`. The `HF_HOME` variable is only honored by `ApiBuilder::from_env()`, which is *not* the code path used. On Linux, `dirs::home_dir()` reads the `HOME` environment variable first, so injecting `HOME` reliably redirects the cache for any runtime user. (The `hf-hub` 1.0.0 dependency in `Cargo.lock` is only a test fixture for this repository and is irrelevant to the runtime path.)
- **Alternatives considered**:
  - *`HF_HOME` env injection* — silently ineffective with the current stack; would produce a PVC that never receives the cache, violating FR-008.
  - *Hardcoded mount at `/root/.cache/huggingface`* — works for the current root-based image but hardcodes user/home layout and breaks if the runtime user or base image changes.
  - *Patching model2vec-rs to accept a cache-dir argument* — upstream change, out of scope for a chart feature; can be revisited later.

## Decision: Chart CI tooling and scope

- **Decision**: Extend the existing `helm` job in `.github/workflows/ci.yml` with chart-testing: `ct list-changed --target-branch main` gates the following steps; `ct lint --target-branch main` (version-increment + schema + yamllint) runs on PRs that change the chart; `helm/kind-action@v1` provisions an ephemeral cluster and `ct install --target-branch main --debug` performs the install smoke test. The existing `tests/helm/lint_test.sh` and `tests/helm/template_test.sh` continue to run unchanged in the same job. An opt-out marker (`[skip install]` in the commit message) skips the install test for documentation-only chart changes, mirroring the reference repo.
- **Rationale**: Matches the clarified tooling decision (chart-testing + keep existing tests) and the it-at-m CI shape, while living in the existing workflow to keep the Actions surface small. `ct lint`'s version-increment check is the enforcement mechanism for FR-015. Keeping the bash tests preserves the value-specific assertions ct cannot express (FR-018), e.g., counting `--model` flags.
- **Alternatives considered**:
  - *Separate `helm-ci.yml` workflow* — rejected: the existing `helm` job already aggregates chart checks; another workflow adds indirection without benefit.
  - *Replacing bash tests with ct entirely* — rejected: loses multi-model and value-specific template assertions.

## Decision: Fast install-test values

- **Decision**: Add `helm/model2vec-serve/ci/test-values.yaml` setting the deprecated single-model value to `minishlab/potion-base-2M` (a ~8 MB model). `ct install` automatically installs the chart once per file in the chart's `ci/` directory.
- **Rationale**: The default model (`minishlab/potion-multilingual-128M`, hundreds of MB) would make every kind install test slow and flaky, risking the SC-007 ten-minute budget. The tiny model exercises the full startup path (image pull, model download, readiness probe) at a fraction of the cost.
- **Alternatives considered**: Installing with default values — rejected for CI time/flakiness; skipping the install test — violates the clarified scope of US4.

## Decision: Chart metadata and default image for publishing

- **Decision**: `Chart.yaml` moves to `version: 0.2.0` and `appVersion: "0.3.0"` (matching the current crate release), gains a `maintainers` entry (required by `ct lint`'s default maintainer validation), and `values.yaml` sets `image.repository: ghcr.io/freinold/model2vec-serve`. The deployment template already defaults `image.tag` to `appVersion`, so the published chart pulls the published image with no overrides (FR-005).
- **Rationale**: chart-releaser only releases charts whose `version` changed; `0.2.0` is the first published version containing persistence and ingress. `appVersion: 0.3.0` corresponds to the existing `ghcr.io/freinold/model2vec-serve:0.3.0` image published by `docker.yml`.
- **Alternatives considered**: Keeping `image.repository: model2vec-serve` — the published chart would reference a nonexistent registry image, failing SC-001.

## Decision: Ingress and PVC template shape

- **Decision**: Standard Helm-idiomatic templates. `ingress.yaml` renders `networking.k8s.io/v1` Ingress when `ingress.enabled`, with `ingressClassName` (when set), `annotations`, merged labels (`model2vec-serve.labels` + `ingress.extraLabels`), per-host path rules routing to the release service's `http` port, and optional `tls`. `pvc.yaml` renders a PersistentVolumeClaim named `<fullname>-models` when `persistence.enabled` and no `persistence.existingClaim` is set, with configurable `storageClass` (empty = cluster default), `accessModes`, `size`, and `annotations`. `NOTES.txt` prints the first ingress host when enabled.
- **Rationale**: Follows the chart's existing conventions (helpers from `_helpers.tpl`, disabled-by-default blocks, `nindent` label inclusion) and the stable networking.k8s.io/v1 API available since Kubernetes 1.19.
- **Alternatives considered**: OpenShift Route — rejected per spec clarification (standard Ingress chosen); Gateway API HTTPRoute — rejected, less ubiquitous than Ingress.
