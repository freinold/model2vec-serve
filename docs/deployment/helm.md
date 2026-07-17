# Helm

The Helm chart under `helm/model2vec-serve/` deploys `model2vec-serve` on
Kubernetes.

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

## Uninstall

```bash
helm uninstall model2vec-serve
```

## Configuration values

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas | `1` |
| `image.repository` | Container image repository | `model2vec-serve` |
| `image.tag` | Container image tag | `0.1.0` |
| `image.pullPolicy` | Image pull policy | `IfNotPresent` |
| `model` | Hugging Face model id or local path | `minishlab/potion-multilingual-128M` |
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
