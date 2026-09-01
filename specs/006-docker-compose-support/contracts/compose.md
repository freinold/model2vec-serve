# Contract: Compose Deployment

This contract fixes the observable behavior of `docker-compose.yml`,
`.env.example`, and the `Dockerfile` health-check change. It is the reference
for `tests/compose/compose_config_test.sh`, the docs, and future changes.
Service API behavior is **not** changed by this feature (spec FR-014); only
the deployment packaging is defined here.

## Files

| Path | Status | Purpose |
|------|--------|---------|
| `docker-compose.yml` | new | Deployment definition per `data-model.md` |
| `.env.example` | new | Documented customization variables (copy to `.env`) |
| `Dockerfile` | modified | Runtime stage gains `curl`; `HEALTHCHECK` instruction added |
| `.gitignore` | modified | Ignores `models/` (cache) and `.env` |

## Compose file rules

1. **MUST** declare a single service named `model2vec-serve` using the image
   `${MODEL2VEC_IMAGE:-ghcr.io/freinold/model2vec-serve:latest}` — never a
   `build:` context as the default path (FR-002).
2. **MUST** set `MODEL` to
   `${MODEL:-minishlab/potion-multilingual-128M,minishlab/potion-code-16M-v2}`
   (FR-001) with the multilingual id as the **first** entry, and pass
   `DEFAULT_MODEL` through with short syntax only (no compose-level default).
   The effective default model is `DEFAULT_MODEL` when set, otherwise the
   first `MODEL` entry (service fallback, `src/config.rs`) — this keeps the
   default always inside the selected `MODEL` set (FR-007): the multilingual
   model is the default exactly as long as it stays the first entry, and an
   overridden `MODEL` can never inherit an out-of-set `DEFAULT_MODEL`.
3. **MUST** mount `${MODEL2VEC_CACHE_DIR:-./models}` at `/models` and set the
   container env `HOME=/models` (FR-003). `HF_HOME` **MUST NOT** be used
   (ineffective with hf-hub 0.4.3 sync API — see research). The override
   covers host paths only: a named Docker volume requires editing this file
   (compose rejects short-syntax references to undeclared volume names).
4. **MUST** map ports as `"127.0.0.1:${MODEL2VEC_PORT:-8080}:8080"` (FR-005):
   loopback-only by default; exposing beyond localhost is an explicit
   compose-file edit.
5. **MUST** set `restart: unless-stopped` and `stop_grace_period: 30s`
   (FR-008, FR-013).
6. **MUST** pass optional service variables using short syntax
   (`- API_KEY`, `- DEFAULT_MODEL`, `- MODEL_OWNER`, `- MODEL_ALIAS`,
   `- MAX_BATCH_SIZE`, `- MAX_INPUT_LENGTH`, `- LOG_LEVEL`,
   `- REQUEST_TIMEOUT_SECONDS`) so unset variables are **absent** from the
   container environment. Mapping form with an empty fallback (e.g.
   `${API_KEY:-}`) is **forbidden**: it would set `API_KEY=""`, activating
   auth with an empty Bearer token (verified against `src/auth.rs`; see
   research D5).
7. **MUST NOT** include a top-level `version:` key (obsolete in Compose Spec).
8. **MUST NOT** duplicate the health check in the compose file; it is
   inherited from the image `HEALTHCHECK` (single source of truth).
9. **MUST** remain valid per `docker compose config` (CI-enforced).

## Dockerfile rules

1. The runtime stage **MUST** install `curl` (plus existing `ca-certificates`).
2. **MUST** define `HEALTHCHECK` invoking
   `curl -fsS http://127.0.0.1:8080/health` with a start period sufficient for
   first-launch model downloads (interval 30 s, timeout 5 s, retries 3,
   start period ≥ 5 min).
3. **MUST NOT** change the build stage, entrypoint, or exposed port.
4. Images published **after** this feature are required for the compose health
   check; the docs **MUST** state this caveat (older published images have no
   in-image `HEALTHCHECK`).

## Variable contract

Defined exhaustively in `data-model.md` → *Customization Surface*. Binding
rules:

- Service env names pass through **verbatim** (`MODEL`, `API_KEY`, …) — the
  compose layer introduces no renamed aliases for service settings.
  `DEFAULT_MODEL` is an optional short-syntax pass-through: unset means "first
  `MODEL` entry" (service fallback), which guarantees the default model is
  always inside the selected `MODEL` set.
- Compose-level knobs use the `MODEL2VEC_` prefix (`MODEL2VEC_IMAGE`,
  `MODEL2VEC_PORT`, `MODEL2VEC_CACHE_DIR`) and never enter the container
  environment.
- Every variable in `.env.example` **MUST** have a comment stating its default
  and effect; `.env.example` **MUST** be committed; `.env` **MUST** be
  git-ignored (FR-009).

## Rendered-config invariants (test-enforced)

`tests/compose/compose_config_test.sh` runs `docker compose --env-file /dev/null
config` (independent of any repo `.env`) and asserts:

- `MODEL` contains exactly `minishlab/potion-multilingual-128M` (first entry =
  effective default via the service's first-entry fallback) and
  `minishlab/potion-code-16M-v2` in the default path.
- `DEFAULT_MODEL` **absent** when unset (fallback rule); passed through
  verbatim when set.
- `HOME: /models` present; `HF_HOME` absent.
- `API_KEY` (and other optional vars) **absent** when unset; present verbatim
  when set via `.env` (script sets a probe value and re-renders).
- Volume source defaults to `./models`, target `/models`.
- Port mapping `127.0.0.1:8080→8080` by default (loopback host binding);
  honored `MODEL2VEC_PORT` override keeps the loopback binding.
- `restart: unless-stopped` present; no `version:` key present.

## Interaction with existing behavior

- Authentication: setting `API_KEY` protects embedding endpoints only;
  `/health`, `/ready`, `/metrics` stay public (existing auth semantics,
  spec FR-006).
- Requests behave exactly as documented in
  `specs/003-multi-model-serving/contracts/` (per-model selection, TEI
  aliases) — the compose path changes nothing about them.
