A command line tool written in Rust that given a configuration file such as `examples/korea.yaml` will go to
the API of Wikipedia, download the source of the specific pages, and create an `.epub` file from them.

* The API URL is based on the configuration language, for example: `https://en.wikipedia.org/w/api.php?action=parse&prop=wikitext&redirects=true&format=json&page=`
* Live API fetches are throttled with a 1-second delay between requests to reduce `429 Too Many Requests` responses.
* The book does not include any embedded or included files such as style sheets or images.

There are several Wikipedia page dumps in the `pages/` folder to allow tests to run without accessing the API.

The CI workflow also generates a small GitHub Pages site that:

* links to the newest generated EPUB artifact bundle
* shows the YAML configs from `books/*.yaml` as examples


## Usage

```bash
cargo run -- examples/korea.yaml
```

Use local page dumps instead of downloading from Wikipedia:

```bash
cargo run -- examples/korea.yaml --local pages
```

The configuration file contains:

```yaml
metadata:
  title: Korea
  author: Wikipedia contributors
  license: Creative Commons Attribution-ShareAlike 4.0 License
  language: en
  edition: First edition
  date: 2026-05-19
output-file: korea.epub
articles:
  - Korea
  - Seoul
```

## Conversion rules

The converter renders a simplified subset of Wikipedia wikitext as XHTML:

* `''seoul''` becomes `<em>seoul</em>`
* `'''seoul'''` becomes `<strong>seoul</strong>`
* `[[Seoul]]` becomes a link to the internal chapter if `Seoul` is listed in `articles`, otherwise it links to the Wikipedia article
* `[[Seoul|capital city]]` becomes a link with `capital city` as the visible text
* `{{Korean|hangul=서울|labels=no}}` becomes `<span title="Korean-language text"><span lang="ko-Hang">서울</span></span>`
* `== History ==` becomes `<h2>History</h2>`; deeper heading levels use deeper XHTML headings
* Lines starting with `*` become unordered list items
* Lines starting with `#` become ordered list items
* References, templates, tables, categories, and file/image links are omitted

## Amazon

You can upload your book to your Kindle via this link: [Send to Kindle](https://www.amazon.com/sendtokindle)
