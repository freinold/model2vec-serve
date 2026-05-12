# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "model2vec",
#   "fastapi",
#   "uvicorn[standard]",
# ]
# ///

import argparse
import base64
import struct
from contextlib import asynccontextmanager
from typing import Literal

import uvicorn
from fastapi import FastAPI
from fastapi.responses import JSONResponse, RedirectResponse
from pydantic import BaseModel, Field
from model2vec import StaticModel

MAX_TOKENS = 512  # model2vec truncates at 512 tokens for all models


def parse_args():
    parser = argparse.ArgumentParser(
        description="Serve a model2vec model via an OpenAI-compatible embeddings API"
    )
    parser.add_argument(
        "--model",
        default="minishlab/potion-multilingual-128M",
        help="Model name or local path",
    )
    parser.add_argument("--host", default="0.0.0.0")
    parser.add_argument("--port", type=int, default=8080)
    return parser.parse_args()


args = parse_args()
loaded_model: StaticModel | None = None


@asynccontextmanager
async def lifespan(app: FastAPI):
    global loaded_model
    print(f"Loading model: {args.model}")
    loaded_model = StaticModel.from_pretrained(args.model)
    print("Model loaded.")
    yield


app = FastAPI(
    lifespan=lifespan,
    docs_url="/docs",
    redoc_url="/redoc",
)


@app.get("/", include_in_schema=False)
def root_redirect():
    return RedirectResponse(url="/docs")


class EmbeddingRequest(BaseModel):
    input: str | list[str] = Field(
        description=(
            "Input text to embed. Can be a single string, or an array of strings for "
            "batch embedding. Token arrays are not supported. "
            f"Each input is truncated to {MAX_TOKENS} tokens — split long documents "
            "into smaller chunks if you need full coverage."
        ),
        examples=[["This text will get embedded.", "This one as well."]],
    )
    model: str = Field(
        description="ID of the model to use. Must match the model name the server was started with.",
        default=args.model,
    )
    encoding_format: Literal["float", "base64"] = Field(
        default="float",
        description='Format for the returned embeddings. "float" returns a list of floats; "base64" returns a base64-encoded string of packed IEEE 754 single-precision floats.',
    )


def inputs_to_texts(input: str | list[str]) -> list[str] | None:
    """Convert input to list of strings, or return None if it's token arrays (unsupported)."""
    if isinstance(input, str):
        return [input]
    elif isinstance(input, list) and len(input) > 0:
        if isinstance(input[0], str):
            return input
    return None  # token arrays — not supported


def count_tokens(texts: list[str]) -> int:
    """Count tokens using the model's own tokenizer, capped at MAX_TOKENS per input."""
    total = 0
    for text in texts:
        ids = loaded_model.tokenizer.encode(text, add_special_tokens=False).ids
        total += min(len(ids), MAX_TOKENS)
    return total


def encode_base64(vec: list[float]) -> str:
    return base64.b64encode(struct.pack(f"{len(vec)}f", *vec)).decode("utf-8")


@app.post("/v1/embeddings")
def create_embeddings(req: EmbeddingRequest):
    texts = inputs_to_texts(req.input)
    if texts is None:
        return JSONResponse(
            status_code=422,
            content={
                "error": {
                    "message": "Token array inputs are not supported; please pass strings.",
                    "type": "invalid_request_error",
                }
            },
        )

    vecs = loaded_model.encode(texts, show_progress_bar=False)

    token_count = count_tokens(texts)

    data = []
    for i, vec in enumerate(vecs):
        vec_list = vec.tolist()
        embedding = (
            encode_base64(vec_list) if req.encoding_format == "base64" else vec_list
        )
        data.append({"object": "embedding", "index": i, "embedding": embedding})

    return {
        "object": "list",
        "model": req.model,
        "data": data,
        "usage": {"prompt_tokens": token_count, "total_tokens": token_count},
    }


@app.get("/v1/models")
def list_models():
    return {
        "object": "list",
        "data": [{"id": args.model, "object": "model", "owned_by": "model2vec"}],
    }


if __name__ == "__main__":
    uvicorn.run(app, host=args.host, port=args.port)
