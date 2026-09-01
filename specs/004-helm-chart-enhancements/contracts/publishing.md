# Contract: Chart Publishing and Chart CI

## Release workflow (`helm-release.yml`)

**Trigger**: push to `main` with changes under `helm/**`; manual `workflow_dispatch` (FR-002).

**Permissions**: `contents: write` (GitHub Releases, `gh-pages` index branch), `packages: write` (ghcr.io push).

**Flow** (mirrors [it-at-m/helm-charts](https://github.com/it-at-m/helm-charts) `release.yml`):

1. Checkout with `fetch-depth: 0` (chart-releaser diffs against tags).
2. Configure git user (`fregante/setup-git-user@v2`).
3. Install Helm (`azure/setup-helm@v5`).
4. `helm/chart-releaser-action@v1.7.0` with `charts_dir: helm`, `config: cr.yaml`, `CR_TOKEN: ${{ secrets.GITHUB_TOKEN }}`.
5. Login to ghcr.io (`docker/login-action@v4`, `GITHUB_TOKEN`).
6. For each package in `.cr-release-packages/*`: `helm push <pkg> oci://ghcr.io/freinold/model2vec-serve`.

**Produced artifacts** (per released chart version):

| Artifact | Location | Naming |
|----------|----------|--------|
| Packaged chart | GitHub Release assets | `model2vec-serve-<version>.tgz` on release `model2vec-serve-<version>` |
| OCI chart | ghcr.io | `oci://ghcr.io/freinold/model2vec-serve/model2vec-serve:<version>` |
| Classic index | `gh-pages` branch | `index.yaml` (maintained, not web-served — Pages serves the docs site) |

**Immutability** (FR-003): chart-releaser detects the existing `model2vec-serve-<version>` release/tag and skips; the OCI push is only reached when a package was produced. A merged chart change without a version bump MUST NOT occur because `ct lint` blocks it in CI (FR-015).

**Install command** (published form):

```bash
helm install model2vec-serve \
  oci://ghcr.io/freinold/model2vec-serve/model2vec-serve \
  --version 0.5.1
```

The ghcr.io helm package must be public for anonymous installs (one-time maintainer action; see spec Assumptions).

## Automated chart release (`release.yml` `helm-chart-bump` job)

**Trigger**: successful completion of the `release-plz-release` job with `releases_created == true` (i.e. release-plz published a new app release).

**Flow** (implemented by `scripts/bump_chart.sh`):

1. Wait for the `docker.yml` run of the release tag to succeed (the chart's `appVersion` must point at an existing image before `ct install` validates it).
2. Set `Chart.yaml` `version` to the app version (mirrored); if a chart with that version was already published (existing `model2vec-serve-<version>` tag), increment the patch until free.
3. Set `Chart.yaml` `appVersion` to the app version (bare semver — the image tag consumed by `image.tag | default .Chart.AppVersion`).
4. Update the `--version` install commands in `docs/deployment/helm.md`, `README.md`, and `helm/model2vec-serve/README.md`.
5. `helm lint` the chart, commit as `chore(helm): release chart <version> with appVersion <appVersion>`, and push to main with the release PAT (GITHUB_TOKEN pushes do not trigger workflows).

The push triggers `helm-release.yml` (helm/** path filter), publishing the chart. The Cargo.toml `exclude` list keeps Chart.yaml and docs out of the cargo package so the chart bump does not trigger another app release.

## Chart CI (in existing `ci.yml` `helm` job)

**Trigger**: pull requests (and pushes) to `main`.

**Flow**:

1. Checkout with `fetch-depth: 0`; set up Helm (`azure/setup-helm@v5`).
2. Existing checks continue to run: `tests/helm/lint_test.sh`, `tests/helm/template_test.sh` (FR-018).
3. Set up chart-testing (`helm/chart-testing-action@v2.8.0`).
4. `ct list-changed --target-branch main` — if no chart changed, remaining ct steps are skipped (FR-017).
5. `ct lint --target-branch main` — includes the chart version-increment check (FR-015).
6. Create kind cluster (`helm/kind-action@v1`) and `ct install --target-branch main --debug` — skipped when the commit message contains `[skip install]` (FR-016 opt-out).

**Failure semantics**: any lint or install failure fails the job and blocks the PR (SC-006). A release that never becomes ready fails rather than passing silently.
