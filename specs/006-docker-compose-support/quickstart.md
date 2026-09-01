# Quickstart: Docker Compose Support

End-to-end validation for the compose feature. Execute the scenarios in order
on a machine with Docker (Compose v2) installed. Each scenario maps to a spec
user story and its acceptance scenarios; the configuration contract is
`contracts/compose.md`, the variables are listed in
`data-model.md` → *Customization Surface*.

> **Note**: the in-image health check and the compose file land in the same
> release. If validating against an **older** published image, the
> `docker compose ps` health column will not appear (the image predates the
> `HEALTHCHECK`); everything else behaves the same.

## Prerequisites

- Docker with Compose v2 (`docker compose version` succeeds).
- Network access for the **first** model download (~2 GB disk for both
  default models).
- Port 8080 free on the host (or set `MODEL2VEC_PORT`).

## Scenario 1: One-command launch, two models served (US1)

```bash
docker compose up -d
docker compose ps                 # wait for STATUS "healthy" (first start downloads models)
docker compose logs -f model2vec-serve   # Ctrl+C once you see the startup-complete logs
```

Both models are listed:

```bash
curl -s http://localhost:8080/v1/models | jq .
```

Expected: a standard OpenAI model list containing **both**
`minishlab/potion-multilingual-128M` and `minishlab/potion-code-16M-v2`.

Embeddings from each model (default request hits the multilingual model; the
code model is selected explicitly):

```bash
curl -s http://localhost:8080/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{"input":"Hello world"}' | jq '.data[0].embedding | length'

curl -s http://localhost:8080/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{"input":"def hello(): pass","model":"minishlab/potion-code-16M-v2"}' \
  | jq '.data[0].embedding | length'
```

Expected: a positive integer (the model dimension) in both cases; no error
body.

TEI endpoints behave identically:

```bash
curl -s http://localhost:8080/info | jq .
curl -s -X POST http://localhost:8080/tei/potion-code-16M-v2/embed \
  -H "Content-Type: application/json" -d '{"inputs":["fn main() {}"]}' | jq '.[0] | length'
```

Operational endpoints are public (no auth in the default path):

```bash
curl -s -o /dev/null -w '%{http_code}\n' http://localhost:8080/health   # 200
curl -s -o /dev/null -w '%{http_code}\n' http://localhost:8080/ready    # 200
curl -s -o /dev/null -w '%{http_code}\n' http://localhost:8080/metrics  # 200
```

## Scenario 2: Model cache persists across restarts (US2)

```bash
ls models/.cache/huggingface/hub        # model artifacts visible on the host
time docker compose down && time docker compose up -d
```

Expected on restart: **no download progress in the logs** and readiness well
under the first-start time (SC-002: ≥ 3× faster; warm start targets < 30 s to
ready).

Offline restart:

```bash
docker compose down
# disable network (e.g. disconnect Wi-Fi / unplug cable)
docker compose up -d && docker compose ps     # becomes healthy
curl -s http://localhost:8080/v1/models | jq '.data | length'   # 2
# re-enable network
```

Different storage location:

```bash
docker compose down
MODEL2VEC_CACHE_DIR=/tmp/m2v-cache docker compose up -d
ls /tmp/m2v-cache/.cache/huggingface/hub
```

Expected: the cache directory is created and used without any compose-file
edits.

## Scenario 3: Customization without editing the compose file (US3)

Copy the example env file and override one setting at a time
(`docker compose up -d` re-reads `.env` on each start):

```bash
cp .env.example .env
```

- **Port**: set `MODEL2VEC_PORT=9090` → `curl -s http://localhost:9090/health`
  returns 200; port 8080 no longer serves.
- **API key**: set `API_KEY=secret123` → embedding calls without the key get
  `401`; with `Authorization: Bearer secret123` succeed;
  `/health`, `/ready`, `/metrics` still return 200 without the key.
- **Models**: set `MODEL=minishlab/potion-base-2M` and
  `DEFAULT_MODEL=minishlab/potion-base-2M` → `/v1/models` lists exactly that
  model and a request without `model` succeeds.
- **Image pin**: set `MODEL2VEC_IMAGE=ghcr.io/freinold/model2vec-serve:v0.5.0`
  → `docker compose ps` shows that image running.

When done, remove `.env` (it is git-ignored) and restart:

```bash
rm .env && docker compose up -d
```

## Scenario 4: Documentation walkthrough (US4)

1. Open `README.md` → the **Docker Compose** section is visible between
   *Container* and *Helm* and links to the docs page.
2. Follow the link to `docs/deployment/compose.md` (also reachable via the
   docs-site sidebar: *Deployment → Docker Compose*).
3. Execute the page's commands in order — they are the same commands as
   Scenarios 1–3 and **every one succeeds as written** (SC-003).

## Teardown

```bash
docker compose down            # stops and removes the container; cache persists
docker compose down -v         # (only if you switched to a named volume)
sudo rm -rf models/            # optionally reclaim ~2 GB of model cache
                               # (sudo: the container runs as root, so the
                               #  downloaded files are owned by root)
```

## Troubleshooting (maps to spec Edge Cases)

| Symptom | Cause | Resolution |
|---------|-------|------------|
| Container exits at startup; logs mention download/HF errors | No network or wrong model id in `MODEL` | Restore network / fix `MODEL`; fix the id, not the compose file |
| Container exits; logs mention permissions on `/models` | Cache directory not writable | Point `MODEL2VEC_CACHE_DIR` at a writable path |
| `Bind for 0.0.0.0:8080 failed: port is already allocated` | Host port busy | Set `MODEL2VEC_PORT` to a free port |
| Health column missing in `docker compose ps` | Image predates the `HEALTHCHECK` change | `docker compose pull && docker compose up -d` (update to a post-feature image) |
| Auth unexpectedly on with no key set | `API_KEY=""` leaked into the environment (e.g. hand-edited compose file) | Use the shipped `docker-compose.yml`; unset `API_KEY` entirely |
| `rm -rf models/` fails with `Permission denied` | Container runs as root; cache files are root-owned on the host | Remove with `sudo rm -rf models/` |
