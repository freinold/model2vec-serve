# model2vec-serve Helm Chart

Deploys the model2vec-serve OpenAI/TEI compatible embeddings server on Kubernetes.

## Installation

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set model=minishlab/potion-base-2M \
  --set apiKey=your-secret-key
```

## Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas | `1` |
| `image.repository` | Container image repository | `model2vec-serve` |
| `image.tag` | Container image tag | `0.1.0` |
| `model` | Hugging Face model id or local path | `minishlab/potion-base-2M` |
| `apiKey` | API key for authentication | `""` |
| `args` | Extra CLI arguments | `[]` |
| `resources` | CPU/memory requests and limits | see `values.yaml` |
| `autoscaling.enabled` | Enable HPA | `false` |
| `extraVolumes` | Extra volumes | `[]` |
| `extraVolumeMounts` | Extra volume mounts | `[]` |

## Volume-mounted models

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set model=/models/my-model \
  --set extraVolumes[0].name=model-volume \
  --set extraVolumes[0].hostPath.path=/path/to/local/model \
  --set extraVolumeMounts[0].name=model-volume \
  --set extraVolumeMounts[0].mountPath=/models/my-model
```
