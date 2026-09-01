# Docker Compose

The repository ships a `docker-compose.yml` at its root that starts
`model2vec-serve` as a single local container serving **two** models:
`minishlab/potion-multilingual-128M` (the default model) and
`minishlab/potion-code-16M-v2`. The stack runs the published image
`ghcr.io/freinold/model2vec-serve:latest` (pushed to the GitHub Container
Registry on every release), so no build step is required — one command starts
the whole stack. This path targets local evaluation, development, and demos;
for production, use [Helm](/deployment/helm) on Kubernetes.

## Prerequisites

- Docker with Compose v2 (`docker compose version` succeeds).
- Network access for the **first** model download; roughly 2 GB of free disk
  space for the two default models.
- Port 8080 free on the host (or a different port via `MODEL2VEC_PORT`, see
  [Configuration](#configuration)).

## Quick start

From the repository root:

```bash
docker compose up -d
docker compose ps                        # wait for STATUS "healthy" (first start downloads models)
docker compose logs -f model2vec-serve   # Ctrl+C once you see the startup-complete logs
```

The first start downloads both models before the service becomes healthy; the
health check allows a five-minute start period to cover that. The health status
shown by `docker compose ps` comes from the `HEALTHCHECK` baked into the image
(a `curl` probe against `/health`).

> **Note**: images published **before** the release that introduced compose
> support carry no in-image health check. With such an image the health column
> simply does not appear in `docker compose ps` — everything else behaves the
> same. Update with `docker compose pull && docker compose up -d` to get the
> health check.

Both models are listed once the stack is healthy:

```bash
curl -s http://localhost:8080/v1/models | jq .
```

Expected: a standard OpenAI model list containing both
`minishlab/potion-multilingual-128M` and `minishlab/potion-code-16M-v2`.

Embeddings from each model (a request without a `model` field hits the default
multilingual model; the code model is selected explicitly):

```bash
curl -s http://localhost:8080/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{"input":"Hello world"}' | jq '.data[0].embedding | length'

curl -s http://localhost:8080/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{"input":"def hello(): pass","model":"minishlab/potion-code-16M-v2"}' \
  | jq '.data[0].embedding | length'
```

Expected: a positive integer (the model dimension) in both cases.

The TEI-compatible endpoints behave identically:

```bash
curl -s http://localhost:8080/info | jq .
curl -s -X POST http://localhost:8080/tei/potion-code-16M-v2/embed \
  -H "Content-Type: application/json" -d '{"inputs":["fn main() {}"]}' | jq '.[0] | length'
```

The operational endpoints are always public and need no authentication:

```bash
curl -s -o /dev/null -w '%{http_code}\n' http://localhost:8080/health   # 200
curl -s -o /dev/null -w '%{http_code}\n' http://localhost:8080/ready    # 200
curl -s -o /dev/null -w '%{http_code}\n' http://localhost:8080/metrics  # 200
```

## Served models

By default the stack serves exactly two models in one process:

| Model | Role |
|-------|------|
| `minishlab/potion-multilingual-128M` | Serves requests that do not name a model (`DEFAULT_MODEL`) |
| `minishlab/potion-code-16M-v2` | Selected explicitly via the request's `model` field or a `/tei/{model_id}/...` path |

Both the model set and the default are overridable without touching
`docker-compose.yml`:

- `MODEL` — comma-separated Hugging Face model ids (or local model directories)
  to serve. Setting it replaces the default two-model list entirely.
- `DEFAULT_MODEL` — model answering requests without an explicit model. It must
  be one of the ids in `MODEL`.

For example, serving a single smaller model:

```bash
cp .env.example .env
# in .env: MODEL=minishlab/potion-base-2M and DEFAULT_MODEL=minishlab/potion-base-2M
docker compose up -d
```

`docker compose up -d` re-reads `.env` on each start, so changes apply on the
next restart. See [Configuration](#configuration) for the `.env` workflow.

## Model cache

Downloaded model artifacts are persisted on the host so restarts are fast and
offline restarts work:

- The compose file bind-mounts the host directory `./models` (relative to the
  repository root; the directory is git-ignored and created automatically on
  first launch) at `/models` inside the container.
- The container's `HOME` is fixed to `/models`, so the Hugging Face cache lands
  in `models/.cache/huggingface/hub` on the host. This mirrors the Helm chart's
  persistence pattern, where `HOME` is set to the mount path so the cache lives
  under the persistent volume — see
  [Persistent model cache](/deployment/helm#persistent-model-cache).
- The cache survives `docker compose down`/`up` cycles and host reboots.
  Deleting the directory forces a full re-download on the next start.
- With a warm cache, a restart skips all downloads and the service becomes
  healthy in well under the first-start time. Once both models are cached, the
  stack also restarts without network access.

Override the location with `MODEL2VEC_CACHE_DIR` (any host path):

```bash
docker compose down
MODEL2VEC_CACHE_DIR=/tmp/m2v-cache docker compose up -d
ls /tmp/m2v-cache/.cache/huggingface/hub
```

If you prefer a Docker-managed named volume over a host bind mount, replace the
service's volume entry with a named volume and declare it:

```yaml
services:
  model2vec-serve:
    volumes:
      - model-cache:/models

volumes:
  model-cache:
```

A named volume is removed with `docker compose down -v`; the default bind-mount
cache is not.

## Configuration

The stack is customized through environment variables — never by editing
`docker-compose.yml`. Copy the shipped example file and uncomment what you
need:

```bash
cp .env.example .env
```

Compose loads `.env` automatically (the file is git-ignored; only
`.env.example` is committed). Every variable is optional, and unset variables
are never injected into the container as empty strings.

| Variable | Scope | Default | Effect when set |
|----------|-------|---------|-----------------|
| `MODEL2VEC_IMAGE` | compose | `ghcr.io/freinold/model2vec-serve:latest` | Use another image tag or a locally built image |
| `MODEL2VEC_PORT` | compose | `8080` | Host port the service is reachable on |
| `MODEL2VEC_CACHE_DIR` | compose | `./models` | Host directory for the model cache |
| `MODEL` | service | two-model list (see *Served models* above) | Replace the served model set (comma-separated) |
| `DEFAULT_MODEL` | service | `minishlab/potion-multilingual-128M` | Model answering requests without an explicit model |
| `API_KEY` | service | *(unset → auth off)* | Enables Bearer auth on embedding endpoints; health/ready/metrics stay public |
| `MODEL_OWNER` | service | `minishlab` (service default) | Owner shown in `/v1/models` |
| `MODEL_ALIAS` | service | *(unset)* | `KEY=ALIAS` pairs for `/tei/{model_id}/...` paths |
| `MAX_BATCH_SIZE` | service | `256` (service default) | Max inputs per request |
| `MAX_INPUT_LENGTH` | service | `512` (service default) | Max tokens per input |
| `LOG_LEVEL` | service | `info` (service default) | Log verbosity |
| `REQUEST_TIMEOUT_SECONDS` | service | `30` (service default) | Per-request timeout |

The three `MODEL2VEC_*` variables are consumed by `docker-compose.yml` itself
(image, host port, cache directory); the remaining variables are passed through
to the service verbatim.

::: warning Empty vs. unset
Unset optional variables are never injected as empty strings — this matters
most for `API_KEY`. An **empty** `API_KEY` would enable authentication with an
empty Bearer token; leaving `API_KEY` unset disables authentication entirely.
If you enabled auth and want it off again, remove the `API_KEY` line from
`.env` (or `unset API_KEY` in your shell) rather than setting it to an empty
value. `/health`, `/ready`, and `/metrics` are always public either way.
:::

## Operations

**Logs.** Follow the combined service logs:

```bash
docker compose logs -f
```

**Stop and teardown.** Both keep the model cache:

```bash
docker compose stop   # stop the container; `docker compose start` resumes it
docker compose down   # stop and remove the container; cache persists
```

To also reclaim the ~2 GB of downloaded models, delete the cache directory:

```bash
docker compose down
rm -rf models/
```

**Update the image.** Pull the latest published image and recreate the
container:

```bash
docker compose pull && docker compose up -d
```

**Restart behavior.** The container uses the `unless-stopped` restart policy: it
comes back automatically after a crash or a host reboot, but stays stopped if
you stopped it deliberately. On `docker compose stop` or `down`, the container
drains for up to 30 seconds (`stop_grace_period`), matching the default
per-request timeout, so in-flight requests can finish.

## Choosing a deployment path

- **Plain `docker run`** — quickest way to smoke-test a single model. See
  [Docker](/deployment/docker) for build, run, and image-tagging details.
- **Docker Compose** (this page) — the middle ground for local two-model
  evaluation, development, and demos: one command, two models, a persisted
  cache, and `.env`-based configuration.
- **Helm on Kubernetes** — the production path, with replicas, autoscaling,
  ingress, and persistent-volume-backed caches. See [Helm](/deployment/helm).

## Troubleshooting

| Symptom | Cause | Resolution |
|---------|-------|------------|
| `Bind for 0.0.0.0:8080 failed: port is already allocated` | Host port 8080 is in use | Set `MODEL2VEC_PORT` to a free port in `.env` and run `docker compose up -d` again |
| Container exits at startup; logs mention permission errors on `/models` | The cache directory is not writable by the container | Point `MODEL2VEC_CACHE_DIR` at a writable host path |
| Container exits at startup; logs mention download or Hugging Face errors | No network access, or `MODEL` contains a wrong model id | Restore network access or fix the model id in `MODEL` — fix the id, not the compose file |
| No health column in `docker compose ps` | The image predates the in-image `HEALTHCHECK` (published before compose support) | Update with `docker compose pull && docker compose up -d` |
| Authentication is unexpectedly enabled although no key was configured | An empty `API_KEY=""` leaked into the environment (for example from a hand-edited compose file or shell) | Use the shipped `docker-compose.yml` and unset `API_KEY` entirely instead of setting it empty |
| `rm -rf models/` fails with `Permission denied` | The container runs as root, so downloaded cache files are owned by root on the host | Remove the cache with `sudo rm -rf models/` |
