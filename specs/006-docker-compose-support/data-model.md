# Data Model: Docker Compose Support

This feature introduces no service-internal data entities. It defines one new
deployment entity, one storage entity, and one configuration surface. The
service's existing entities (models, requests, responses) are consumed
unchanged — see `specs/003-multi-model-serving/data-model.md`.

## Entity: Compose Deployment Definition

Single source of truth for launching the service locally. Realized as
`docker-compose.yml` at the repository root; validated by
`tests/compose/compose_config_test.sh`.

| Field | Value (default) | Source of truth | Notes |
|-------|-----------------|-----------------|-------|
| Service name | `model2vec-serve` | this entity | Single service; both models live in one process (multi-model spec) |
| Image | `${MODEL2VEC_IMAGE:-ghcr.io/freinold/model2vec-serve:latest}` | `MODEL2VEC_IMAGE` | Published GHCR image; no build step (FR-002) |
| Container env `MODEL` | `${MODEL:-minishlab/potion-multilingual-128M,minishlab/potion-code-16M-v2}` | `MODEL` (service env) | Comma-split by `--model`'s `value_delimiter` (FR-001) |
| Container env `DEFAULT_MODEL` | short-syntax pass-through (no compose default) | `DEFAULT_MODEL` | Unset = first `MODEL` entry (service fallback, `src/config.rs`), so the default is always inside the selected set; the multilingual model is first in the default stack (FR-007) |
| Container env `HOME` | `/models` (fixed) | this entity | Cache redirection; MUST NOT be overridable via `.env` (Helm parity; see research) |
| Optional service env | `DEFAULT_MODEL`, `API_KEY`, `MODEL_OWNER`, `MODEL_ALIAS`, `MAX_BATCH_SIZE`, `MAX_INPUT_LENGTH`, `LOG_LEVEL`, `REQUEST_TIMEOUT_SECONDS` | short-syntax pass-through | Absent from the container when unset — never set to empty string (see `contracts/compose.md` rationale) |
| Ports | `"127.0.0.1:${MODEL2VEC_PORT:-8080}:8080"` | `MODEL2VEC_PORT` | Host side overridable (FR-005), bound to loopback by default; exposing beyond localhost is an explicit compose-file edit; container fixed at 8080 (`EXPOSE`) |
| Volumes | `${MODEL2VEC_CACHE_DIR:-./models}:/models` | `MODEL2VEC_CACHE_DIR` | Bind mount; host path relative to the compose file (FR-003/FR-004) |
| Restart policy | `unless-stopped` | this entity | Crash + reboot recovery, respects deliberate stop (FR-008) |
| Stop grace period | `30s` | this entity | Matches default `REQUEST_TIMEOUT_SECONDS`; drains in-flight requests (FR-013) |
| Health check | Inherited from image `HEALTHCHECK` | `Dockerfile` | `curl -fsS http://127.0.0.1:8080/health`; compose MUST NOT duplicate it |
| Container name | `model2vec-serve` | this entity | Predictable name for `docker logs` / `docker exec` in docs |

**Validation rules** (enforced by the test script):

- `MODEL` renders with exactly the two configured model ids in the default
  path, multilingual **first** (first entry = effective default via the
  service fallback); `DEFAULT_MODEL` absent when unset, passed through when
  set.
- `HOME` renders as `/models` and is not sourced from a variable.
- Optional vars are **absent** (not empty) from the rendered config when
  unset.
- Port mapping and volume use the documented variables with the documented
  defaults; the host side binds `127.0.0.1` (loopback) in every render,
  including `MODEL2VEC_PORT` overrides.
- No obsolete top-level `version:` key.

## Entity: Model Storage Location

Host-side directory holding downloaded model artifacts; the compose-level
realization of the Helm chart's persistence concept.

| Attribute | Value |
|-----------|-------|
| Host default | `./models` (inside the checkout; git-ignored) |
| Override | `MODEL2VEC_CACHE_DIR` (any host path; **host paths only** — a named Docker volume requires editing `docker-compose.yml`, since compose rejects short-syntax references to undeclared volume names) |
| Container mount point | `/models` |
| Effective cache path | `/models/.cache/huggingface/hub` (because `HOME=/models`) |
| Lifecycle | Created automatically by the bind mount on first launch; persists across `down`/`up` and reboots; deleting it forces re-download |
| Failure modes | Unwritable location → clear startup failure logged by the container (never silent); host port/path conflicts fail immediately |

## Entity: Customization Surface (`.env.example`)

The documented set of variables a developer may set in `.env` (or the shell)
without editing `docker-compose.yml`. Shipped as `.env.example`; users copy it
to `.env`. Compose auto-loads `.env`.

| Variable | Scope | Default | Effect when set |
|----------|-------|---------|-----------------|
| `MODEL2VEC_IMAGE` | compose | `ghcr.io/freinold/model2vec-serve:latest` | Use another image tag or a locally built image |
| `MODEL2VEC_PORT` | compose | `8080` | Host port the service is reachable on (loopback-bound; expose beyond localhost by editing the ports mapping) |
| `MODEL2VEC_CACHE_DIR` | compose | `./models` | Host directory for the model cache (host paths only) |
| `MODEL` | service | two-model list (above) | Replace the served model set (comma-separated) |
| `DEFAULT_MODEL` | service | first `MODEL` entry | Model answering requests without an explicit model; must be one of `MODEL` |
| `API_KEY` | service | *(unset → auth off)* | Enables Bearer auth on embedding endpoints; health/ready/metrics stay public |
| `MODEL_OWNER` | service | `minishlab` (service default) | Owner shown in `/v1/models` |
| `MODEL_ALIAS` | service | *(unset)* | `KEY=ALIAS` pairs for `/tei/{model_id}/...` paths |
| `MAX_BATCH_SIZE` | service | `256` (service default) | Max inputs per request |
| `MAX_INPUT_LENGTH` | service | `512` (service default) | Max tokens per input |
| `LOG_LEVEL` | service | `info` (service default) | Log verbosity |
| `REQUEST_TIMEOUT_SECONDS` | service | `30` (service default) | Per-request timeout |

**Validation rules**: values are exactly the service's documented env names
(no renaming at the compose layer); optional service vars never appear empty
in the container; `.env` is git-ignored (only `.env.example` is committed).

## Container Lifecycle (state transitions)

```text
created ──▶ starting ──▶ healthy ──▶ (unhealthy ⇄ healthy)
   │            │             │
   │            │             └── docker compose stop ──▶ exited (graceful, ≤ 30 s drain)
   │            └── load failure (download/network/storage) ──▶ exited (clear log)
   └── restart: unless-stopped ──▶ re-created on crash/host reboot
```

- `starting`: models loading (first start includes download; start period must
  exceed this).
- `healthy`: image health check passes on `/health`.
- Restarting a healthy stack: cache is warm → readiness < 30 s (SC-002).
