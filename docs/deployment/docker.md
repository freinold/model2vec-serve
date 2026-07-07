# Docker

`model2vec-serve` ships with a multi-stage `Dockerfile` that produces a small
runtime image.

## Build

```bash
docker build -t model2vec-serve:latest .
```

The build stage uses `rust:1.85-slim` and installs the native dependencies
required by `model2vec-rs`. The runtime stage uses `debian:bookworm-slim` and
only the compiled binary plus CA certificates.

## Run

```bash
docker run -p 8080:8080 \
  -e MODEL=minishlab/potion-base-2M \
  model2vec-serve:latest
```

### With an API key

```bash
docker run -p 8080:8080 \
  -e MODEL=minishlab/potion-base-2M \
  -e API_KEY=my-secret-key \
  model2vec-serve:latest
```

### With extra CLI arguments

Pass them after the image name:

```bash
docker run -p 8080:8080 \
  -e MODEL=minishlab/potion-base-2M \
  model2vec-serve:latest \
  --max-batch-size 128 \
  --log-level debug
```

### With a local model

Mount the model directory into the container and point `--model` at the mount
path:

```bash
docker run -p 8080:8080 \
  -v /path/to/local/model:/models/my-model \
  -e MODEL=/models/my-model \
  model2vec-serve:latest
```

## GitHub Container Registry

Pre-built images are published to the GitHub Container Registry (GHCR) on every
GitHub release. The image name is derived from the repository:

```text
ghcr.io/freinold/model2vec-serve
```

### Pull a released image

Replace `<version>` with the desired release tag (for example `v0.1.0`):

```bash
docker pull ghcr.io/freinold/model2vec-serve:<version>
```

The `latest` tag is also pushed for the most recent release, but pinning to a
semantic version is recommended for reproducible deployments.

### Run the published image

```bash
docker run -p 8080:8080 \
  -e MODEL=minishlab/potion-base-2M \
  ghcr.io/freinold/model2vec-serve:<version>
```

## Image size

The final image is designed to stay lightweight enough to start and serve
requests with modest resource limits (around 1 CPU and 1 GiB memory, depending
on the model).

## Release process

Releases are automated with `release-plz`:

1. Commit changes using [Conventional Commits](https://www.conventionalcommits.org/).
2. `release-plz` opens a release PR that bumps `Cargo.toml` and updates
   `CHANGELOG.md`.
3. After the release PR is merged, `release-plz` creates a GitHub release and git
   tag.
4. The `docker.yml` workflow builds and pushes the container image for the new
   tag to GHCR.
