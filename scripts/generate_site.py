#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "jinja2",
# ]
# ///
from __future__ import annotations

import argparse
from pathlib import Path

from jinja2 import Environment, FileSystemLoader, select_autoescape

BINARY_DOWNLOADS = [
    {
        "label": "Linux",
        "href": "downloads/wikipedia-to-epub-linux/wikipedia-to-epub",
    },
    {
        "label": "macOS",
        "href": "downloads/wikipedia-to-epub-macos/wikipedia-to-epub",
    },
    {
        "label": "Windows",
        "href": "downloads/wikipedia-to-epub-windows/wikipedia-to-epub.exe",
    },
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate the GitHub Pages site")
    parser.add_argument("--template", default="templates/site/index.html.j2", type=Path)
    parser.add_argument("--output-dir", default="site", type=Path)
    return parser.parse_args()


def main() -> None:
    args = parse_args()

    env = Environment(
        loader=FileSystemLoader(args.template.parent),
        autoescape=select_autoescape(["html", "xml"]),
    )
    template = env.get_template(args.template.name)

    rendered = template.render(
        binary_downloads=BINARY_DOWNLOADS,
    )

    args.output_dir.mkdir(parents=True, exist_ok=True)
    (args.output_dir / "index.html").write_text(rendered, encoding="utf-8")


if __name__ == "__main__":
    main()
