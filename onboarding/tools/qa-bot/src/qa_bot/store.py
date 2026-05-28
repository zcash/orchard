"""SQLite + sqlite-vec persistence for embedded chunks."""

from __future__ import annotations

import sqlite3
from pathlib import Path

import sqlite_vec

from . import config


def open_db(db_path: Path) -> sqlite3.Connection:
    db = sqlite3.connect(db_path)
    db.enable_load_extension(True)
    sqlite_vec.load(db)
    db.enable_load_extension(False)
    return db


def init_schema(db: sqlite3.Connection) -> None:
    db.executescript(
        f"""
        CREATE TABLE IF NOT EXISTS chunks (
            id            INTEGER PRIMARY KEY,
            page_slug     TEXT NOT NULL,
            page_title    TEXT NOT NULL,
            anchor        TEXT NOT NULL,
            heading       TEXT NOT NULL,
            heading_level INTEGER NOT NULL,
            text          TEXT NOT NULL
        );
        CREATE VIRTUAL TABLE IF NOT EXISTS chunks_vec USING vec0(
            embedding float[{config.EMBED_DIM}]
        );
        """
    )
    db.commit()
