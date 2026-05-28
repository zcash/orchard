"""Build the embedded chunk index from the docs directory."""

from __future__ import annotations

from pathlib import Path

import numpy as np
from fastembed import TextEmbedding

from . import config
from .chunk import Chunk, chunk_all
from .store import init_schema, open_db


def _vec_bytes(arr: np.ndarray) -> bytes:
    return arr.astype(np.float32, copy=False).tobytes()


def build_index(
    docs_dir: Path | None = None, db_path: Path | None = None
) -> int:
    docs_dir = docs_dir or config.DOCS_DIR
    db_path = db_path or config.DB_PATH
    db_path.parent.mkdir(parents=True, exist_ok=True)
    if db_path.exists():
        db_path.unlink()

    chunks = chunk_all(docs_dir)
    if not chunks:
        raise RuntimeError(f"no markdown chunks under {docs_dir}")

    model = TextEmbedding(model_name=config.EMBED_MODEL)
    texts = [c.text for c in chunks]
    embeddings = list(model.embed(texts))

    db = open_db(db_path)
    try:
        init_schema(db)
        cur = db.cursor()
        for chunk, emb in zip(chunks, embeddings, strict=True):
            cur.execute(
                "INSERT INTO chunks (page_slug, page_title, anchor,"
                " heading, heading_level, text) VALUES (?, ?, ?, ?, ?, ?)",
                (
                    chunk.page_slug,
                    chunk.page_title,
                    chunk.anchor,
                    chunk.heading,
                    chunk.heading_level,
                    chunk.text,
                ),
            )
            cur.execute(
                "INSERT INTO chunks_vec (rowid, embedding) VALUES (?, ?)",
                (cur.lastrowid, _vec_bytes(np.asarray(emb))),
            )
        db.commit()
    finally:
        db.close()

    return len(chunks)


def index_size(db_path: Path | None = None) -> int:
    db_path = db_path or config.DB_PATH
    if not db_path.exists():
        return 0
    db = open_db(db_path)
    try:
        return db.execute("SELECT COUNT(*) FROM chunks").fetchone()[0]
    finally:
        db.close()


__all__ = ["build_index", "index_size", "Chunk"]
