# model2vec-serve Helm Chart

Deploys the model2vec-serve OpenAI/TEI compatible embeddings server on Kubernetes.

## Installing from the OCI registry

The chart is published to the GitHub Container Registry on every versioned chart change:

```bash
helm install model2vec-serve \
  oci://ghcr.io/freinold/model2vec-serve/model2vec-serve \
  --version 0.2.0 \
  --set models[0]=minishlab/potion-multilingual-128M \
  --set apiKey=your-secret-key
```

Installing from a local checkout (below) still works for development.

## Installation

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set model=minishlab/potion-multilingual-128M \
  --set apiKey=your-secret-key
```

## Multi-model installation

Load multiple models and specify the default one:

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

## Exposing via Ingress

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

## Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas | `1` |
| `image.repository` | Container image repository | `ghcr.io/freinold/model2vec-serve` |
| `image.tag` | Container image tag | `0.3.0` |
| `models` | List of Hugging Face model ids or local paths | `[]` |
| `defaultModel` | Default model when a request does not specify one (defaults to the first model in the list if omitted) | `""` |
| `modelOwner` | Model publisher or owner shown in `/v1/models` responses | `"minishlab"` |
| `modelAliases` | List of `{key, alias}` pairs overriding the `/tei/{model_id}/...` path segments; keys must match a `models` entry, duplicate resolved segments abort startup | `[]` |
| `model` | (Deprecated) Hugging Face model id or local path | `minishlab/potion-multilingual-128M` |
| `apiKey` | API key for authentication | `""` |
| `args` | Extra CLI arguments | `[]` |
| `resources` | CPU/memory requests and limits | see `values.yaml` |
| `autoscaling.enabled` | Enable HPA | `false` |
| `extraVolumes` | Extra volumes | `[]` |
| `extraVolumeMounts` | Extra volume mounts | `[]` |
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

## Volume-mounted models

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set model=/models/my-model \
  --set extraVolumes[0].name=model-volume \
  --set extraVolumes[0].hostPath.path=/path/to/local/model \
  --set extraVolumeMounts[0].name=model-volume \
  --set extraVolumeMounts[0].mountPath=/models/my-model
```
