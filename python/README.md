# model2vec-serve - Python version

Minimal OpenAI-compatible embeddings server for [model2vec](https://github.com/minishlab/model2vec) static embedding models.

## Requirements

- [uv](https://docs.astral.sh/uv/)
- A model2vec model name or local path

## Run

```bash
uv run model2vec-serve.py --model minishlab/potion-multilingual-128M --host 0.0.0.0 --port 8080
```

Flags:

- `--model`: model name or local path to load
- `--host`: bind address, default `0.0.0.0`
- `--port`: port to listen on, default `8080`

## API

- `POST /v1/embeddings`
- `GET /v1/models`
- `GET /docs`

The embeddings endpoint accepts a single string or a list of strings. Token arrays are not supported.

### Example

```bash
curl http://localhost:8080/v1/embeddings \
	-H "Content-Type: application/json" \
	-d '{"input":"Hello world","model":"minishlab/potion-multilingual-128M","encoding_format":"float"}'
```

Responses follow the OpenAI embeddings shape and support `encoding_format` values of `float` and `base64`.
