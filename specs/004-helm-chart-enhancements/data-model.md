# Data Model: Helm Chart Enhancements

This feature is chart/CI-only; there is no runtime data model. The entities below are the configuration and release artifacts the feature introduces, with validation rules derived from the functional requirements.

## Chart Release

A versioned, immutable package of the Helm chart distributed through two channels.

| Field | Type | Constraints |
|-------|------|-------------|
| `name` | string | Fixed: `model2vec-serve` (from `Chart.yaml`) |
| `version` | semver string | Must increment for every merged chart change (enforced by `ct lint`, FR-015); releases are immutable (FR-003) |
| `appVersion` | string | Matches a published container image tag (e.g. `0.3.0`) so default installs pull a real image (FR-005) |
| OCI artifact | `oci://ghcr.io/freinold/model2vec-serve/model2vec-serve:<version>` | Pushed by the release workflow after packaging (FR-001) |
| Release asset | `model2vec-serve-<version>.tgz` | Attached to GitHub Release `model2vec-serve-<version>` (FR-004) |

**Lifecycle**: chart change merged to `main` with bumped `version` → packaged → GitHub Release created → OCI artifact pushed. Re-running with an unchanged version is a no-op (no overwrite, no failure).

## Persistence Configuration (`persistence.*`)

Operator-facing values controlling model-file storage (US2, FR-006…FR-009).

| Field | Type | Default | Validation |
|-------|------|---------|------------|
| `enabled` | bool | `false` | When `false`, no PVC/volume/mount/`HOME` env is rendered (FR-006, SC-005) |
| `existingClaim` | string | `""` | When non-empty, no PVC is created; the deployment references this claim name (FR-007) |
| `storageClass` | string | `""` | Empty = cluster default storage class (field omitted from PVC spec) |
| `accessModes` | list[string] | `["ReadWriteOnce"]` | Passed through to the PVC spec |
| `size` | quantity | `5Gi` | PVC storage request; must be large enough for the configured models |
| `mountPath` | string | `/models` | Absolute path; the claim mounts here and `HOME` is set to it, so downloads land in `<mountPath>/.cache/huggingface/hub` (FR-008) |
| `annotations` | map | `{}` | Merged onto the PVC metadata annotations |

**Relationships**: `Persistence Configuration` 0..1 → `PersistentVolumeClaim` (created only when enabled and no existing claim); exactly 1 → deployment volume + volumeMount + `HOME` env (when enabled).

## Ingress Configuration (`ingress.*`)

Operator-facing values controlling external exposure (US3, FR-010…FR-012).

| Field | Type | Default | Validation |
|-------|------|---------|------------|
| `enabled` | bool | `false` | When `false`, no Ingress is rendered (SC-005) |
| `className` | string | `""` | When non-empty, sets `spec.ingressClassName` |
| `annotations` | map | `{}` | Merged onto the Ingress metadata annotations |
| `extraLabels` | map | `{}` | Merged with the standard chart labels on the Ingress metadata (FR-011) |
| `hosts` | list | one example host | Each entry: `host` (string) + `paths` list of `{path, pathType}`; rules route to the release service `http` port (FR-012) |
| `tls` | list | `[]` | Standard Ingress TLS entries (`secretName`, `hosts`) |

**Relationships**: `Ingress Configuration` 0..1 → `Ingress` resource → routes to the chart's `Service` (existing entity, unchanged).

## Chart CI Configuration (`ct.yaml`, `cr.yaml`)

Tooling configuration at the repository root.

| File | Keys | Purpose |
|------|------|---------|
| `ct.yaml` | `chart-dirs: [helm]` | Chart discovery for `ct list-changed`/`lint`/`install` |
| `cr.yaml` | `generate-release-notes: true` | Release notes on generated GitHub Releases; `charts_dir: helm` is passed as an action input |

**Install-test variant**: `helm/model2vec-serve/ci/test-values.yaml` sets `model: minishlab/potion-base-2M` so `ct install` runs a fast smoke install.
