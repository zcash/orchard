"""RAG prompt builder. Forces grounded answers with inline citations
and a refusal path when retrieval is empty or off-topic."""

from __future__ import annotations

from . import config
from .retrieve import Hit


SYSTEM = """You answer questions about a software codebase by quoting \
and citing the provided documentation chunks. You do NOT rely on \
background knowledge; if the chunks do not contain the answer, you \
say so.

Rules:
1. Every claim must be supported by one of the provided chunks. Cite \
each claim with [n] where n is the chunk number.
2. If the chunks are insufficient or off-topic, say "I cannot answer \
this from the provided documentation." and stop. Do not guess.
3. Quote technical names exactly. Do not paraphrase identifiers, \
function names, or domain vocabulary.
4. Keep answers concise. Prefer short paragraphs and bullets.
5. End every answer with a "Sources" list of the chunks you cited, \
in the format `[n] <Page title> -- <Heading> -- <URL>`.

The site this is built from is not authoritative documentation. \
Treat the chunks as the only ground truth available for this turn.
"""


def build_user_message(question: str, hits: list[Hit]) -> str:
    parts = [f"Question: {question}", ""]
    if not hits:
        parts.append("No chunks were retrieved.")
        return "\n".join(parts)
    parts.append("Chunks:")
    for i, h in enumerate(hits, 1):
        parts.append("")
        parts.append(f"[{i}] {h.page_title} -- {h.heading}")
        parts.append(f"URL: {config.SITE_BASE}{h.url_path}")
        parts.append("")
        parts.append(h.text)
        parts.append("---")
    return "\n".join(parts)
