#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "jinja2",
#   "markdown",
# ]
# ///
from __future__ import annotations

import argparse
import datetime
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


def get_version() -> str:
    cargo_path = Path("Cargo.toml")
    if not cargo_path.exists():
        return "0.1.0"
    for line in cargo_path.read_text(encoding="utf-8").splitlines():
        if line.startswith("version ="):
            return line.split("=")[1].strip().strip('"')
    return "0.1.0"


def get_skeleton_yaml() -> str:
    skeleton_path = Path("skeleton.yaml")
    if not skeleton_path.exists():
        return ""
    return skeleton_path.read_text(encoding="utf-8")


def get_readme_html() -> str:
    readme_path = Path("README.md")
    if not readme_path.exists():
        return ""
    import markdown
    readme_text = readme_path.read_text(encoding="utf-8")
    return markdown.markdown(readme_text)


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

    version = get_version()
    release_date = datetime.date.today().strftime("%Y-%m-%d")
    skeleton_yaml = get_skeleton_yaml()
    readme_html = get_readme_html()

    rendered = template.render(
        binary_downloads=BINARY_DOWNLOADS,
        version=version,
        release_date=release_date,
        skeleton_yaml=skeleton_yaml,
        readme_html=readme_html,
    )

    args.output_dir.mkdir(parents=True, exist_ok=True)
    (args.output_dir / "index.html").write_text(rendered, encoding="utf-8")


if __name__ == "__main__":
    main()

