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

## Image size

The final image is designed to stay lightweight enough to start and serve
requests with modest resource limits (around 1 CPU and 1 GiB memory, depending
on the model).
