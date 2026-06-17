# Copilot instructions for `wikipedia-to-epub`

## Build and test commands

Use Cargo from the repository root:

```bash
cargo build
cargo test
cargo test render_wikitext_handles_sections_links_and_lists
cargo fmt --check
```

Run the CLI with the sample config:

```bash
cargo run -- examples/korea.yaml
cargo run -- examples/korea.yaml --local pages
```

Generate the GitHub Pages index locally:

```bash
uv run tools/generate_site.py
```

## High-level architecture

This repository is currently a single-binary Rust CLI with all implementation in `src/main.rs`.

- `BookConfig`, `Metadata`, and `PageResponse` deserialize the input YAML config and Wikipedia page dump JSON.
- `run()` is the top-level flow: parse the config path and optional `--local <dir>` flag, read the config, load each article from either the Wikipedia parse API or the specified local fixture directory, render chapters, then write the EPUB.
- `WikipediaApiPageSource` is the default runtime article source. `FixturePageSource` is used both by tests and by `--local` runs to read cached JSON dumps from disk.
- `find_page_path()` resolves article names to fixture files in `pages/` using several filename variants plus a normalized fallback that ignores case and punctuation.
- `render_wikitext()` converts Wikipedia `parse.wikitext["*"]` content into simplified XHTML by stripping templates/tables and cleaning inline wiki markup before the EPUB is assembled.
- `write_epub()` builds the final archive directly with the `zip` crate. It writes the required EPUB pieces (`mimetype`, `META-INF/container.xml`, `OEBPS/content.opf`, `OEBPS/toc.ncx`, `OEBPS/nav.xhtml`, chapter files, and a minimal stylesheet) without an external EPUB library.
- `scripts/generate_site.py` renders `templates/site/index.html.j2` with Jinja to publish a GitHub Pages index with links to compiled binary downloads.

Important repo context from the docs and code together:

- `examples/*.yaml` are book configs that drive output file name, metadata, and article order.
- `pages/*.json` are cached Wikipedia API `parse` responses used by tests as offline fixtures.
- `books/*.yaml` are sample book configs; CI does not generate or publish EPUBs from them.

## Key conventions

- Keep error handling explicit through `AppError` and `AppResult<T>`; this codebase does not use broad fallbacks or silent skips for missing inputs.
- Tests are inline unit tests at the bottom of `src/main.rs`, not in separate files.
- The EPUB writer is intentionally minimal and hand-built; if you change packaging, preserve the current structure and the uncompressed `mimetype` entry.
- Wikitext cleanup is heuristic and regex/string based. Reuse `cleanup_inline_markup()`, `parse_heading()`, and `strip_balanced_sections()` instead of adding parallel ad hoc cleanup paths.
- Article lookup is designed to tolerate common filename variations (`space`, `_`, `-`, case changes). Keep that behavior when changing how configs map to files.
- The project currently excludes embedded assets such as images and external stylesheets from the generated book, matching the DEVELOPMENT.md’s stated scope.

## Documentation

As new instructions are given to add features, change design or improve coding style, update this document as well.
