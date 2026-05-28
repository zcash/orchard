"""Embed a question and retrieve the most relevant chunks."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import numpy as np
from fastembed import TextEmbedding

from . import config
from .store import open_db


@dataclass(frozen=True)
class Hit:
    chunk_id: int
    page_slug: str
    page_title: str
    anchor: str
    heading: str
    text: str
    distance: float

    @property
    def url_path(self) -> str:
        anchor = f"#{self.anchor}" if self.anchor else ""
        return f"/{self.page_slug}{anchor}"


_MODEL: TextEmbedding | None = None


def get_model() -> TextEmbedding:
    global _MODEL
    if _MODEL is None:
        _MODEL = TextEmbedding(model_name=config.EMBED_MODEL)
    return _MODEL


def retrieve(
    query: str, k: int = 8, db_path: Path | None = None
) -> list[Hit]:
    db_path = db_path or config.DB_PATH
    if not db_path.exists():
        raise RuntimeError(
            f"index not built at {db_path}; run `qa-bot index`"
        )

    model = get_model()
    q_emb = next(iter(model.embed([query])))
    q_bytes = np.asarray(q_emb).astype(np.float32, copy=False).tobytes()

    db = open_db(db_path)
    try:
        rows = db.execute(
            """
            SELECT c.id, c.page_slug, c.page_title, c.anchor,
                   c.heading, c.text, v.distance
              FROM chunks_vec v
              JOIN chunks c ON c.id = v.rowid
             WHERE v.embedding MATCH ? AND k = ?
             ORDER BY v.distance
            """,
            (q_bytes, k),
        ).fetchall()
    finally:
        db.close()

    return [
        Hit(
            chunk_id=r[0],
            page_slug=r[1],
            page_title=r[2],
            anchor=r[3],
            heading=r[4],
            text=r[5],
            distance=float(r[6]),
        )
        for r in rows
    ]
