# Configuration

All configuration is passed as command-line arguments. Every argument also has
a corresponding environment variable.

## CLI arguments

| Argument | Environment variable | Default | Description |
|----------|----------------------|---------|-------------|
| `--host` | `HOST` | `0.0.0.0` | Network interface to bind to |
| `--port` | `PORT` | `8080` | Port to listen on |
| `--model` | `MODEL` | `minishlab/potion-multilingual-128M` | Hugging Face model id or local path; repeatable |
| `--default-model` | `DEFAULT_MODEL` | first `--model` | Model to use when a request does not specify one |
| `--api-key` | `API_KEY` | none | Enables Bearer token authentication |
| `--max-batch-size` | `MAX_BATCH_SIZE` | `256` | Maximum inputs per request |
| `--max-input-length` | `MAX_INPUT_LENGTH` | `512` | Maximum tokens per input |
| `--log-level` | `LOG_LEVEL` | `info` | Log level (e.g. `trace`, `debug`, `info`, `warn`, `error`) |
| `--request-timeout-seconds` | `REQUEST_TIMEOUT_SECONDS` | `30` | Per-request timeout |

## Example: local development

```bash
cargo run --release -- \
  --model minishlab/potion-multilingual-128M \
  --port 8080 \
  --log-level debug
```

## Example: with API key

```bash
cargo run --release -- \
  --model minishlab/potion-multilingual-128M \
  --api-key my-secret-key \
  --max-batch-size 128 \
  --max-input-length 256
```

## Example: multiple models

```bash
cargo run --release -- \
  --model minishlab/potion-multilingual-128M \
  --model minishlab/potion-code-16M-v2 \
  --default-model minishlab/potion-multilingual-128M \
  --port 8080
```

The `MODEL` environment variable accepts a comma-separated list:

```bash
MODEL=minishlab/potion-multilingual-128M,minishlab/potion-code-16M-v2 \
DEFAULT_MODEL=minishlab/potion-multilingual-128M \
cargo run --release
```

## Docker / Kubernetes

When running the container or Helm chart, pass values as environment variables:

```bash
docker run -p 8080:8080 \
  -e MODEL=minishlab/potion-multilingual-128M \
  -e API_KEY=my-secret-key \
  -e MAX_BATCH_SIZE=128 \
  model2vec-serve:latest
```

In Helm, use `--set`:

```bash
helm install model2vec-serve ./helm/model2vec-serve \
  --set models={minishlab/potion-multilingual-128M,minishlab/potion-code-16M-v2} \
  --set defaultModel=minishlab/potion-multilingual-128M \
  --set apiKey=my-secret-key \
  --set args[0]=--max-batch-size \
  --set args[1]=128
```

See [Helm](./deployment/helm.md) for the full list of chart values.
