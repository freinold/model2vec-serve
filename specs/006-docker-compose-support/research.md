# Research: Docker Compose Support

All NEEDS CLARIFICATION items from the spec's Technical Context were resolved
during research; none remain. Findings below are verified against the actual
repository state (files cited) and, where marked, empirically against the local
Docker toolchain.

## Decision: Compose file location, name, and format

- **Decision**: A single `docker-compose.yml` at the repository root, following
  the current Compose Spec (no top-level `version:` key, which is obsolete and
  warns on Compose v2).
- **Rationale**: Root placement makes the launch command `docker compose up -d`
  work directly from a fresh checkout — the core requirement (US1, SC-001) —
  and mirrors where `Dockerfile` and `helm/` already live. Compose v2 is
  ubiquitous (bundled with Docker Desktop and Docker Engine ≥ 20.10).
- **Alternatives considered**:
  - `compose.yaml` (the spec-preferred name) — equivalent, but
    `docker-compose.yml` remains the more recognizable name for README/docs
    discoverability; either works with Compose v2.
  - A `deploy/compose/` subdirectory — rejected: forces `-f` flags on every
    command, hurting SC-001's "under 5 minutes" goal for no benefit.

## Decision: Image reference and override mechanism

- **Decision**: `image: ${MODEL2VEC_IMAGE:-ghcr.io/freinold/model2vec-serve:latest}`
  — the published GHCR image by default (FR-002), overridable via the
  `MODEL2VEC_IMAGE` variable (shell env or `.env`, Compose auto-loads `.env`).
- **Rationale**: `docker.yml` publishes `ghcr.io/freinold/model2vec-serve` with
  `latest`, `<semver>`, and `sha-` tags on every release; `latest` always
  corresponds to the most recent release, which is the correct default for a
  local evaluation path. Pinning is documented (set
  `MODEL2VEC_IMAGE=ghcr.io/freinold/model2vec-serve:vX.Y.Z` in `.env`).
- **Alternatives considered**:
  - Building locally from the `Dockerfile` — rejected as the default per the
    spec's assumptions (no build step in the fresh-checkout path); documented
    as an override (`MODEL2VEC_IMAGE=model2vec-serve:local` after
    `docker build`).
  - Hard-pinning a semver tag in the file — rejected: the file cannot know its
    own release version and would rot; `latest` + documented pinning keeps the
    default path working forever.

## Decision: Model cache persistence via `HOME` redirection

- **Decision**: Bind-mount a host directory (default `./models`, overridable
  via `MODEL2VEC_CACHE_DIR`) at container path `/models` and set the container
  env `HOME=/models`. Downloaded models land in
  `models/.cache/huggingface/hub` on the host and survive container recreation
  (FR-003, FR-004).
- **Rationale**: Verified in `specs/004-helm-chart-enhancements/research.md`
  against vendored crate sources: `model2vec-rs` 0.2.1 loads remote models via
  `hf_hub::api::sync::Api::new()` (hf-hub 0.4.3), whose `Cache::default()` uses
  `dirs::home_dir()/.cache/huggingface/hub`; `HF_HOME` is only honored by
  `ApiBuilder::from_env()`, which is **not** the code path. On Linux,
  `dirs::home_dir()` reads `HOME` first. Reusing the exact Helm pattern
  (`persistence.mountPath` default `/models`, `HOME` set to it) gives one
  consistent mental model across both deployment paths and reuses verified
  behavior instead of re-deriving it.
- **Alternatives considered**:
  - `HF_HOME=/models/hf` — silently ineffective with the current stack (would
    produce a volume that never receives the cache; violates FR-003).
  - Named Docker volume as the default — works, but hides the artifacts from
    the developer and complicates "inspect the downloaded files on the host"
    (US2 acceptance scenario 2); documented as an option
    (`MODEL2VEC_CACHE_DIR` accepting any path, or editing the compose file to
    a named volume), bind mount chosen as default per the spec's assumptions.
  - Mounting at `/root/.cache/huggingface` directly — hardcodes the user/home
    layout; rejected for the same reasons as in spec 004.

## Decision: Health check mechanism

- **Decision**: Add `curl` to the Dockerfile **runtime** stage and define
  `HEALTHCHECK CMD curl -fsS http://127.0.0.1:8080/health || exit 1` (with
  `--start-interval`-style defaults: 30 s interval, 5 s timeout, 3 retries, 30 s
  start period) in the Dockerfile. The compose service inherits the image
  health check automatically (no duplication in `docker-compose.yml`), which
  also benefits plain `docker run` users (FR-008).
- **Rationale**: The runtime base `debian:bookworm-slim` ships neither `curl`
  nor `wget`, so the compose file cannot HTTP-probe the published image as-is
  (Kubernetes probes run from the kubelet, which is why Helm never needed an
  in-image tool). `curl` costs ~10 MB against an image whose payload is
  dominated by models — negligible. The check targets `/health` (public,
  unauthenticated). Startup includes model downloads, so a generous start
  period prevents false negatives on first launch.
- **Alternatives considered**:
  - `bash /dev/tcp` raw TCP check (bash exists in bookworm-slim) — no image
    change, but arcane, TCP-only, and fragile across base-image updates;
    rejected per Complexity Tracking.
  - Compose-level health check in `docker-compose.yml` — duplicates the image
    definition and silently goes stale; inheriting the image `HEALTHCHECK` is
    the single-source-of-truth option.
  - No health check — rejected: violates FR-008 and loses
    depends-on/restart signaling.

## Decision: Environment variable surface (customization without editing the file)

- **Decision**: Two naming rules in `docker-compose.yml`:
  1. Service env vars pass through **verbatim** under their documented
     `src/config.rs` names: `MODEL` (mapping form with the two-model default),
     and optional vars via **short syntax** (`- DEFAULT_MODEL`, `- API_KEY`,
     `- MODEL_OWNER`, `- MODEL_ALIAS`, `- MAX_BATCH_SIZE`,
     `- MAX_INPUT_LENGTH`, `- LOG_LEVEL`, `- REQUEST_TIMEOUT_SECONDS`).
  2. Compose-level knobs (which are *not* service env vars) use a
     `MODEL2VEC_` prefix: `MODEL2VEC_IMAGE`, `MODEL2VEC_PORT` (host-side port
     mapping), `MODEL2VEC_CACHE_DIR` (host-side storage path).
  Defaults: `MODEL=minishlab/potion-multilingual-128M,minishlab/potion-code-16M-v2`
  (comma-separated; `--model`'s `value_delimiter = ','` splits env values),
  `DEFAULT_MODEL=minishlab/potion-multilingual-128M` (FR-007), host port 8080.
  An `.env.example` documents every variable (FR-009).
- **Rationale**: Verbatim pass-through means one env-var vocabulary across
  `docker run`, compose, and Helm (`env` values) — no translation layer to
  document or get wrong. The `MODEL2VEC_` prefix marks the three variables
  consumed by the compose file itself, avoiding collisions with the service's
  own namespace.
- **Alternatives considered**:
  - Mapping form with `${API_KEY:-}` for optional vars — **rejected on
    correctness**: compose then always sets `API_KEY=""`; clap's `env` yields
    `Some("")` and `src/auth.rs` activates the auth layer matching an empty
    Bearer token (broken auth out of the box). Same trap for
    `DEFAULT_MODEL=""` (default-model resolution would target an empty id).
  - Hard-coding all settings in the file — rejected: violates FR-005/FR-006/
    FR-009 and the spec's "fully fledged customization" story (US3).

## Decision: Compose v2 optional-env semantics (empirically verified)

- **Decision**: Rely on the short syntax (`- VAR`) for optional variables.
- **Rationale**: **Empirically verified today against Docker Compose v5.5.0**
  with a probe project: (a) with the variable set in the host environment or
  `.env`, `docker compose config` renders `VAR: <value>` and the container
  receives it; (b) with the variable unset, config renders `VAR: null`, and a
  real `docker compose run` confirms the variable is **absent** from the
  container environment (`env | grep API_KEY` → empty). This "absent when
  unset" semantics is exactly what the empty-string trap analysis above
  requires, and it works uniformly from both shell env and `.env`.
- **Alternatives considered**: `env_file` with `required: false` (Compose ≥
  2.24) — viable but adds a second mechanism with different interpolation
  rules for no benefit over short syntax; rejected to keep one mechanism.

## Decision: Documentation placement and wiring

- **Decision**: New dedicated page `docs/deployment/compose.md`; sidebar entry
  in `docs/.vitepress/config.ts` under **Deployment** between Docker and Helm;
  README gains a "Docker Compose" section (after **Container**, before
  **Helm**) plus a Features bullet; `docs/deployment/docker.md` gains a
  cross-link to the compose page (FR-010, FR-011).
- **Rationale**: Follows the established docs architecture exactly (spec 004
  updated `helm.md` + sidebar the same way). The Deployment sidebar order
  (Docker → Compose → Helm) reflects escalating deployment complexity:
  single container → local multi-model stack → Kubernetes.
- **Alternatives considered**:
  - Extending `docs/deployment/docker.md` with a compose section — rejected:
    the user asked for "fully fledged … docs"; a dedicated page can carry the
    full lifecycle (launch, persistence, customization, teardown,
    troubleshooting) without bloating the Docker page, and gets its own
    search-index entry.
  - README-only docs — rejected: README cannot hold the full guide and the
    docs site is the canonical reference (linked from README).

## Decision: Automated validation and CI wiring

- **Decision**: New `tests/compose/compose_config_test.sh` (mirroring
  `tests/helm/lint_test.sh` conventions: `set -euo pipefail`, repo-root
  resolution) asserting the rendered `docker compose config` output: exactly
  two models in `MODEL`, `DEFAULT_MODEL` set to the multilingual model,
  `HOME=/models`, the `/models` bind mount, inherited/declared health check,
  `restart: unless-stopped`, and port mapping. Wired into
  `.github/workflows/ci.yml` as a lightweight job/step using the preinstalled
  Docker CLI (`docker compose config` is fully offline — no image pull).
- **Rationale**: Constitution II demands automated, CI-visible tests; the
  Helm feature validated with offline template/config tests the same way
  (spec 004). `docker compose config` validates schema *and* interpolation,
  catching regressions in the variable surface without any network or model
  downloads, keeping CI fast.
- **Alternatives considered**:
  - Full `docker compose up` smoke test in CI (pull image, wait healthy,
  request embeddings) — the strongest check but adds minutes, network
  flakiness, and multi-hundred-MB pulls to every CI run; rejected for CI,
  covered instead by the manual quickstart (and optionally runnable locally).
  - Testing only in the docs workflow — rejected: docs builds don't run
  compose and validation belongs with the other deployment tests.

## Decision: Lifecycle settings

- **Decision**: `restart: unless-stopped` (survives crashes and host reboots
  once Docker starts, but respects an explicit `docker compose stop` — FR-008),
  `stop_grace_period: 30s` to give in-flight embedding requests room to finish
  during `docker compose stop` (FR-013), container port fixed at 8080
  (matches `EXPOSE 8080`; the service binds `0.0.0.0` by default), host side
  overridable via `MODEL2VEC_PORT`.
- **Rationale**: `unless-stopped` is the standard choice for developer
  workstations (unlike `always`, it won't resurrect a stack the user
  deliberately stopped). 30 s matches the service's default
  `REQUEST_TIMEOUT_SECONDS`, so a graceful stop can drain at least one
  in-flight worst-case request.
- **Alternatives considered**: `always` (ignores deliberate stops), `no`
  (no crash recovery — violates FR-008), container-side `PORT` override
  (pointless: only the host mapping matters and the image exposes 8080).
