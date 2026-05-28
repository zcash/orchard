"""FastAPI server: minimal HTML chat UI plus a JSON /api/ask endpoint."""

from __future__ import annotations

from fastapi import FastAPI, HTTPException
from fastapi.responses import HTMLResponse
from pydantic import BaseModel, Field

from . import config
from .index import index_size
from .llm import LLMError, chat
from .prompt import SYSTEM, build_user_message
from .retrieve import get_model, retrieve

app = FastAPI(title="qa-bot")


class AskRequest(BaseModel):
    question: str = Field(min_length=1, max_length=2000)
    k: int = Field(default=8, ge=1, le=20)


class Source(BaseModel):
    n: int
    title: str
    heading: str
    url: str
    distance: float


class AskResponse(BaseModel):
    answer: str
    sources: list[Source]


@app.on_event("startup")
def _warmup() -> None:
    # Pre-load the embedding model so the first request is not slow.
    get_model()


@app.get("/", response_class=HTMLResponse)
def index_page() -> HTMLResponse:
    html = (config.STATIC_DIR / "index.html").read_text(encoding="utf-8")
    return HTMLResponse(html)


@app.get("/api/health")
def health() -> dict[str, object]:
    return {
        "ok": True,
        "chunks_indexed": index_size(),
        "embed_model": config.EMBED_MODEL,
        "llm_model": config.LLM_MODEL,
        "llm_base_url": config.LLM_BASE_URL,
        "llm_api_key_set": bool(config.LLM_API_KEY),
    }


@app.post("/api/ask", response_model=AskResponse)
def ask(req: AskRequest) -> AskResponse:
    try:
        hits = retrieve(req.question, k=req.k)
    except RuntimeError as e:
        raise HTTPException(503, str(e)) from e

    user = build_user_message(req.question, hits)
    try:
        answer = chat(
            [
                {"role": "system", "content": SYSTEM},
                {"role": "user", "content": user},
            ]
        )
    except LLMError as e:
        raise HTTPException(502, str(e)) from e

    return AskResponse(
        answer=answer,
        sources=[
            Source(
                n=i,
                title=h.page_title,
                heading=h.heading,
                url=f"{config.SITE_BASE}{h.url_path}",
                distance=h.distance,
            )
            for i, h in enumerate(hits, 1)
        ],
    )
