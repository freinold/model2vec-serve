# Implementation Plan: Docker Compose Support

**Branch**: `006-docker-compose-support` | **Date**: 2026-08-31 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/006-docker-compose-support/spec.md`

## Summary

Provide a fully fledged local deployment path via Docker Compose: a root-level
`docker-compose.yml` that launches the published GHCR image serving two models
(`minishlab/potion-multilingual-128M` as default and
`minishlab/potion-code-16M-v2`), persists the Hugging Face model cache in a
host-mounted directory using the same `HOME` redirection pattern as the Helm
chart, defines a health check and restart policy, and exposes every service
setting through environment variables (with an `.env.example`). Documentation
ships as a dedicated VitePress page (`docs/deployment/compose.md`) plus a
README section and cross-links. Automated validation mirrors the Helm test
pattern with a `docker compose config`-based test script. No service code or
API behavior changes.

## Technical Context

**Language/Version**: Rust 1.98 (image builder, MSRV 1.85 — unchanged); Docker Compose Spec targeting Compose v2 CLI (verified against Docker Compose v5.5.0)

**Primary Dependencies**: `ghcr.io/freinold/model2vec-serve` published image (built by `.github/workflows/docker.yml` on releases; tags `latest`, `<semver>`, `sha-`); existing service env configuration (`src/config.rs`: `MODEL`, `DEFAULT_MODEL`, `API_KEY`, `MAX_BATCH_SIZE`, `MAX_INPUT_LENGTH`, `LOG_LEVEL`, `REQUEST_TIMEOUT_SECONDS`, `MODEL_OWNER`, `MODEL_ALIAS`); runtime base `debian:trixie-slim` (pinned by OCI index digest)

**Storage**: Host bind-mount directory (default `./models`) mounted at `/models` with `HOME=/models`, so the model cache lands in `models/.cache/huggingface/hub` — identical to the Helm persistence pattern (`hf-hub` 0.4.3 sync API ignores `HF_HOME`; `dirs::home_dir()` reads `HOME` first; see `specs/004-helm-chart-enhancements/research.md`)

**Testing**: New `tests/compose/compose_config_test.sh` validating the rendered compose configuration offline (`docker compose config`), mirroring `tests/helm/*.sh`; CI wiring in `.github/workflows/ci.yml`; end-to-end validation via this feature's `quickstart.md`; existing `cargo test` suite unaffected

**Target Platform**: Developer workstations with Docker (Linux, macOS, Windows via Docker Desktop); amd64 (image platform published by `docker.yml`)

**Performance Goals**: Warm-cache restart reaches readiness in < 30 s (disk model load only, no downloads); unhealthy state detected by the health check within ~90 s (three consecutive failed checks at a 30 s interval, after the 300 s start period); first-launch download time is network-bound and out of scope

**Constraints**: No new runtime dependencies beyond `curl` in the Dockerfile runtime stage (for the health check); compose file must work with the published image (no local build step); service API/error contract unchanged (FR-014)

**Scale/Scope**: Single compose service, two models; ~10 files touched (compose file, `.env.example`, Dockerfile, `.gitignore`, README, 2 docs pages, VitePress sidebar, 1 test script, CI workflow)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Code Quality | PASS | Compose file, `.env.example`, and test script follow existing repo conventions (mirrors `tests/helm/`); no dead config; every setting documented; docs updated per Development Workflow |
| II. Test Coverage | PASS | New automated test (`tests/compose/compose_config_test.sh`) validates the deployment contract (two models, default model, `HOME`, volume, health check, restart policy) and fails before the change (no compose file exists); no Rust/Python code touched so coverage percentages unaffected; contract tests unaffected (FR-014) |
| III. API Conformity | PASS | No endpoints, response shapes, or error codes changed; service started by compose is byte-identical to the service documented in existing contracts |
| IV. Simplicity Over Complexity | PASS | Single compose service reusing the service's existing env configuration (no new abstraction); one container serving both models rather than two containers; two justified deviations recorded in Complexity Tracking (curl in runtime image; short-syntax env pass-through) |
| V. Performance Focus | PASS | Deployment-level performance goals defined (warm restart < 30 s to ready; health-check failure detection ~90 s after the 300 s start period); no hot-path code changed, so no new benchmarks required — justified in Complexity Tracking |

**Post-Phase-1 re-check**: All gates still PASS. Phase 1 contracts fixed the
env-var surface (`contracts/compose.md`) and documentation requirements
(`contracts/docs.md`) without introducing additional complexity.

## Project Structure

### Documentation (this feature)

```text
specs/006-docker-compose-support/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── compose.md       # Compose deployment contract (env surface, volumes, health)
│   └── docs.md          # Documentation contract (README + docs site)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
docker-compose.yml            # NEW: compose deployment (published image, 2 models, volume, health)
.env.example                  # NEW: documented customization variables (copy to .env)
Dockerfile                    # MODIFIED: runtime stage gains curl + HEALTHCHECK instruction
.gitignore                    # MODIFIED: ignore models/ cache directory
README.md                     # MODIFIED: "Docker Compose" section + Features bullet
docs/
├── .vitepress/config.ts      # MODIFIED: sidebar entry for the compose page
├── deployment/compose.md     # NEW: dedicated compose documentation page
└── deployment/docker.md      # MODIFIED: cross-link to compose page
tests/compose/
└── compose_config_test.sh    # NEW: offline validation of the rendered compose config
.github/workflows/ci.yml      # MODIFIED: run the compose config test in CI
```

**Structure Decision**: Deployment-artifact feature in the repository root,
matching the existing layout conventions: compose file next to `Dockerfile`
(like `helm/` sits beside it), tests mirrored after `tests/helm/`, docs under
`docs/deployment/` wired into the existing VitePress sidebar, and CI extended
in the existing workflow file rather than a new one (same rationale as spec
004's CI decision).

## Complexity Tracking

> Recorded deviations from the simplest possible approach, per Constitution IV.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| `curl` added to the Dockerfile runtime stage (~10 MB) | Enables an HTTP-level `HEALTHCHECK /health` inherited by compose and plain `docker run` alike; the runtime image ships neither curl nor wget | `bash /dev/tcp` TCP connect check inside compose — rejected: shell-arcane, cannot assert an HTTP 200 from `/health`, and silently breaks if the base image or shell changes; TCP-accept also cannot distinguish "listener up" from "service healthy" for future changes |
| Short-syntax env pass-through (`- API_KEY`) for optional variables instead of mapping form | Mapping form with `${API_KEY:-}` always sets an *empty-string* env var; clap then yields `Some("")` and `src/auth.rs` activates auth expecting an empty Bearer token — broken auth by default | Intercepting/conditionals in compose — not expressible; verified empirically (research.md D5) that short syntax omits unset variables entirely from the container environment, which is the exact semantics needed |
| No new performance benchmarks | Constitution V requires measured performance, but this feature changes no request-path or model-loading code | Adding benchmarks would measure unchanged code paths; deployment-level budgets (restart readiness, health-check detection) are defined instead and verified in quickstart.md |
