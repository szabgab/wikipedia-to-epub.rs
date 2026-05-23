A command line tool written in Rust that given a configuration file such as `examples/korea.yaml` will go to
the API of Wikipedia, download the source of the specific pages, and create an `.epub` file from them.

* The API URL is based on the configuration language, for example: `https://en.wikipedia.org/w/api.php?action=parse&prop=wikitext&redirects=true&format=json&page=`
* Live API fetches are throttled with a 1-second delay between requests to reduce `429 Too Many Requests` responses.
* The book does not include any embedded or included files such as style sheets or images.

There are several Wikipedia page dumps in the `pages/` folder to allow tests to run without accessing the API.

The CI workflow also generates a small GitHub Pages site that links to the newest compiled binaries.


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
* `{{Nihongo4|''[[Edo (Tokyo)|Edo]]''|[[wikt:江戸|江戸]]}}` becomes an italicized `Edo` article link followed by the Japanese text in a `lang="ja"` span
* `{{lang|ko|서울}}` becomes `<span lang="ko">서울</span>`
* `{{Percentage|1|4}}` becomes `25%`
* `{{UN_Population|Dem. People's Republic of Korea}}` becomes `26,100,000`; `{{UN_Population|ref}}` is omitted
* `{{convert|1100|km|abbr=on}}` becomes `1100 km`; range forms such as `{{convert|10|to|47|km2}}` become `10 to 47 km²`
* `{{ill|Ch'ilchŏngsan|ko}}` becomes a link to `Ch'ilchŏngsan` followed by `[ko]`
* `{{Reign|1400|1418}}` becomes `r. 1400–1418`
* `{{Main|Names of Korea}}` becomes `Main article:` followed by a link to `Names of Korea`
* `{{See also|Korean tea ceremony|Korean royal court cuisine}}` becomes `See also:` followed by links to those articles
* `== History ==` becomes `<h2>History</h2>`; deeper heading levels use deeper XHTML headings
* Lines starting with `*` become unordered list items
* Lines starting with `#` become ordered list items
* References, unhandled templates, tables, categories, and file/image links are omitted

## Amazon

You can upload your book to your Kindle via this link: [Send to Kindle](https://www.amazon.com/sendtokindle)
