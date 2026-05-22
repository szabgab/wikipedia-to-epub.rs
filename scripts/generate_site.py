#!/usr/bin/env python3
from __future__ import annotations

import argparse
from pathlib import Path

import yaml
from jinja2 import Environment, FileSystemLoader, select_autoescape


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate the GitHub Pages site")
    parser.add_argument("--books-dir", default="books", type=Path)
    parser.add_argument("--template", default="templates/site/index.html.j2", type=Path)
    parser.add_argument("--output-dir", default="site", type=Path)
    parser.add_argument("--artifact-url", required=True)
    return parser.parse_args()


def load_examples(books_dir: Path) -> list[dict[str, str]]:
    examples: list[dict[str, str]] = []

    for path in sorted(books_dir.glob("*.yaml")):
        content = path.read_text(encoding="utf-8")
        # Validate that examples are valid YAML before publishing.
        yaml.safe_load(content)
        examples.append(
            {
                "name": path.name,
                "content": content,
            }
        )

    return examples


def main() -> None:
    args = parse_args()

    env = Environment(
        loader=FileSystemLoader(args.template.parent),
        autoescape=select_autoescape(["html", "xml"]),
    )
    template = env.get_template(args.template.name)

    rendered = template.render(
        artifact_url=args.artifact_url,
        examples=load_examples(args.books_dir),
    )

    args.output_dir.mkdir(parents=True, exist_ok=True)
    (args.output_dir / "index.html").write_text(rendered, encoding="utf-8")


if __name__ == "__main__":
    main()
