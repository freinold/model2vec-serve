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
| `--model-owner` | `MODEL_OWNER` | `minishlab` | Model publisher or owner shown in `/v1/models` responses |
| `--model-alias` | `MODEL_ALIAS` | none | Path identifier alias for a model, as `KEY=ALIAS`; repeatable |
| `--api-key` | `API_KEY` | none | Enables Bearer token authentication |
| `--tls-cert` | `TLS_CERT` | none | Path to the PEM TLS certificate (leaf plus optional chain); enables HTTPS when combined with `--tls-key` |
| `--tls-key` | `TLS_KEY` | none | Path to the unencrypted PEM private key matching `--tls-cert` |
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

## Example: model aliases

The `/tei/{model_id}/...` endpoints select a model by its path identifier. By
default this is the last segment of the model identifier (e.g.
`minishlab/potion-code-16M-v2` → `potion-code-16M-v2`). Override it with
`--model-alias KEY=ALIAS`:

```bash
cargo run --release -- \
  --model minishlab/potion-multilingual-128M \
  --model /models/potion-code-16M-v2 \
  --model-alias /models/potion-code-16M-v2=code \
  --port 8080
```

The `MODEL_ALIAS` environment variable accepts multiple `KEY=ALIAS` pairs as a
comma-separated list:

```bash
MODEL_ALIAS=/models/potion-code-16M-v2=code \
cargo run --release
```

Rules:

- `KEY` is the model identifier or local path exactly as configured via
  `--model`.
- `ALIAS` must be a single URL path segment (no slashes).
- Two models resolving to the same path identifier abort startup with an error
  hinting at `--model-alias`.

## TLS / HTTPS

The service can terminate TLS itself so traffic is encrypted end to end —
for example all the way to the application inside the container. Provide a
certificate and its matching private key as file paths:

```bash
cargo run --release -- \
  --model minishlab/potion-multilingual-128M \
  --tls-cert /etc/model2vec-serve/tls/tls.crt \
  --tls-key /etc/model2vec-serve/tls/tls.key \
  --port 8443
```

Rules:

- **Opt-in and all-or-nothing**: without `--tls-cert`/`--tls-key` the port
  serves plain HTTP exactly as before. Providing exactly one of the two fails
  startup with an error naming the missing option.
- **Single listener**: when TLS is enabled, the configured port serves HTTPS
  only. Every endpoint (embedding endpoints, `/health`, `/ready`, `/metrics`,
  `/docs`) is served over TLS with identical request/response behavior — only
  the URL scheme changes.
- **File paths only**: the certificate and key are read from files so they can
  be provisioned as mounted secrets; inline key material is never accepted.
- **Formats**: PEM-encoded X.509 certificate (full chain encouraged) and an
  unencrypted PEM private key. Encrypted (passphrase-protected) keys are
  rejected with a clear error.
- **Protocols**: TLS 1.2 and TLS 1.3 only; older protocol versions are
  refused.
- **Fail fast**: misconfiguration (missing file, invalid content, key/cert
  mismatch) aborts startup within seconds with an error naming the offending
  file and the remedy. An expired but otherwise valid certificate starts with
  a prominent warning naming the expiry date.
- **Certificate renewal**: the certificate is loaded once at startup.
  Rotating a mounted secret requires a rolling restart; there is no hot
  reload.
- **Mutual TLS** (client certificate verification) is not supported.

When TLS is enabled, configure TLS-aware probes and monitoring for
`/health`, `/ready`, and `/metrics` (they are served over the same TLS
listener). See [deployment via Helm](./deployment/helm.md) for the chart's
`tls` values that mount a Kubernetes secret and switch probes automatically.

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
