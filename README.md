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
* `{{Korean/auto|hangul=^해동_^요순|hanja=海東堯舜|labels=no}}` is rendered like `Korean`, with auto-template markers removed
* `{{Nihongo4|''[[Edo (Tokyo)|Edo]]''|[[wikt:江戸|江戸]]}}` becomes an italicized `Edo` article link followed by the Japanese text in a `lang="ja"` span
* `{{lang|ko|서울}}` becomes `<span lang="ko">서울</span>`
* `{{in lang|ko}}` becomes `(in Korean)`
* `{{linktext|漢|字}}` becomes `漢字`
* `{{langx|ko|溝樓|lit=Walled City|label=none}}` becomes `<span lang="ko">溝樓</span>, lit. Walled City`
* `{{Lang-zh|t=朝鮮|p=Cháoxiǎn|labels=no}}` becomes `<span lang="zh">朝鮮</span> (Cháoxiǎn)`
* `{{Transliteration|zh|pinyin|Zhuāngxiàn}}` becomes `<span lang="zh-Latn">Zhuāngxiàn</span>`
* `{{Ko-translit|rr|^한국}}` becomes `Hanguk`
* `{{lit|Vernacular Script Commission}}` becomes `lit. Vernacular Script Commission`
* `{{IPA|ko|haːnɡuk|}}` becomes `<span title="International Phonetic Alphabet">[haːnɡuk]</span>`
* `{{Abbr|c.|circa}}` becomes `<abbr title="circa">c.</abbr>`
* `{{frac|2|3}}` becomes `2/3`; mixed-number forms such as `{{frac|1|1|2}}` become `1 1/2`
* `{{Coord|37|33|36|N|126|59|24|E|display=inline}}` becomes `37°33′36″N 126°59′24″E`; decimal forms such as `{{Coord|43.65107|-79.347015|display=inline}}` become `43.65107, -79.347015`
* `{{rp|12}}` becomes an inline reference page marker: `p. 12`; multiple values such as `{{rp|12|15}}` become `pp. 12, 15`
* `{{Cite book|last=Oberdorfer|first=Don|title=The Two Koreas|year=2001|publisher=Basic Books}}` becomes `Don Oberdorfer. <em>The Two Koreas</em>. Basic Books, 2001`
* `{{Cite journal|last=Kim|first=Chin W.|title=The Legacy of King Sejong the Great|journal=Studies in the Linguistic Sciences|year=2000}}` becomes `Chin W. Kim. "The Legacy of King Sejong the Great". <em>Studies in the Linguistic Sciences</em>. 2000`
* `{{Cite report|last=Ledyard|first=Gari Keith|title=The Cultural Work of Sejong the Great|publication-date=November 2002|pages=7–18}}` becomes `Gari Keith Ledyard. <em>The Cultural Work of Sejong the Great</em>. November 2002. p. 7–18`
* `{{Citation|last=Cumings|first=Bruce|title=Korea's Place in the Sun|publisher=Norton|year=1997}}` becomes `Bruce Cumings. <em>Korea's Place in the Sun</em>. Norton, 1997`
* `{{harvc|last=Peterson|first=Mark|year=1992|in=Kim-Renaud|c=The Sejong Sillok}}` becomes `Mark Peterson. "The Sejong Sillok". In Kim-Renaud 1992`
* `{{As of|2023}}` becomes `As of 2023`; `{{As of|2009|lc=y}}` becomes `as of 2009`
* `{{Blockquote|text=Quoted text|source=Source}}` becomes `<blockquote><p>Quoted text</p><p class="blockquote-source">Source</p></blockquote>`
* `{{Percentage|1|4}}` becomes `25%`
* `{{UN_Population|Dem. People's Republic of Korea}}` becomes `26,100,000`; `{{UN_Population|ref}}` is omitted
* `{{convert|1100|km|abbr=on}}` becomes `1100 km`; range forms such as `{{convert|10|to|47|km2}}` become `10 to 47 km²`
* `{{ill|Ch'ilchŏngsan|ko}}` becomes a link to `Ch'ilchŏngsan` followed by `[ko]`
* `{{Reign|1400|1418}}` becomes `r. 1400–1418`
* `{{Open access}}` and `{{Free access}}` become an open-lock marker: `<span title="open access">&#128275;</span>`
* `{{Main|Names of Korea}}` becomes `Main article:` followed by a link to `Names of Korea`
* `{{See also|Korean tea ceremony|Korean royal court cuisine}}` becomes `See also:` followed by links to those articles
* `{{Further|Joseon dynasty|Downtown Seoul}}` becomes `Further information:` followed by links to those articles
* `{{For timeline|Timeline of Korean history}}` becomes `For a timeline, see:` followed by a link to `Timeline of Korean history`
* `{{Excerpt|Korean literature|templates=no}}` becomes `Excerpt from:` followed by a link to `Korean literature`
* `{{Wiktionary|Korea}}` becomes `Wiktionary:` followed by a link to the Wiktionary entry
* `{{Wikivoyage|Korea}}` becomes `Wikivoyage:` followed by a link to the Wikivoyage entry
* `{{Official website|https://example.com|name=Example}}` becomes an external link to `https://example.com` with `Example` as the visible text
* `{{URL|1=https://english.seoul.go.kr/|2=Official website}}` becomes an external link to `https://english.seoul.go.kr/` with `Official website` as the visible text
* `{{Largest cities|country=Korea|city_1=Seoul|div_1=Seoul|pop_1=9,904,312}}` becomes `Largest cities in Korea:` followed by a list of linked cities with division and population details
* `{{Historical populations|5=1949|6=1437670|7=1960|8=2445402}}` becomes `Historical populations:` followed by a list of year/population entries such as `1949: 1,437,670`
* `{{sclass|Valiant|harbor tug}}` becomes links to the ship-class article and ship type: `[[Valiant-class harbor tug|''Valiant''-class]] [[harbor tug]]`
* Observed Wikipedia navigation templates such as `{{History of Korea}}`, `{{Korea topics}}`, `{{East Asian topics}}`, `{{Portal bar}}`, `{{Portal}}`, `{{Commons category}}`, `{{Commons and category}}`, `{{Wikisource-inline}}`, `{{Seoul}}`, `{{Navboxes}}`, and `{{Authority control}}` are omitted
* Wikipedia succession-box templates such as `{{Succession box}}` or those whose names start with `s-`, such as `{{s-start}}`, `{{s-bef}}`, `{{s-ttl}}`, and `{{s-end}}`, are omitted
* Footnote wrappers such as `{{efn|...}}` and `{{refn|...}}` are omitted
* Maintenance and metadata templates such as `{{unreferenced section}}`, `{{Excessive citations inline}}`, `{{Unreliable source?}}`, `{{Better source needed}}`, `{{columns-list}}`, `{{location map+}}`, `{{Wide image}}`, `{{Pie chart}}`, `{{ahnentafel}}`, and `{{DEFAULTSORT:...}}` are omitted
* Reference-list wrappers such as `{{Reflist}}`, `{{notelist}}`, `{{Refbegin}}`, and `{{Refend}}` are omitted while surrounding list contents are preserved
* Decorative flag image templates such as `{{flagicon|US}}` are omitted
* `== History ==` becomes `<h2>History</h2>`; deeper heading levels use deeper XHTML headings
* Lines starting with `*` become unordered list items
* Lines starting with `#` become ordered list items
* References, unhandled templates, tables, categories, and file/image links are omitted

## Amazon

You can upload your book to your Kindle via this link: [Send to Kindle](https://www.amazon.com/sendtokindle)
