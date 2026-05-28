"""Markdown chunker. Splits onboarding/docs/*.md by H2/H3 boundaries
and then by character budget. Keeps the chapter title, the section
heading, and a Docusaurus-style anchor on every chunk so the LLM can
cite a clickable URL."""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path

from . import config

FRONTMATTER_RE = re.compile(r"^---\n(.*?)\n---\n", re.DOTALL)
TITLE_RE = re.compile(r"^title:\s*(.+)$", re.MULTILINE)
HEADING_RE = re.compile(r"^(#{2,3})\s+(.+)$", re.MULTILINE)
SLUG_PREFIX_RE = re.compile(r"^\d+-")
NON_WORD_RE = re.compile(r"[^\w\s-]")
SPACE_RE = re.compile(r"[\s_]+")


@dataclass(frozen=True)
class Chunk:
    page_slug: str
    page_title: str
    anchor: str
    heading: str
    heading_level: int
    text: str

    @property
    def url_path(self) -> str:
        anchor = f"#{self.anchor}" if self.anchor else ""
        return f"/{self.page_slug}{anchor}"


def slugify(s: str) -> str:
    s = s.lower().strip()
    s = NON_WORD_RE.sub("", s)
    s = SPACE_RE.sub("-", s)
    return s.strip("-")


def file_slug(path: Path) -> str:
    """Mirror Docusaurus's default URL: strip a leading NN- prefix.

    e.g. 09-notes-nullifiers-commitments.md -> notes-nullifiers-commitments
    index.md -> "" (the doc lives at /)."""
    stem = path.stem
    if stem == "index":
        return ""
    return SLUG_PREFIX_RE.sub("", stem)


def parse_doc(path: Path) -> tuple[str, str, str]:
    raw = path.read_text(encoding="utf-8")
    fm = FRONTMATTER_RE.match(raw)
    body = raw[fm.end() :] if fm else raw
    title = path.stem
    if fm:
        m = TITLE_RE.search(fm.group(1))
        if m:
            title = m.group(1).strip().strip("\"'")
    return file_slug(path), title, body


def _split_long(
    slug: str,
    title: str,
    anchor: str,
    heading: str,
    level: int,
    text: str,
) -> list[Chunk]:
    """Break a section that exceeds the character budget into
    overlapping chunks, preferring paragraph boundaries."""
    text = text.strip()
    if not text:
        return []
    if len(text) <= config.MAX_CHUNK_CHARS:
        return [Chunk(slug, title, anchor, heading, level, text)]
    out: list[Chunk] = []
    start = 0
    while start < len(text):
        end = min(start + config.MAX_CHUNK_CHARS, len(text))
        if end < len(text):
            nl = text.rfind("\n\n", start, end)
            if nl != -1 and nl > start + config.MAX_CHUNK_CHARS // 2:
                end = nl
        piece = text[start:end].strip()
        if piece:
            out.append(Chunk(slug, title, anchor, heading, level, piece))
        if end >= len(text):
            break
        start = max(end - config.OVERLAP_CHARS, start + 1)
    return out


def chunk_doc(path: Path) -> list[Chunk]:
    slug, title, body = parse_doc(path)
    out: list[Chunk] = []

    cur_heading = title
    cur_anchor = ""
    cur_level = 1
    cur_start = 0

    matches = list(HEADING_RE.finditer(body))
    for m in matches + [None]:  # sentinel for tail
        end = m.start() if m is not None else len(body)
        section_text = body[cur_start:end].strip()
        out.extend(
            _split_long(
                slug, title, cur_anchor, cur_heading, cur_level, section_text
            )
        )
        if m is None:
            break
        cur_level = len(m.group(1))
        cur_heading = m.group(2).strip()
        cur_anchor = slugify(cur_heading)
        cur_start = m.end()

    return out


def chunk_all(docs_dir: Path) -> list[Chunk]:
    out: list[Chunk] = []
    for path in sorted(docs_dir.glob("*.md")):
        out.extend(chunk_doc(path))
    return out
