# Contract: Chart Values (`tls` block)

New optional values block. With all defaults, renders MUST be equivalent to
the chart before this feature (FR-011, SC-006): no new resources, mounts,
args, or probe schemes — version-derived metadata (`helm.sh/chart` label,
default image tag from `appVersion`) is expected to change on routine version
bumps and is excluded from the equivalence requirement.

```yaml
tls:
  enabled: false          # master switch
  existingSecret: ""      # required when enabled: operator-managed secret
  certKey: tls.crt        # secret key holding the PEM certificate chain
  keyKey: tls.key         # secret key holding the PEM private key
  mountPath: /etc/model2vec-serve/tls
```

## Rendered behavior when `enabled: true`

- **Volume** `tls`: `secret.secretName: <existingSecret>`, `optional: false`
  (a missing secret keeps the pod from starting — it stays
  `Pending`/`ContainerCreating` with a `MountVolume.SetUp failed` event until
  the secret exists — instead of silently serving without certs).
- **VolumeMount** `tls` at `<mountPath>` (`readOnly: true`).
- **Args** appended to the container command:
  `--tls-cert <mountPath>/<certKey>` and `--tls-key <mountPath>/<keyKey>`.
- **Probes**: `livenessProbe`/`readinessProbe` switch to `scheme: HTTPS`
  (kubelet HTTPS probes skip certificate verification, so self-signed and
  cert-manager-issued certificates work unchanged).
- **Template failure**: rendering fails (`fail`) when `tls.enabled` is true
  and `existingSecret` is empty — misconfiguration is caught at deploy time,
  not runtime.
- Everything else (service, ports, env, persistence, extraVolumes) renders
  exactly as before.

## Rendered behavior when `enabled: false` (default)

No volume, no volumeMount, no TLS args, HTTP probes — identical to the chart
before this feature.

## Secret provisioning (operator responsibility)

The chart never creates the TLS secret. The operator provisions it, e.g.:

```bash
kubectl create secret tls model2vec-serve-tls \
  --cert=fullchain.pem --key=privkey.pem
```

Standard `tls.crt`/`tls.key` keys from `kubectl create secret tls` match the
defaults; `certKey`/`keyKey` adapt to nonstandard secret layouts. The
certificate file SHOULD contain the full chain (leaf + intermediates).

## Ingress modes

| Mode | How | Encryption path |
|------|-----|-----------------|
| Edge termination (default, unchanged) | Existing `ingress` values as today | Client→edge encrypted; edge→pod plaintext |
| End-to-end (passthrough) | Existing `ingress.annotations`, e.g. nginx: `nginx.ingress.kubernetes.io/ssl-passthrough: "true"` + `nginx.ingress.kubernetes.io/backend-protocol: HTTPS` | Client→pod encrypted on every hop; the application terminates TLS |

No new ingress template logic: passthrough is annotation-driven per ingress
class and documented with a worked nginx example in the chart README and
`docs/deployment/helm.md`. With passthrough, edge routing targets the
unchanged service port; the pod serves HTTPS.

## Rotation

Certificate renewal = update the secret + rolling restart of the deployment
(e.g. `kubectl rollout restart`). The application loads cert/key once at
startup; hot reload is explicitly out of scope (spec assumption).

## Versioning

Additive feature: MINOR chart version bump, no deprecated values, no
behavior change for default installs.
