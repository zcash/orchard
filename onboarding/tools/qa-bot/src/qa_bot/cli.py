"""qa-bot command-line interface."""

from __future__ import annotations

import sys

import click

from . import config
from .index import build_index, index_size
from .llm import LLMError, chat
from .prompt import SYSTEM, build_user_message
from .retrieve import Hit, retrieve


@click.group()
@click.version_option()
def cli() -> None:
    """Local RAG chatbot complementary to the Docusaurus search."""


@cli.command()
def info() -> None:
    """Show resolved configuration."""
    click.echo(f"docs_dir:        {config.DOCS_DIR}")
    click.echo(f"db_path:         {config.DB_PATH}")
    click.echo(f"embed_model:     {config.EMBED_MODEL}")
    click.echo(f"embed_dim:       {config.EMBED_DIM}")
    click.echo(f"llm_base_url:    {config.LLM_BASE_URL}")
    click.echo(f"llm_model:       {config.LLM_MODEL}")
    click.echo(f"llm_api_key_set: {bool(config.LLM_API_KEY)}")
    click.echo(f"site_base:       {config.SITE_BASE}")
    click.echo(f"chunks_indexed:  {index_size()}")


@cli.command(name="index")
def index_cmd() -> None:
    """Build the embedded chunk index from the docs directory."""
    if not config.DOCS_DIR.exists():
        click.echo(f"docs dir not found: {config.DOCS_DIR}", err=True)
        sys.exit(1)
    n = build_index()
    click.echo(f"indexed {n} chunks into {config.DB_PATH}")


def _print_hits(hits: list[Hit]) -> None:
    for i, h in enumerate(hits, 1):
        click.echo(
            f"[{i}] dist={h.distance:.4f}  {h.page_title} -- {h.heading}"
        )
        click.echo(f"    {config.SITE_BASE}{h.url_path}")
        preview = h.text.strip().splitlines()[0][:100]
        click.echo(f"    > {preview}")


@cli.command()
@click.argument("question", nargs=-1, required=True)
@click.option("-k", default=8, show_default=True, help="Top-k chunks.")
def retrieve_cmd(question: tuple[str, ...], k: int) -> None:
    """Show retrieved chunks for a question (no LLM call)."""
    hits = retrieve(" ".join(question), k=k)
    if not hits:
        click.echo("no hits")
        return
    _print_hits(hits)


cli.add_command(retrieve_cmd, name="retrieve")


@cli.command()
@click.argument("question", nargs=-1, required=True)
@click.option("-k", default=8, show_default=True, help="Top-k chunks.")
@click.option(
    "--show-sources/--no-show-sources",
    default=True,
    help="Append a Sources block to the printed answer.",
)
def ask(
    question: tuple[str, ...], k: int, show_sources: bool
) -> None:
    """Answer a question via retrieval + LLM."""
    q = " ".join(question)
    hits = retrieve(q, k=k)
    user = build_user_message(q, hits)
    try:
        answer = chat(
            [
                {"role": "system", "content": SYSTEM},
                {"role": "user", "content": user},
            ]
        )
    except LLMError as e:
        click.echo(f"error: {e}", err=True)
        sys.exit(2)
    click.echo(answer)
    if show_sources:
        click.echo("\n--- Retrieved chunks (lowest distance first) ---")
        _print_hits(hits)


@cli.command()
@click.option("--host", default="127.0.0.1", show_default=True)
@click.option("--port", default=8765, show_default=True, type=int)
def serve(host: str, port: int) -> None:
    """Run the FastAPI server with the chat UI."""
    import uvicorn

    if index_size() == 0:
        click.echo(
            "warning: index is empty; run `qa-bot index` first",
            err=True,
        )
    uvicorn.run("qa_bot.server:app", host=host, port=port, reload=False)


if __name__ == "__main__":
    cli()
