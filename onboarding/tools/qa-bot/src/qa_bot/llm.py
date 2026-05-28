"""OpenAI-compatible chat client. Reads endpoint, model, and API key
from `qa_bot.config`. Works against OpenRouter (default), Cerebras,
Groq, Together, LM Studio, Ollama (post-0.1.39), vLLM, and any other
provider that exposes the OpenAI chat-completions schema."""

from __future__ import annotations

from typing import TypedDict

import httpx

from . import config


class Message(TypedDict):
    role: str
    content: str


class LLMError(RuntimeError):
    pass


def chat(
    messages: list[Message],
    *,
    max_tokens: int = 1024,
    temperature: float = 0.2,
) -> str:
    if not config.LLM_API_KEY:
        raise LLMError(
            "no LLM API key; set QA_BOT_API_KEY (any non-empty value"
            " works against a local LM Studio / Ollama / vLLM server)"
        )

    payload = {
        "model": config.LLM_MODEL,
        "messages": messages,
        "max_tokens": max_tokens,
        "temperature": temperature,
    }
    headers = {
        "Authorization": f"Bearer {config.LLM_API_KEY}",
        "Content-Type": "application/json",
    }
    url = f"{config.LLM_BASE_URL.rstrip('/')}/chat/completions"

    try:
        r = httpx.post(url, json=payload, headers=headers, timeout=60.0)
    except httpx.HTTPError as e:
        raise LLMError(f"network error talking to {url}: {e}") from e
    if r.status_code >= 400:
        raise LLMError(
            f"LLM returned HTTP {r.status_code}: {r.text[:200]}"
        )
    body = r.json()
    try:
        return body["choices"][0]["message"]["content"]
    except (KeyError, IndexError, TypeError) as e:
        raise LLMError(
            f"unexpected LLM response shape: {str(body)[:200]}"
        ) from e
