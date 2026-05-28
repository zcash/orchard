"""Resolved configuration. Defaults match the orchard onboarding repo
layout; every value is overridable via environment variables so the
tool can be lifted into another course without touching the code."""

from __future__ import annotations

import os
from pathlib import Path

from dotenv import load_dotenv

load_dotenv()


# Repo-relative defaults assume this file lives at
# <repo>/onboarding/tools/qa-bot/src/qa_bot/config.py.
_HERE = Path(__file__).resolve()
TOOL_ROOT = _HERE.parents[2]  # .../onboarding/tools/qa-bot/
ONBOARDING_ROOT = _HERE.parents[4]  # .../onboarding/


def _env_path(name: str, default: Path) -> Path:
    raw = os.environ.get(name)
    return Path(raw).expanduser().resolve() if raw else default


DOCS_DIR: Path = _env_path("QA_BOT_DOCS_DIR", ONBOARDING_ROOT / "docs")
DB_PATH: Path = _env_path("QA_BOT_DB_PATH", TOOL_ROOT / "data" / "qa-bot.db")
STATIC_DIR: Path = TOOL_ROOT / "static"

# Embedding model. fastembed downloads on first use to its cache dir.
# BGE-small-en-v1.5 is the smallest competitive choice: 384 dim,
# ~130 MB on disk, CPU-only.
EMBED_MODEL: str = os.environ.get(
    "QA_BOT_EMBED_MODEL", "BAAI/bge-small-en-v1.5"
)
EMBED_DIM: int = int(os.environ.get("QA_BOT_EMBED_DIM", "384"))

# Generation backend. OpenAI-compatible; works with OpenRouter, Together,
# Cerebras, Groq, LM Studio, Ollama (post-0.1.39), vLLM, etc.
LLM_API_KEY: str | None = os.environ.get("QA_BOT_API_KEY")
LLM_BASE_URL: str = os.environ.get(
    "QA_BOT_BASE_URL", "https://openrouter.ai/api/v1"
)
LLM_MODEL: str = os.environ.get(
    "QA_BOT_MODEL", "meta-llama/llama-3.3-70b-instruct"
)

# Used to build absolute URLs in the "Sources" section of answers.
SITE_BASE: str = os.environ.get(
    "QA_BOT_SITE_BASE", "https://dannywillems.github.io/orchard"
)

# Chunking. Roughly token-counted: ~3.5 chars / token for English prose,
# so 1800 chars ~ 500 tokens. 200-char overlap preserves cross-boundary
# context.
MAX_CHUNK_CHARS: int = 1800
OVERLAP_CHARS: int = 200
