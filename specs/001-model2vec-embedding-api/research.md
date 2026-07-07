# Research: model2vec-embedding-api

## Decision: Model inference crate

**Decision**: Use the official `model2vec-rs` crate from crates.io (current
version 0.2.1) for static embedding inference.

**Rationale**:
- It is the official Rust implementation from MinishLab, the creators of
  Model2Vec, optimized for CPU inference with models stored in `safetensors`.
- It supports f32, f16, and i8 weight types and batch encoding.
- It exposes a simple API: `StaticModel::from_pretrained(repo_or_path, token,
  normalize, subfolder)` and `model.encode(sentences)` returning `Vec<Vec<f32>>`.
- It supports loading from the Hugging Face Hub, local paths, and raw bytes.
- Performance is ~1.7× faster than the Python implementation according to the
  upstream benchmark.
- It matches the project goal of a lightweight, containerized Rust service.

**Feature flags**:
- Default features (`onig`, `hf-hub`) are appropriate for the standard container
  build and allow Hub downloads.
- For air-gapped deployments that only load mounted local models, the Helm chart
  can build or run with `--no-default-features --features onig,local-only` to
  avoid network dependencies.

**Alternatives considered**:
- `model2vec` (Swiftosis) — unofficial port with a similar API but fewer
  downloads and no official maintenance.
- `candle` — more general ML framework; adds significant dependency surface for a
  single model2vec inference task.
- `ort` (ONNX Runtime) — would require exporting models to ONNX and adds a large
  native dependency; unnecessary when a native Rust crate exists.
- Shell out to the Python implementation — would bloat the container and defeat
  the Rust performance goals.

## Decision: Web framework and OpenAPI

**Decision**: Use `axum` for the HTTP layer and `utoipa` for OpenAPI generation.

**Rationale**:
- Axum integrates cleanly with Tower middleware (tracing, timeout, CORS) and
  provides typed extractors/handlers.
- Utoipa derives OpenAPI specs from annotated handlers and `ToSchema` types,
  keeping `/docs` in sync with code and satisfying the API Conformity principle.
- Both are widely used in the Rust ecosystem and align with general Rust best
  practices.

## Decision: Configuration model

**Decision**: Parse all configuration as CLI arguments via `clap`, then mirror
every argument into Helm `values.yaml` and map it to container args or
environment variables.

**Rationale**:
- CLI args are explicit, easy to document, and simple to map to Kubernetes
  container specifications.
- Helm `values.yaml` becomes the single source of truth for operators.
- Volume mounts for local models are supported through standard Helm
  `extraVolumes` / `extraVolumeMounts` values.

## Decision: Observability stack

**Decision**: Use `tracing` + `tracing-subscriber` for structured JSON logs with a
request-correlation ID, and `metrics` + `metrics-exporter-prometheus` for
Prometheus-compatible metrics.

**Rationale**:
- `tracing` is the idiomatic Rust logging framework and supports structured
  output with custom fields such as `request_id`.
- `metrics-exporter-prometheus` exposes a `/metrics` endpoint in the format
  expected by Kubernetes monitoring stacks.
- No external observability agent is required inside the container.

## Decision: API key authentication

**Decision**: Implement API key auth as a Tower layer that checks the
`Authorization: Bearer <key>` header against a configured secret. Auth can be
disabled by omitting the API key.

**Rationale**:
- Bearer tokens are the de-facto standard for both OpenAI and TEI clients.
- A Tower layer keeps auth cross-cutting and testable.
- The key is supplied via CLI arg / environment variable / Kubernetes secret and
  is never hard-coded.

## Decision: Container and deployment packaging

**Decision**: Build a distroless or Debian-slim multi-stage Docker image and
package it with a Helm chart under `helm/model2vec-serve`.

**Rationale**:
- A multi-stage build keeps the final image small, supporting the lightweight
  container goal.
- Helm is the requested deployment method and supports volume mounts for model
  files or secrets.
- A `values.yaml` with documented defaults keeps operator configuration simple.

## Decision: CI/CD image publishing

**Decision**: Use a GitHub Actions workflow that builds the `Dockerfile` and
pushes the resulting image to the GitHub Container Registry (GHCR) on every
tagged release.

**Rationale**:
- GHCR is free for public repositories, integrated with GitHub permissions, and
  requires no extra secrets beyond the auto-generated `GITHUB_TOKEN`.
- Triggering on `release: [published]` (created from a git tag) produces
  immutable, versioned images that operators can pin in Helm `image.tag`.
- The official `docker/build-push-action`, `docker/metadata-action`, and
  `docker/login-action` actions are maintained by Docker and are the de-facto
  standard for building and tagging images.
- `docker/metadata-action` automatically derives `latest`, semver, and
  type=sha tags from git refs, removing manual tag bookkeeping.
- Artifact attestations (`actions/attest`) increase supply-chain security by
  publishing signed provenance for the image digest.

**Workflow outline**:
- `actions/checkout@v4`
- `docker/setup-buildx-action@v3`
- `docker/login-action@v3` to GHCR with `secrets.GITHUB_TOKEN`
- `docker/metadata-action@v5` for `ghcr.io/${{ github.repository }}` tags
- `docker/build-push-action@v6` with `push: true`, multi-platform via
  `platforms: linux/amd64,linux/arm64`
- `actions/attest@v4` to generate an attestation for the pushed digest

**Permissions**:
- `packages: write` to push the image.
- `contents: read` to check out the repo.
- `attestations: write` and `id-token: write` for artifact attestations.

**Alternatives considered**:
- Trigger on every push to `main` with a `latest` tag only — rejected because it
  provides no stable versioned images and makes rollbacks harder.
- Trigger on git tags directly (`on: push: tags: ['v*']`) — acceptable, but
  `release: published` is more explicit and integrates with GitHub releases.
- Push to Docker Hub — rejected to avoid managing a second registry and secrets
  when GHCR is sufficient for an open-source project.

## Decision: Semantic release automation for Rust

**Decision**: Use `release-plz` to automate changelog generation, version
bumping, git tagging, GitHub releases, and (optionally) crates.io publishing.

**Rationale**:
- `release-plz` is purpose-built for Rust/Cargo projects and understands
  `Cargo.toml` versioning and workspace layouts.
- It parses Conventional Commits to determine the next SemVer version.
- It runs `cargo-semver-checks` to detect API-breaking changes automatically.
- It opens a release PR with updated `Cargo.toml` versions and `CHANGELOG.md`,
  giving maintainers a review checkpoint before a release is published.
- A separate `release-plz release` job publishes the GitHub release and tag
  after the PR is merged.

**Configuration**:
- GitHub workflow at `.github/workflows/release.yml` with two jobs:
  `release-plz-release` (publish on push to `main`) and
  `release-plz-pr` (open/update release PR on push to `main`).
- `.release-plz.toml` to disable crates.io publishing if the crate is not meant
  to be published, or to configure changelog formatting.
- Repository secret: `GITHUB_TOKEN` is sufficient for GitHub releases; add
  `CARGO_REGISTRY_TOKEN` only if publishing to crates.io.

**Alternatives considered**:
- `semantic-release` (Node.js-based) — would introduce a Node dependency and
  require a custom Rust plugin; rejected in favor of a Rust-native tool.
- `cargo-release` — excellent for manual releases but does not automatically
  create release PRs from Conventional Commits; rejected for less automation.
- Manual versioning — rejected because it is error-prone and inconsistent with
  the Constitution's release workflow requirement.
