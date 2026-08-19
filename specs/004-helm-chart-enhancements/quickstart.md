# Quickstart: Helm Chart Enhancements

End-to-end validation for the three capabilities. See `contracts/values.md` for the values schema and `contracts/publishing.md` for the release flow.

## Prerequisites

- Helm 3.x, plus `ct` (chart-testing) and `kind` for the CI-parity checks.
- A Kubernetes cluster with an ingress controller only for the ingress scenario.

## 1. Lint and template tests (always)

```bash
bash tests/helm/lint_test.sh
bash tests/helm/template_test.sh
```

**Expected**: both scripts pass, including the new persistence and ingress assertions.

## 2. Backward compatibility (SC-005)

```bash
helm template model2vec-serve helm/model2vec-serve > /tmp/new.yaml
```

**Expected**: rendered output contains no PVC, no Ingress, no `HOME` env, and no `models` volume — identical resources to the pre-feature chart.

## 3. Persistence (US2)

```bash
helm template model2vec-serve helm/model2vec-serve \
  --set persistence.enabled=true
```

**Expected**: output contains `kind: PersistentVolumeClaim` named `<release>-models` (5Gi, RWO); the deployment has a `models` volume referencing that claim, a volumeMount at `/models`, and env `HOME=/models`.

```bash
helm template model2vec-serve helm/model2vec-serve \
  --set persistence.enabled=true \
  --set persistence.existingClaim=my-models
```

**Expected**: no `PersistentVolumeClaim` is rendered; the deployment volume references `my-models`.

## 4. Ingress (US3)

```bash
helm template model2vec-serve helm/model2vec-serve \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=embeddings.example.com \
  --set ingress.extraLabels.environment=staging
```

**Expected**: output contains `kind: Ingress` with `environment: staging` alongside the standard chart labels, a rule for `embeddings.example.com` routing `/` to service port `http`. `helm install` output (NOTES) prints the ingress URL.

## 5. Chart-testing parity (US4)

```bash
ct list-changed --config ct.yaml --target-branch main
ct lint --config ct.yaml --target-branch main        # fails if Chart.yaml version was not bumped
kind create cluster
ct install --config ct.yaml --target-branch main --debug
```

**Expected**: lint fails on a chart change without a version bump and passes with one; the install test deploys the chart with `ci/test-values.yaml` (tiny model) and the release becomes ready. A commit message containing `[skip install]` skips only the install step.

## 6. Publishing (US1)

After merging a chart change with a bumped `Chart.yaml` version to `main`:

1. The `helm-release` workflow completes (check Actions).
2. A GitHub Release `model2vec-serve-<version>` exists with the `.tgz` asset.
3. Install from the registry with no local checkout:

```bash
helm install model2vec-serve \
  oci://ghcr.io/freinold/model2vec-serve/model2vec-serve \
  --version <version>
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=model2vec-serve --timeout=300s
```

**Expected**: the release becomes ready using the default `ghcr.io/freinold/model2vec-serve` image. Re-running the workflow without a version change creates no new release and does not fail.
