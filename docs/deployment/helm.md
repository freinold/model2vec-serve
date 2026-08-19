# Helm

The Helm chart under `helm/model2vec-serve/` deploys `model2vec-serve` on
Kubernetes.

## Install from the OCI registry

The chart is published to the GitHub Container Registry on every versioned
chart change:

```bash
helm install model2vec-serve \
  oci://ghcr.io/freinold/model2vec-serve/model2vec-serve \
  --version 0.2.0 \
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

## Uninstall

```bash
helm uninstall model2vec-serve
```

## Configuration values

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas | `1` |
| `image.repository` | Container image repository | `ghcr.io/freinold/model2vec-serve` |
| `image.tag` | Container image tag | `0.3.0` |
| `image.pullPolicy` | Image pull policy | `IfNotPresent` |
| `models` | List of Hugging Face model ids or local paths | `[]` |
| `defaultModel` | Default model when a request does not specify one (defaults to the first model in the list if omitted) | `""` |
| `modelOwner` | Model publisher or owner shown in `/v1/models` responses | `"minishlab"` |
| `model` | (Deprecated) Hugging Face model id or local path | `minishlab/potion-multilingual-128M` |
| `apiKey` | API key for authentication | `""` |
| `args` | Extra CLI arguments | `[]` |
| `env` | Extra environment variables | `[]` |
| `service.type` | Kubernetes service type | `ClusterIP` |
| `service.port` | Service port | `80` |
| `service.targetPort` | Container port | `8080` |
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

## See also

- `helm/model2vec-serve/README.md` for the embedded chart README.
- `helm/model2vec-serve/values.yaml` for all defaults and comments.
