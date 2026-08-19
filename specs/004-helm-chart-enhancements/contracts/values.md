# Contract: Chart Values (new blocks)

This contract documents the new `values.yaml` blocks introduced by this feature. All blocks are optional and disabled by default; a default `helm template` render MUST be identical to the chart before this feature (SC-005).

## `persistence`

```yaml
persistence:
  enabled: false            # create/wire a PVC for the model cache
  existingClaim: ""         # use this existing PVC instead of creating one
  storageClass: ""          # empty = cluster default
  accessModes: ["ReadWriteOnce"]
  size: 5Gi
  mountPath: /models        # claim mount path; HOME is set to this path
  annotations: {}
```

**Rendered behavior when `enabled: true`:**

- `PersistentVolumeClaim` `<release>-models` (only when `existingClaim` is empty) with the configured size, storage class, access modes, annotations.
- Deployment volume `models` → `persistentVolumeClaim.claimName: <existingClaim or <release>-models>`.
- Container volumeMount `models` at `mountPath`.
- Container env `HOME=<mountPath>` rendered **before** `.Values.env`, so an operator-supplied `HOME` overrides it (FR-009). Downloads land in `<mountPath>/.cache/huggingface/hub` (see `research.md` → cache redirection).

## `ingress`

```yaml
ingress:
  enabled: false
  className: ""             # optional spec.ingressClassName
  annotations: {}
  extraLabels: {}           # merged with standard chart labels on the Ingress
  hosts:
    - host: model2vec-serve.local
      paths:
        - path: /
          pathType: Prefix
  tls: []                   # standard ingress TLS entries
```

**Rendered behavior when `enabled: true`:**

- `networking.k8s.io/v1` `Ingress` named `<release>` (chart fullname).
- Metadata labels = `model2vec-serve.labels` ⊕ `ingress.extraLabels` (FR-011).
- Metadata annotations = `ingress.annotations`.
- Rules: one rule per `hosts[]` entry; each path routes to service `<release>` port `http` (FR-012).
- `tls` block rendered only when non-empty.

## Changed existing values

| Value | Before | After | Reason |
|-------|--------|-------|--------|
| `image.repository` | `model2vec-serve` | `ghcr.io/freinold/model2vec-serve` | Published chart must pull a real image (FR-005) |

## `Chart.yaml` changes

| Field | Before | After |
|-------|--------|-------|
| `version` | `0.1.0` | `0.2.0` |
| `appVersion` | `"0.1.0"` | `"0.3.0"` |
| `maintainers` | — | project maintainer entry (required by `ct lint`) |
| `icon` | — | `https://raw.githubusercontent.com/freinold/model2vec-serve/main/docs/public/model2vec_logo.png` (reused docs logo) |
