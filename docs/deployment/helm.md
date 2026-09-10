# Helm

The Helm chart under `helm/model2vec-serve/` deploys `model2vec-serve` on
Kubernetes.

## Install from the OCI registry

The chart is published to the GitHub Container Registry on every versioned
chart change. Chart releases are automated: the chart version mirrors the app
version, and every app release automatically bumps and publishes a matching
chart (see the [release automation](#release-automation) section):

```bash
helm install model2vec-serve \
  oci://ghcr.io/freinold/model2vec-serve/model2vec-serve \
  --version 0.5.1 \
  --set models[0]=minishlab/potion-multilingual-128M \
  --set apiKey=your-secret-key
```

Installing from a local checkout (see "## Install" below) still works for
development.

## Install

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set model=minishlab/potion-multilingual-128M \
  --set apiKey=your-secret-key
```

## Upgrade

```bash
helm upgrade model2vec-serve ./helm/model2vec-serve \
  --set model=minishlab/potion-multilingual-128M
```

## Multi-model install

Load more than one model and choose which one is used when the request does
not specify a model:

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set models[0]=minishlab/potion-base-2M \
  --set models[1]=minishlab/potion-multilingual-128M \
  --set defaultModel=minishlab/potion-base-2M \
  --set apiKey=your-secret-key
```

## Model path aliases

The TEI per-model endpoints (`/tei/{model_id}/embed`, `/tei/{model_id}/info`)
address models by a path identifier. Set `modelAliases` to override the path
identifier of a model. Each entry's `key` must match a `models` entry (or its
derived id); the `alias` becomes the `/tei/{alias}/...` path segment. Duplicate
resolved path segments abort startup.

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set models[0]=minishlab/potion-multilingual-128M \
  --set modelAliases[0].key=minishlab/potion-multilingual-128M \
  --set modelAliases[0].alias=potion-multi
```

## Uninstall

```bash
helm uninstall model2vec-serve
```

## Configuration values

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas | `1` |
| `image.repository` | Container image repository | `ghcr.io/freinold/model2vec-serve` |
| `image.tag` | Container image tag | `""` (falls back to `appVersion`) |
| `image.pullPolicy` | Image pull policy | `IfNotPresent` |
| `models` | List of Hugging Face model ids or local paths | `[]` |
| `defaultModel` | Default model when a request does not specify one (defaults to the first model in the list if omitted) | `""` |
| `modelOwner` | Model publisher or owner shown in `/v1/models` responses | `"minishlab"` |
| `modelAliases` | List of `{key, alias}` pairs overriding the `/tei/{model_id}/...` path segments; keys must match a `models` entry, duplicate resolved segments abort startup | `[]` |
| `model` | (Deprecated) Hugging Face model id or local path | `minishlab/potion-multilingual-128M` |
| `apiKey` | API key for authentication | `""` |
| `args` | Extra CLI arguments | `[]` |
| `env` | Extra environment variables | `[]` |
| `service.type` | Kubernetes service type | `ClusterIP` |
| `service.port` | Service port | `80` |
| `service.targetPort` | Container port | `8080` |
| `service.annotations` | Extra annotations merged into the Service metadata | `{}` |
| `resources` | CPU/memory requests and limits | see `values.yaml` |
| `autoscaling.enabled` | Enable Horizontal Pod Autoscaler | `false` |
| `autoscaling.minReplicas` | Minimum replicas | `1` |
| `autoscaling.maxReplicas` | Maximum replicas | `10` |
| `autoscaling.targetCPUUtilizationPercentage` | HPA CPU target | `80` |
| `autoscaling.targetMemoryUtilizationPercentage` | HPA memory target | `80` |
| `extraVolumes` | Extra volumes | `[]` |
| `extraVolumeMounts` | Extra volume mounts | `[]` |
| `podSecurityContext` | Pod security context | `{}` |
| `securityContext` | Container security context | `{}` |
| `nodeSelector` | Node selector | `{}` |
| `tolerations` | Tolerations | `[]` |
| `affinity` | Affinity rules | `{}` |
| `persistence.enabled` | Create and mount a PVC for the model download cache | `false` |
| `persistence.existingClaim` | Use an existing PVC instead of creating one | `""` |
| `persistence.storageClass` | Storage class (empty = cluster default) | `""` |
| `persistence.accessModes` | PVC access modes | `["ReadWriteOnce"]` |
| `persistence.size` | PVC storage request | `5Gi` |
| `persistence.mountPath` | Mount path; `HOME` is set here so the HF cache lives at `<mountPath>/.cache/huggingface/hub` | `/models` |
| `persistence.annotations` | PVC annotations | `{}` |
| `tls.enabled` | Serve HTTPS from the container (single listener; disables plain HTTP) | `false` |
| `tls.existingSecret` | Kubernetes secret with the cert/key entries (required when enabled) | `""` |
| `tls.certKey` | Secret key holding the PEM certificate chain | `tls.crt` |
| `tls.keyKey` | Secret key holding the PEM private key | `tls.key` |
| `tls.mountPath` | Container mount path for the secret | `/etc/model2vec-serve/tls` |
| `ingress.enabled` | Create an Ingress for external access | `false` |
| `ingress.className` | Ingress class name | `""` |
| `ingress.annotations` | Ingress annotations | `{}` |
| `ingress.extraLabels` | Extra labels merged into the Ingress metadata | `{}` |
| `ingress.hosts` | Host/path rules (host, paths[path, pathType]) | see `values.yaml` |
| `ingress.tls` | TLS entries (secretName, hosts) | `[]` |

## Readiness and liveness

The chart exposes Kubernetes probes on:

- `/ready` for readiness
- `/health` for liveness

The service is considered ready only after the model has loaded successfully at
startup.

## Volume-mounted models

To use a model stored on a cluster volume instead of downloading from Hugging
Face:

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set model=/models/my-model \
  --set extraVolumes[0].name=model-volume \
  --set extraVolumes[0].hostPath.path=/path/to/local/model \
  --set extraVolumeMounts[0].name=model-volume \
  --set extraVolumeMounts[0].mountPath=/models/my-model
```

## Persistent model cache

Set `persistence.enabled` to mount a persistent volume claim at
`persistence.mountPath`. The chart sets the container's `HOME` to that path, so
model downloads land in `<mountPath>/.cache/huggingface/hub` and survive pod
restarts. An operator-supplied `HOME` entry in `env` overrides the injected
value.

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set models[0]=minishlab/potion-multilingual-128M \
  --set persistence.enabled=true \
  --set persistence.size=10Gi
```

Set `persistence.existingClaim` to reuse a pre-provisioned claim instead of
creating one.

## Expose via Ingress

The chart can create a Kubernetes Ingress (disabled by default). Host rules
route to the service `http` port; `ingress.extraLabels` are merged with the
standard chart labels.

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set ingress.enabled=true \
  --set ingress.className=nginx \
  --set ingress.hosts[0].host=embeddings.example.com \
  --set ingress.extraLabels.environment=production
```

## Application TLS (end-to-end encryption)

By default the chart serves plain HTTP and TLS terminates at the edge in
front of it (use `ingress.tls` for that mode). To encrypt traffic on every
hop — all the way to the application inside the container — enable
`tls.enabled` with an operator-managed secret. The chart mounts the secret as
files, passes `--tls-cert`/`--tls-key`, serves HTTPS on the target port
(single listener; plain HTTP is disabled), and switches probes to the HTTPS
scheme.

Provision the secret first (the chart never creates it):

```bash
kubectl create secret tls model2vec-serve-tls \
  --cert=fullchain.pem --key=privkey.pem
```

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set models[0]=minishlab/potion-multilingual-128M \
  --set tls.enabled=true \
  --set tls.existingSecret=model2vec-serve-tls
```

With a passthrough ingress the edge forwards encrypted traffic without
terminating TLS, so encryption reaches the application process (nginx
example). Note that the ingress-nginx controller must be started with the
`--enable-ssl-passthrough` command-line flag — it is disabled by default:

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set models[0]=minishlab/potion-multilingual-128M \
  --set tls.enabled=true \
  --set tls.existingSecret=model2vec-serve-tls \
  --set ingress.enabled=true \
  --set ingress.className=nginx \
  --set ingress.hosts[0].host=embeddings.example.com \
  --set ingress.annotations."nginx\.ingress\.kubernetes\.io/ssl-passthrough"=true \
  --set ingress.annotations."nginx\.ingress\.kubernetes\.io/backend-protocol"=HTTPS
```

Notes:

- The certificate file should contain the full chain (leaf plus
  intermediates).
- Certificates are loaded once at startup. Renewal = update the secret +
  `kubectl rollout restart deployment/<release>`; there is no hot reload.
- `tls.enabled` without `tls.existingSecret` fails `helm template`/install
  with a template error.
- Mutual TLS (client certificate verification) is not supported.

### Automatic certificates with cert-manager

cert-manager can create and renew the certificate that either mode consumes.

*Edge termination* (simplest): put the issuer annotations on the Ingress and
name the secret in `ingress.tls`. cert-manager's ingress-shim issues and
renews the certificate into that secret; the edge terminates TLS:

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set ingress.enabled=true \
  --set ingress.className=nginx \
  --set ingress.hosts[0].host=embeddings.example.com \
  --set ingress.annotations."cert-manager\.io/cluster-issuer"=letsencrypt-prod \
  --set ingress.tls[0].secretName=model2vec-serve-tls \
  --set "ingress.tls[0].hosts[0]=embeddings.example.com"
```

*Application TLS (end to end)*: have cert-manager issue into the secret the
chart mounts. Point the issuer annotations (or a standalone
`cert-manager.io/v1 Certificate` resource) at the same secret referenced by
`tls.existingSecret`:

```bash
kubectl apply -f - <<'EOF'
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: model2vec-serve
spec:
  secretName: model2vec-serve-tls
  issuerRef:
    name: letsencrypt-prod
    kind: ClusterIssuer
  dnsNames: [embeddings.example.com]
EOF

helm install model2vec-serve ./helm/model2vec-serve \
  --set models[0]=minishlab/potion-multilingual-128M \
  --set tls.enabled=true \
  --set tls.existingSecret=model2vec-serve-tls
```

ACME HTTP01 still works with the passthrough example above (ssl-passthrough
only affects port 443; the challenge is served on port 80) — or use a DNS01
solver. After each renewal, restart the pods so the application reloads the
certificate: `kubectl rollout restart deployment/<release>`.

## Resource defaults

The default `resources` block in `values.yaml` is:

```yaml
resources:
  limits:
    cpu: 1000m
    memory: 1500Mi
  requests:
    cpu: 500m
    memory: 512Mi
```

Tune these based on your model size and request volume.

## Horizontal Pod Autoscaler

Enable autoscaling with:

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set model=minishlab/potion-multilingual-128M \
  --set autoscaling.enabled=true \
  --set autoscaling.minReplicas=2 \
  --set autoscaling.maxReplicas=10
```

## Release automation

The chart version mirrors the app version and is released automatically:
after release-plz merges an app release (git tag + GitHub release + docker
image), the `helm-chart-bump` job in `.github/workflows/release.yml` sets
`Chart.yaml` `version`/`appVersion` and all live version examples (install
commands, README docker tags) via `scripts/bump_chart.sh`, pushes to main,
and `helm-release.yml` publishes the chart to GHCR. The job is safe to
re-run: if the published chart at the app version already has that
`appVersion`, it no-ops; if the version was taken by a chart-only hotfix for
a different app version, it increments the patch until free. App releases
themselves are only prepared for `feat`/`fix`/`perf`/`revert` and
`chore(deps)` commits (`release_commits` in `.release-plz.toml`), so the
chart bump never triggers another release.

## See also

- `helm/model2vec-serve/README.md` for the embedded chart README.
- `helm/model2vec-serve/values.yaml` for all defaults and comments.
