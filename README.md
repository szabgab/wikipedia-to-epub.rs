A command line tool written in Rust that given a configuration file such as `examples/korea.json` will go to
the API of Wikipedia, download the source of the specific pages, and create an `.epub` file from them.

* The API URL is this: `https://en.wikipedia.org/w/api.php?action=parse&prop=wikitext&redirects=true&format=json&page=`
* The book does not include any embedded or included files such as style sheets or images.

There are several Wikipedia page dumps in the `pages/` folder to allow tests to run without accessing the API.

The CI workflow also generates a small GitHub Pages site that:

* links to the newest generated EPUB artifact bundle
* shows the JSON configs from `books/*.json` as examples


## Usage

```bash
cargo run -- examples/korea.json
```

Use local page dumps instead of downloading from Wikipedia:

```bash
cargo run -- examples/korea.json --local pages
```

The configuration file contains:

```json
{
  "metadata": {
    "title": "Korea",
    "author": "Wikipedia contributors",
    "license": "Creative Commons Non-Commercial Share Alike 3.0",
    "language": "en",
    "date": "2026-05-19"
  },
  "output-file": "korea.epub",
  "articles": ["Korea", "Seoul"]
}
```
