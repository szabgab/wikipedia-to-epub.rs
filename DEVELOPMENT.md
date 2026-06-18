A command line tool written in Rust that given a configuration file such as `examples/korea.yaml` will go to
the API of Wikipedia, download the source of the specific pages, and create an `.epub` file from them.

* The API URL is based on the configuration language, for example: `https://en.wikipedia.org/w/api.php?action=parse&prop=wikitext&redirects=true&format=json&page=`
* Live API fetches are throttled with a 1-second delay between requests to reduce `429 Too Many Requests` responses.
* The book includes a generated style sheet. Images are omitted by default and can be embedded with `images: true`.
* Live downloads are cached in the OS user cache directory under `wikipedia-to-epub/`.

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

Refresh cached live downloads:

```bash
cargo run -- examples/korea.yaml --refresh-cache
```

The configuration file contains:

```yaml
metadata:
  title: Korea
  author: Wikipedia contributors
  license: Creative Commons Attribution-ShareAlike 4.0 License
  language: en
  edition: First edition
output-file: korea.epub
images: false
links_to_pages: false
links_to_excluded_pages: emphasize
articles:
  - Korea
  - Seoul
```

Set `images: true` to embed resolvable `[[File:...]]` and `[[Image:...]]` links in the EPUB. Live runs download bounded thumbnails from Wikipedia/Commons and cache article JSON, image metadata JSON, and image files. Local `--local` runs use image fixtures from `pages/images/manifest.json` and ignore the live download cache; missing fixture images are omitted with a warning. Live runs log a final cache report with needed, cached, downloaded, and failed counts for JSON files and image files.

Set `chapters: numbered-title` to automatically prepend hierarchical numbers (e.g., `1 `, `1.1 `, `1.2 `) to chapter and subchapter titles both in the Table of Contents and on the actual chapter pages. By default, this is set to `title`, which leaves titles unnumbered.

Each successful run also writes a companion HTML report next to the EPUB output (for example, `korea.html` for `korea.epub`). The report shows the included hierarchy and the same-language Wikipedia pages that were linked from included pages but not added to the book.


## Conversion rules

The converter renders a simplified subset of Wikipedia wikitext as XHTML:

* `''seoul''` becomes `<em>seoul</em>`
* `'''seoul'''` becomes `<strong>seoul</strong>`
* `[[Seoul]]` becomes a link to the internal chapter if `Seoul` is listed in `articles`; otherwise `links_to_excluded_pages` controls whether it is shown as a plain Wikipedia link (`display`), an emphasized Wikipedia link with an arrow (`emphasize`), or plain text without a link (`disregard`)
* `[[Seoul|capital city]]` becomes a link with `capital city` as the visible text
* With `images: true`, `[[File:Ships in Busan.jpg|thumb|alt=Shipyard view|Busan port]]` becomes an embedded EPUB image block with `Shipyard view` as the image alt text and `Busan port` as the caption
* `{{Korean|hangul=서울|labels=no}}` and `{{ko|hangul=서울|labels=no}}` become `<span title="Korean-language text">Korean: <span lang="ko-Hang">서울</span></span>`
* `{{Korean/auto|hangul=부산|hanja=釜山|ko_ipa=pusʰa̠n}}` becomes Korean and Hanja labels followed by `pronounced [pusʰa̠n]`; auto-template markers such as `^` and `_` are removed
* `{{Nihongo|Tokyo Tower|東京タワー|Tōkyō tawā|extra text|extra2 text}}` becomes the English text followed by Japanese-language text, italicized Rōmaji, extra text inside the parentheses, and extra2 text outside the parentheses
* `{{Nihongo4|''[[Edo (Tokyo)|Edo]]''|[[wikt:江戸|江戸]]}}` becomes an italicized `Edo` article link followed by the Japanese text in a `lang="ja"` span
* `{{lang|ko|서울}}` becomes `<span lang="ko">서울</span>`
* `{{in lang|ko}}` becomes `(in Korean)`
* `{{linktext|漢|字}}` becomes `漢字`
* `{{langx|ko|溝樓|lit=Walled City|label=none}}` becomes `<span lang="ko">溝樓</span>, lit. Walled City`
* `{{Lang-zh|t=朝鮮|p=Cháoxiǎn|labels=no}}` becomes `<span lang="zh">朝鮮</span> (Cháoxiǎn)`
* `{{zh|t=西漢|p=Xīhàn}}` and `{{zhi|c=比}}` are rendered like Chinese-language text spans, including pinyin when present
* `{{Transliteration|zh|pinyin|Zhuāngxiàn}}` (and its alias `translit`) becomes `<span lang="zh-Latn">Zhuāngxiàn</span>`
* `{{tlit|ko|mr|Chosŏn'gŭl}}` becomes `<span lang="ko-Latn">Chosŏn'gŭl</span>`
* `{{Ko-translit|rr|^한국}}` becomes `Hanguk`
* `{{lit|Vernacular Script Commission}}` becomes `lit. Vernacular Script Commission`
* `{{ISBN|0-8248-0673-5}}` becomes `ISBN 0-8248-0673-5`
* `{{ASIN|B00086U61Y}}` becomes `ASIN B00086U61Y`
* `{{Script|Hani|神}}` becomes `神`
* `{{oclc|58053128}}` becomes `OCLC 58053128`
* `{{doi|10.1080/02757206.2013.726990}}` becomes `doi:10.1080/02757206.2013.726990`
* `{{NDLDC|782854/146}}` and its formatted variants (e.g. `format=url`, `format=pid`) render National Diet Library Digital Collection link identifiers
* `{{IPA|ko|haːnɡuk|}}` and `{{IPAc-en|lang|ˈ|tʃ|oʊ|s|ʌ|n}}` become International Phonetic Alphabet spans such as `<span title="International Phonetic Alphabet">[haːnɡuk]</span>` and `<span title="International Phonetic Alphabet">[ˈtʃoʊsʌn]</span>`
* `{{Respell|CHOH|sun}}` becomes `CHOH-sun`
* `{{Abbr|c.|circa}}` becomes `<abbr title="circa">c.</abbr>`
* `{{frac|2|3}}` and `{{fraction|365|385|1539}}` become `2/3` and `365 385/1539`; mixed-number forms such as `{{frac|1|1|2}}` become `1 1/2`
* `{{sfrac|1|6}}` renders vulgar fraction using superscript/subscript spans (e.g. `<sup>1</sup>⁄<sub>6</sub>`)
* `{{mvar|k}}` formats variables in italic text (`<em>k</em>`)
* `{{math|''y'' {{=}} 2}}` formats mathematical expressions inline (`<em>y</em> = 2`), supporting the nested `{{=}}` template which resolves to `=`
* `{{tmath|E = mc^2}}` renders the mathematical formula as plain text
* `{{closed-open|a|b}}` (or `{{closed-open|a, b}}`) renders the mathematical interval notation: `[a, b)`
* `{{sqrt|x}}` renders the square root symbol with the content: `√x`
* `{{Section link|Page|Section}}` (or `{{section link}}` / `{{slink}}`) renders the section link in internal format with a section sign: `[[Page#Section|Page § Section]]`
* `{{mset|1|2|3}}` renders mathematical set curly braces notation: `{1, 2, 3}`
* `{{hidden begin|title=Proof}}` renders the bold header title of the collapsible section: `'''Proof'''`
* `{{hidden end}}` is omitted from the output
* `{{JSTOR|1400906}}` renders the JSTOR citation identifier: `JSTOR 1400906`
* `{{wsPSM|Title|1|Date}}` renders a Wikisource Popular Science Monthly article citation
* `{{em|text}}` renders text in emphasis (italic) tags: `<em>text</em>`
* `{{StatsTopicTOC}}`, `{{Math topics TOC}}`, `{{Areas of mathematics}}`, and `{{Glossaries of science and engineering}}` are silent templates and are omitted from the output
* `{{MathWorld|title=...}}` renders a MathWorld article citation
* `{{AS ref|section|page}}` renders a reference citation to Abramowitz and Stegun
* `{{OEIS2C|A...}}` renders a linked OEIS sequence identifier
* `{{thinsp|a|b}}` renders arguments joined by a thin space
* `{{dfn|text}}` wraps text in definition tags: `<dfn>text</dfn>`
* `{{subsup|base|sub|sup}}` renders subscript and superscript elements aligned vertically
* `{{abs|x}}` renders the absolute value of the argument: `|x|`
* `{{mono|text}}` wraps text in code (monospace) tags: `<code>text</code>`
* `{{pi}}` renders the Greek letter pi: `π`
* `{{Springer|title=...}}` renders a Springer Encyclopedia of Mathematics article citation
* `{{Probability fundamentals}}` and `{{ProbDistributions}}` are silent templates and are omitted from the output
* `{{divcol}}` and `{{divcol end}}` are silent templates and are omitted from the output
* `{{cite paper|title=...}}` is handled as an alias of `{{cite journal}}`
* `{{cite court|litigants=...}}` formats legal case citations in Bluebook style
* `{{Cite Dictionary.com|word}}` formats Dictionary.com Unabridged definition citations
* `{{Cite speech|title=...}}` formats speech/presentation citations
* `{{cite SSRN|ssrn=...}}` formats Social Science Research Network paper citations
* `{{cite tech report|title=...}}` is handled as an alias of `{{cite report}}`
* `{{Cite CiteSeerX|citeseerx=...}}` formats CiteSeerX paper citations
* `{{nobr|text}}` renders the text as a passthrough (without breaking line breaks)
* `{{Collapse top|title}}` formats a collapsible title block header: `'''title'''`
* `{{Collapse bottom}}` is omitted from the output
* `{{var|x}}` formats variables in `<var>` tags: `<var>x</var>`
* `{{gaps|1|2|3|e=-4|u=kg|lhs=y}}` formats numbers with digit grouping, optional scientific notation, and units: `y = 1 2 3×10<sup>-4</sup> kg`
* `{{which?}}`, `{{Redirect-distinguish}}`, and `{{example needed}}` are silent templates and are omitted from the output
* `{{formatnum:5324}}` formats the number using thousands separators: `5,324`
* `{{Birth date and age|1931|3|7}}` and `{{birth date and age|1931|3|7|df=yes}}` render the birth date and current age: `March 7, 1931 (age X)` and `7 March 1931 (age X)`
* `{{dts|1947-5-20}}` (or `{{dts|1947|May|20}}`) formats the date for table sorting, displaying it in a human-readable form: `May 20, 1947`; also supports the `format=dmy` parameter (e.g. `20 May 1947`) and the `bc` flag
* `{{unbulleted list|item1|item2}}` (or its aliases `{{ubl}}`, `{{ubli}}`, `{{ublist}}`, and `{{unbulleted indent list}}`) renders standard XHTML list items wrapped in unordered list tags: `<ul><li>item1</li><li>item2</li></ul>`
* `{{hlist|item1|item2}}` (and `{{flatlist}}`) renders standard XHTML list items joined by commas: `item1, item2`
* `{{native name list|tag1=ja|name1=Name}}` renders native name list as a comma-separated list of names and their language tags: `Name (Japanese)`
* `{{Infobox mountain|name=...}}` renders mountain infoboxes as a two-column wikitable containing properties (e.g. Name, Native name, Country, Highest point, Coordinates, etc.)
* `{{Infobox country|conventional_long_name=...}}` renders country infoboxes as a two-column wikitable containing core political, symbolic, language, and historical fields used by the cached country pages
* `{{Infobox military conflict|conflict=...}}` renders military-conflict infoboxes as a two-column wikitable containing conflict metadata, combatants, commanders, strength, casualties, and notes used by the cached conflict page
* `{{Infobox planet|name=...}}` renders planetary infoboxes as a two-column wikitable containing image/caption, orbital data, physical characteristics, temperature rows, and atmospheric properties
* `{{Infobox settlement|name=...}}` renders settlement infoboxes as a two-column wikitable containing properties (e.g. Name, Official name, Native name, Country, Governing body, Area, Population, Website, etc.)
* `{{infobox|title=...|label1=...|data1=...}}` renders generic infoboxes as a two-column wikitable mapping titles, headers, labels, and data fields
* `{{mdash}}` becomes `—`
* `{{snd}}` (or `{{dash}}` / `{{snds}}`) becomes a spaced en dash: ` – `
* `{{!}}` becomes `|`
* `{{ndash}}` and `{{endash}}` become `–`
* `{{citation needed span|text}}` acts as a passthrough wrapper, omitting the superscript citation warning
* `{{Quote box|quote=...}}` renders as a blockquote
* `{{center|text}}` acts as a passthrough wrapper preserving inner formatting
* `{{singular}}` becomes `<abbr title="singular form">sg.</abbr>`
* `{{nihongo2|日本}}` becomes `<span lang="ja">日本</span>`
* `{{gloss|sun origin}}` becomes `'sun origin'`; definition mode such as `{{gloss|mode=def|ensemble drumming}}` becomes `(ensemble drumming)`
* `{{xref|(see [[Nanban trade]])}}` becomes visible cross-reference text with links
* `{{Shy|Pre|fec|tures}}` becomes `Pre\u00adfec\u00adtures` using a soft hyphen
* `{{color box|#EF7979}}` becomes `<span style="color: #EF7979;">■</span>`
* `{{pb}}` becomes a paragraph/line break: `<br /><br />`
* `{{parabr}}` outputs paragraph break tokens: `__WIKIPEDIA_TO_EPUB_BR____WIKIPEDIA_TO_EPUB_BR__`
* `{{Break}}` (or `{{br}}`, `{{brk}}`, `{{crlf}}`) becomes a line break: `<br />`; supports an optional positional parameter `n` to repeat the break `n` times
* `{{okina}}` becomes `ʻ`
* `{{'s}}` becomes `'s`
* `{{harvp|Martin|1966}}` becomes `(Martin 1966)`; multi-author and page/location variants are formatted identically to standard Harvard citations, e.g. `(Sohn 2001, loc=Section 1.5.3)` becomes `(Sohn 2001, Section 1.5.3)`
* `{{harvtxt|Martin|1966}}` renders Harvard citations with authors outside the parentheses: `Martin (1966)`
* `{{IPAslink|m}}` becomes an IPA-styled phonetic alphabet span: `<span title="International Phonetic Alphabet">[m]</span>`
* `{{angbr|a}}` wraps the text in phonetic angle brackets: `⟨a⟩`
* `{{angbr IPA|◌̧}}` wraps the IPA text in angle brackets and styles the inner text as IPA: `⟨<span lang="und-fonipa">◌̧</span>⟩`
* `{{unichar|0348|cwith=◌}}` displays the combined glyph with its code point and name: `◌͈ (U+0348)`
* `{{xlit|ko|'''r'''odong}}` becomes a transliterated language span: `<span lang="ko-Latn"><strong>r</strong>odong</span>`
* `{{note|ㅏ|[A]}}` renders a bold footnote label: `<strong>[A]</strong>`
* `{{fs interlinear|lang=ko|...}}` renders aligned linguistic glosses as a cleanly styled `blockquote` block
* `{{Tooltip|RR|Revised Romanization}}` becomes `<abbr title="Revised Romanization">RR</abbr>`
* `{{Nihongo krt||\u{5927}\u{962a}|\u{14c}saka}}` becomes `<span lang="ja">大阪</span> (<em>Ōsaka</em>)`
* `{{nihongo3|shrine temple|神宮寺|[[jingū-ji]]}}` displays Rōmaji first, followed by script and translation: `<em>[[jingū-ji]]</em> (<span lang="ja">神宮寺</span>, "shrine temple")`
* `{{Nihongo foot|Tokyo|東京|Tōkyō}}` formats Japanese language information inline next to the English text: `Tokyo (<span lang="ja">東京</span>, <em>Tōkyō</em>)`
* `{{Easy CSS image crop|Image=Osaka Urban Railway network.svg|...}}` converts seamlessly to standard image links inside the EPUB book
* `{{Multiple images|image1=...|caption1=...}}` (and its alias `{{Multiple image}}`) converts multiple grouped images into individual standard File links
* `{{South Korea Provincial level Labelled Map}}` and other labelled map templates listed in `src/maps.csv` convert to standard image links inside the EPUB book
* `{{ISSN|0268-4160}}` becomes `ISSN 0268-4160`
* `{{Cite NSRW|wstitle=Osaka}}` renders as an article citation linking to Wikisource
* `{{circa}}` 10 million` becomes `c. 10 million`
* `{{est.|1990}}` wraps established abbreviation: `est. 1990`
* `{{c.|115 BC}}` and `{{cx|150 AD}}` become `c. 115 BC` and `c. 150 AD`
* `{{e28|kor|Korean}}` renders Ethnologue 28th edition citation format: `Eberhard, David M.; Simons, Gary F.; Fennig, Charles D., eds. (2025). "[[official-url:https://www.ethnologue.com/language/kor|Korean]]". ''Ethnologue: Languages of the World'' (28th ed.). Dallas, Texas: SIL International`
* `{{floruit|6th century BC}}` becomes `fl. 6th century BC`
* `{{legend|#EF767E|North Korean forces}}` and `{{legend0|#EF767E|North Korean forces}}` become `North Korean forces`
* `{{numero|3}}` becomes `No. 3`
* `{{sic|was}}` becomes `was [sic]`
* `{{AWOL}}` becomes `([Absent without leave|AWOL])` with the label wrapped in an abbreviation tag: `<abbr title="Desertion">AWOL</abbr>`
* `{{Assassinated}}` and `{{Assassinated|alt=yes}}` render assassination templates: `[[Assassination|'''X''']]` and `[[Assassination|(Assassinated)]]`
* `{{DOW}}` and `{{Died of wounds}}` become `([[Killed in action|DOW]])` with the label wrapped in an abbreviation tag: `<abbr title="Died of wounds">DOW</abbr>`
* `{{Executed}}` and `{{Executed|link=Target}}` render execution markers: `[[File:Skull and Crossbones.svg|14px|Executed|link=Target]]`
* `{{KIA}}` and `{{KIA|alt=yes}}` render killed-in-action templates: `[[Killed in action|'''†''']]` and `[[Killed in action|(KIA)]]`
* `{{KIA2}}` becomes `[[Killed in action|(KIA)]]`
* `{{MIA}}` becomes `([[Missing in action|MIA]])` with the label wrapped in an abbreviation tag: `<abbr title="Missing in action">MIA</abbr>`
* `{{Natural Causes}}` and `{{Natural Causes|alt=yes}}` render natural causes templates with tooltip abbreviation tags
* `{{PKIA}}` becomes `([[Killed in action|PKIA]])` with the label wrapped in an abbreviation tag: `<abbr title="Presumed killed in action">PKIA</abbr>`
* `{{POW}}` becomes `([[Prisoner of war|POW]])` with the label wrapped in an abbreviation tag: `<abbr title="Prisoner of war">POW</abbr>`
* `{{Suicide}}` and `{{Suicide|alt=yes}}` render suicide templates: `[[Suicide|'''‡‡''']]` and `[[Suicide|(Suicide)]]`
* `{{Surrendered}}` and `{{Surrendered|link=Target}}` render surrender markers: `[[File:White flag icon.svg|14px|Surrendered|link=Target]]`
* `{{Turncoat}}` and `{{Turncoat|link=Target}}` render turncoat markers: `[[File:Black flag icon.svg|14px|Turncoat|link=Target]]`
* `{{WIA}}` becomes `([[Wounded in action|WIA]])` with the label wrapped in an abbreviation tag: `<abbr title="Wounded in action">WIA</abbr>`
* `{{Nowrap|June 10}}` becomes `June 10`
* `{{su|p=2|b=x}}` becomes `<sup>2</sup><sub>x</sub>`
* `{{Smaller|note}}` and `{{small|note}}` become `<small>note</small>`
* `{{Coord|37|33|36|N|126|59|24|E|display=inline}}` becomes `37°33′36″N 126°59′24″E`; decimal forms such as `{{Coord|43.65107|-79.347015|display=inline}}` become `43.65107, -79.347015`
* `{{rp|12}}` (or its alias `{{Reference page|page=12}}`) becomes an inline reference page marker: `p. 12`; multiple values such as `{{rp|12|15}}` become `pp. 12, 15`
* `{{Cite book|last=Oberdorfer|first=Don|title=The Two Koreas|year=2001|publisher=Basic Books}}` becomes `Don Oberdorfer. <em>The Two Koreas</em>. Basic Books, 2001`
* `{{cite dictionary |last=Smith |first=John |title=Apple |dictionary=English Dictionary |edition=2nd |publisher=Oxford |date=2020 |page=15}}` becomes `John Smith. "Apple". <em>English Dictionary</em>. 2nd ed. Oxford. 2020. p. 15`
* `{{cite press release |last=Doe |first=Jane |title=New Release |publisher=Company |date=2021}}` becomes `Jane Doe. "New Release" (Press release). Company. 2021`
* `{{cite apod |title=Nebula |date=2020-04-15 |access-date=2020-05-01}}` becomes `R. Nemiroff & J. Bonnell, eds. (2020-04-15). "Nebula". <em>Astronomy Picture of the Day</em>. NASA. Retrieved 2020-05-01`
* `{{cite OED |entry=Word |id=12345 |date=2015 |access-date=2016}}` becomes `"Word". <em>Oxford English Dictionary</em> (Online ed.). Oxford University Press. 2015. Retrieved 2016`
* `{{OED |entry=Word |id=12345 |date=2015 |access-date=2016}}` becomes `"Word". <em>Oxford English Dictionary</em> (Online ed.). Oxford University Press. 2015. Retrieved 2016`
* `{{cite AV media |last=Director |first=A. |title=Movie |format=Film |publisher=Studio |via=YouTube |date=2010}}` becomes `A. Director. "Movie" (Film). Studio. YouTube. 2010`
* `{{cite American Heritage Dictionary |1=Lexicon |date=2018 |access-date=2019}}` becomes `"Lexicon". <em>The American Heritage Dictionary of the English Language</em>. 2018. Retrieved 2019`
* `{{cite wikisource |last=Author |first=B. |title=Book Title |wslink=Book Title |publisher=Paris |year=1800}}` becomes `B. Author. <em>[[src:Book Title|Book Title]]</em>. Paris, 1800. [[Wikisource]]`
* `{{cite CIA World Factbook |country=North Korea |section=Geography |year=2021 |access-date=2022-03-01}}` becomes `"North Korea § Geography". <em>The World Factbook</em> (2021 ed.). Central Intelligence Agency. Retrieved 2022-03-01`
* `{{cite letter |last=Sender |first=A. |recipient=Recipient |subject=Important Matters |publisher=Archive |date=1900}}` becomes `A. Sender. "Important Matters" (Letter to Recipient). Archive. 1900`
* `{{cite arXiv |last=Physicist |first=A. |title=Quantum Theory |date=2019 |eprint=1901.00001 |class=hep-th}}` becomes `A. Physicist. "Quantum Theory". 2019. arXiv:1901.00001 [hep-th]`
* `{{cite q |Q123456}}` becomes `Wikidata item [https://www.wikidata.org/wiki/Q123456 Q123456]`
* `{{cite web|last=Demick|first=Barbara|title=North Korea's giant leap backwards|url=http://example.com|website=The Guardian|date=16 July 2010}}` becomes `Barbara Demick. "North Korea's giant leap backwards". <em>The Guardian</em>. 16 July 2010`, with the title linked when `url=` is present
* `{{Cite journal|last=Kim|first=Chin W.|title=The Legacy of King Sejong the Great|journal=Studies in the Linguistic Sciences|year=2000}}` becomes `Chin W. Kim. "The Legacy of King Sejong the Great". <em>Studies in the Linguistic Sciences</em>. 2000`
* `{{cite magazine|title=The Japan Alps|magazine=National Geographic|date=1910}}` becomes `"The Japan Alps". <em>National Geographic</em>. 1910`
* `{{cite news|title=Alpine Explorer|newspaper=The Times|date=1920}}` becomes `"Alpine Explorer". <em>The Times</em>. 1920`
* `{{Cite report|last=Ledyard|first=Gari Keith|title=The Cultural Work of Sejong the Great|publication-date=November 2002|pages=7–18}}` becomes `Gari Keith Ledyard. <em>The Cultural Work of Sejong the Great</em>. November 2002. p. 7–18`
* `{{cite ECCP|last=Kennedy|first=George A.|title=Amin|pages=8–9|date=1943}}` becomes `George A. Kennedy. "Amin". Eminent Chinese of the Ch'ing Period. 1943. pp. 8–9`
* `{{cite gvp|name=Norikuradake|vn=283060|access-date=2021-06-24}}` becomes `"Norikuradake". <em>Global Volcanism Program</em>. Smithsonian Institution. Retrieved 2021-06-24`
* `{{e28|kor|Korean}}` renders Ethnologue 28th edition citation format: `Eberhard, David M.; Simons, Gary F.; Fennig, Charles D., eds. (2025). "[[official-url:https://www.ethnologue.com/language/kor|Korean]]". ''Ethnologue: Languages of the World'' (28th ed.). Dallas, Texas: SIL International`
* `{{cite conference|author=Smith|title=Ancient Borders|book-title=Proceedings of Archaeology|year=2010}}` becomes `Smith. <em>Ancient Borders</em>. 2010`
* `{{cite thesis|last=Kim|first=Jane|title=Origins|year=2010|publisher=Seoul University}}` becomes `Jane Kim. <em>Origins</em>. Seoul University, 2010`
* `{{worldhistory|section=378|quote=the state of Parhae}}` becomes `"the state of Parhae". <em>The Encyclopedia of World History</em> (6th ed.)`
* `{{Citation|last=Cumings|first=Bruce|title=Korea's Place in the Sun|publisher=Norton|year=1997}}` becomes `Bruce Cumings. <em>Korea's Place in the Sun</em>. Norton, 1997`
* `{{multiref|Ref 1|Ref 2}}` (and its alias `{{Multiref2}}`) joins positional ref values with a semicolon: `Ref 1; Ref 2`
* `{{hosking-jfood|page=123}}` renders Richard Hosking book citation format: `Hosking, Richard (1996). ''A Dictionary of Japanese Food: Ingredients & Culture''. Tuttle Publishing. p. 123. ISBN 978-0-8048-2042-4`
* `{{citation-attribution|text}}` renders public domain text attribution: `One or more of the preceding sentences incorporates text from a work now in the public domain: text`
* `{{olist|Item 1|Item 2}}` (and its alias `{{ordered list}}`) renders a numbered list: `# Item 1\n# Item 2`
* `{{webtrans|url|title|lang}}` renders an external link with translation language indicator: `[[official-url:url|title]] (in lang)`
* `{{OSM|type|id|name}}` renders an OpenStreetMap link pointing to a node, way, or relation
* `{{Wiktionary-inline|word}}` (and its alias `{{wti}}`) renders a definition reference link: `The dictionary definition of [[wikt:word|word]] at Wiktionary`
* `{{cite opentopomap|name|lat|long}}` renders a topographic map citation from opentopomap.org
* `{{EngvarB}}` is silently skipped
* `{{colorbull|color|shape|link}}` renders a colored bullet point character: `__WIKIPEDIA_TO_EPUB_COLOR_START__color__WIKIPEDIA_TO_EPUB_COLOR_MID__shape_char__WIKIPEDIA_TO_EPUB_COLOR_END__`
* `{{how-to}}` is silently skipped
* `{{Portal-inline|name}}` renders an inline portal link: `[[Portal:name|name portal]]`
* `{{end box}}` is silently skipped
* `{{mp|leading|subscript}}` (and its alias `{{minor planet}}`) formats minor planet designations with subscripts
* `{{harvc|last=Peterson|first=Mark|year=1992|in=Kim-Renaud|c=The Sejong Sillok}}` becomes `Mark Peterson. "The Sejong Sillok". In Kim-Renaud 1992`
* `{{As of|2023}}` becomes `As of 2023`; `{{As of|2009|lc=y}}` becomes `as of 2009`
* `{{died-in|202 BC}}` becomes `d. 202 BC`
* `{{age|1989|11|9|2019|11|9}}` calculates and displays age between two dates: `30`; if only birth date is provided, calculates age relative to the current date
* `{{Age in years, months, weeks and days|2020|1|15|2021|3|20}}` renders date duration difference in years, months, weeks, and days: `1 year, 2 months and 5 days`
* `{{ayd|April 26, 2001|September 26, 2006}}` calculates and displays duration in years and days: `5 years, 153 days`; also supports numeric parameters and single-date relative calculations
* `{{Blockquote|text=Quoted text|source=Source}}` (or `{{Quote|text=Quoted text|author=Source}}`) becomes `<blockquote><p>Quoted text</p><p class="blockquote-source">Source</p></blockquote>`
* `{{Poem quote|text=old pond\nfrog leaps in|source=Basho}}` (or its alias `{{poemquote}}`) renders as a blockquote, preserving line breaks and an optional source: `<blockquote><p>old pond</p><p>frog leaps in</p><p class="blockquote-source">Basho</p></blockquote>`
* `{{Verse translation|L'autunno giovane|The young autumn}}` renders the original verse (italicized by default) and its translation sequentially inside a blockquote: `<blockquote><em>L'autunno giovane</em><p>The young autumn</p></blockquote>`
* `{{Verse transliteration-translation|稲妻の|inazuma no|the flash}}` renders the original verse, transliterated verse (italicized by default), and its translation sequentially inside a blockquote: `<blockquote><p>稲妻の</p><em>inazuma no</em><p>the flash</p></blockquote>`
* `{{Percentage|1|4}}` becomes `25%`
* `{{UN_Population|Dem. People's Republic of Korea}}` becomes `26,100,000`; `{{UN_Population|ref}}` is omitted
* `{{convert|1100|km|abbr=on}}` and `{{cvt|314|km|0}}` become `1,100 km (684 mi)` and `314 km (195 mi)`; large numeric values are grouped with commas every three digits, and range forms such as `{{convert|10|to|47|km2}}` become `10 to 47 km² (3.86 to 18.1 mi²)`. When `convert` supplies multiple alternate units, they are rendered together inside the parentheses, e.g. `{{convert|737|K|C F|abbr=on}}` becomes `737 K (464 °C, 867 °F)`.
* `{{ill|Ch'ilchŏngsan|ko}}` (along with its aliases `{{illm}}`, `{{Interlanguage link}}`, and `{{Interlanguage link multi}}`) becomes a link to `Ch'ilchŏngsan` followed by `[ko]`
* `{{Reign|1400|1418}}` becomes `r. 1400–1418`
* `{{Open access}}` and `{{Free access}}` become an open-lock marker: `<span title="open access">&#128275;</span>`
* `{{Main|Names of Korea}}` (or `{{Main article|Names of Korea}}`) becomes `Main article:` followed by a link to `Names of Korea`
* `{{Main list|List of members of the Diet of Japan}}` becomes `For a more comprehensive list, see` followed by a link to `List of members of the Diet of Japan`
* `{{See also|Korean tea ceremony|Korean royal court cuisine}}` becomes `See also:` followed by links to those articles
* `{{Further|Joseon dynasty|Downtown Seoul}}` becomes `Further information:` followed by links to those articles
* `{{For|histories of the modern Korean countries|History of North Korea|History of South Korea}}` becomes `For histories of the modern Korean countries, see:` followed by links to those articles
* `{{For timeline|Timeline of Korean history}}` becomes `For a timeline, see:` followed by a link to `Timeline of Korean history`
* `{{crossreference|See {{slink|#Letter counts}}.}}` becomes visible cross-reference text with a section link
* `{{anl|Battle of Jushi}}` becomes a link to `Battle of Jushi`
* `{{Arrow|r}}` becomes `→`
* `{{Excerpt|Korean literature|templates=no}}` becomes `Excerpt from:` followed by a link to `Korean literature`
* `{{Wiktionary|Korea}}` becomes `Wiktionary:` followed by a link to the Wiktionary entry
* `{{Wikivoyage|Korea}}`, `{{Wikivoyage-inline|Korea}}`, and `{{Wikivoyage inline|Korea}}` become `Wikivoyage:` followed by a link to the Wikivoyage entry
* `{{Wikisource|Korea}}` becomes `Wikisource:` followed by a link to the Wikisource entry
* `{{Wikibooks|1=Book title|2=Chapter title|3=label}}` becomes `Wikibooks:` followed by a link to the Wikibooks chapter
* `{{Britannica|322222}}` becomes `Britannica:` followed by a link to the Britannica article id
* `{{Britannica URL|url=https://www.britannica.com/topic/test|title=Test Topic}}` renders Britannica citation format: `"[[official-url:https://www.britannica.com/topic/test|Test Topic]]" at ''Encyclopædia Britannica''`
* `{{Jaanus|w/washi|Washi}}` renders as an external link to the JAANUS database: `<a href="http://www.aisf.or.jp/~jaanus/deta/w/washi.htm">Washi</a> at JAANUS`
* `{{Official website|https://example.com|name=Example}}` (and its alias `{{official|...}}`) becomes an external link to `https://example.com` with `Example` as the visible text
* `{{URL|1=https://english.seoul.go.kr/|2=Official website}}` becomes an external link to `https://english.seoul.go.kr/` with `Official website` as the visible text
* `{{osmrelation-inline|2396450}}` and `{{OSM relation|382313}}` become an external link to the OpenStreetMap relation; `{{osmway|131922091}}` and `{{OSM way|131922091}}` similarly link to the OpenStreetMap way
* `{{Webarchive|url=https://web.archive.org/web/20140703095242/http://example.com/report.pdf|date=3 July 2014}}` becomes an external archive link labelled `Archived on 3 July 2014`
* `{{GBurl|id=abc123|p=12}}` and `{{GBurl|id=abc123|pg=PA12|q=search+term}}` become external links to the Google Books page, such as `https://books.google.com/books?id=abc123&pg=PA12&q=search+term`
* `{{Google books|abc123|''Example'', p. 57|page=57}}` becomes an external link to the matching Google Books page with the supplied label, such as `<a href="https://books.google.com/books?id=abc123&pg=PA57"><em>Example</em>, p. 57</a>`
* `{{usurped|1=https://web.archive.org/web/20130101000000/http://example.com}}` renders the archive URL as an external link, preserving the wikitext hyperlink format
* `{{Largest cities|country=Korea|city_1=Seoul|div_1=Seoul|pop_1=9,904,312}}` becomes `Largest cities in Korea:` followed by a list of linked cities with division and population details
* `{{Historical populations|5=1949|6=1437670|7=1960|8=2445402}}` becomes `Historical populations:` followed by a list of year/population entries such as `1949: 1,437,670`
* `{{climate chart|Busan|−0.1|8.2|34.5|...}}` becomes `Climate chart for Busan:` followed by monthly low/high temperature and precipitation entries
* `{{sclass|Valiant|harbor tug}}` becomes links to the ship-class article and ship type: `[[Valiant-class harbor tug|''Valiant''-class]] [[harbor tug]]`
* `{{ROKS|Sejong the Great||2}}` becomes a link to `ROKS Sejong the Great` with the visible label `ROKS Sejong the Great`
* `{{STN|Ginza}}` and `{{STN|Hamaōtsu|x}}` create railway station links such as `[[Ginza Station|Ginza]]` and `[[Hamaōtsu Station|Hamaōtsu]]`
* `{{Station|Shibuya|1|Tokyo}}` renders a railway station link with customizable capitalization, location suffix, and label parameters: `[[Shibuya Station (Tokyo)|Shibuya]]`
* `{{For-multi|topic1|link1|topic2|link2}}` alternates topics and links, rendering as `For topic1, see [[link1]]; for topic2, see [[link2]].`
* `{{Inflation|US|12|1950}}` and `{{Inflation/year|US}}` adjust values using US CPI table indices (1950 to 2023)
* `{{FXConvert|KOR|293.823|b|cursign=[[₩]]|year=2020|showdate=no}}` formats and converts historical currency values (e.g. `₩293.82 billion (US$248.95 million)`)
* `{{JPY|1234.56}}` displays formatted currency with the Yen symbol: `¥1,234.56`; if no amount is provided, displays just `¥`
* `{{stack|content}}` acts as a generic passthrough wrapper preserving inner wikitext
* `{{longitem|content}}` acts as a generic passthrough wrapper preserving inner wikitext
* `{{USS|Missouri|BB-63|6}}`, `{{HMS|Jamaica|44|6}}`, and `{{ship|Japanese cruiser|Kiso}}` render as formatted, italicized ship names linked to their respective articles
* `{{Nb5}}` renders as five non-breaking spaces (e.g. `     `)
* `{{color|red|text}}` (and its British spelling alias `colour`) renders text with the given foreground color: `<span style="color: red;">text</span>`
* `{{Ja-rail-color|JY}}` returns the standardized hex color code for the Japanese rail line (e.g. `#80c241`)
* `{{Ja-platform|pfn=1|name=Yamanote Line|dir=for Tokyo}}` (or `{{jpf}}`, `{{Ja-platform-m}}`, `{{jpfm}}`) renders a Japanese rail platform layout as a table row inside wikitables
* `{{ja-rail-linem|m|linename=Tokyo Metro|linecol=blue}}` renders a Japanese railway line row inside wikitables, supporting customizable symbol types, line colors, names, and direction descriptions
* `{{rail-interchange|JR East|JT}}` (or `{{ric}}`, `{{rint}}`) displays railway system/line abbreviations in brackets, e.g. `[JT]`
* `{{Line link|JR East|JY}}` (or `{{lnl}}`) creates formatted internal links for rail transit lines, e.g. `[[Yamanote Line]]` or `[[Chūō Line (Rapid)|Chūō Line]]`
* `{{nbndash}}` (or `{{nbnd}}`) renders a non-breaking en-dash: `–`
* `{{RouteBox|JY|Yamanote Line|#80c241|white}}` renders a route box badge with colored background and text wrapping a wikilink to the route: `<span style="background-color: #80c241; color: white; ...">[[Yamanote Line|JY]]</span>`
* `{{plainlist|1=* Item}}` renders as a standard plain list without bullet styling
* `{{indented plainlist|1=* Item}}` creates an unbulleted list (wrapped identically to `{{plainlist}}`)
* `{{bulleted list|Item 1|Item 2}}` (and its alias `{{blist}}`) renders bulleted list items
* `{{flagdeco|United Nations}}` renders as an empty string (silent placeholder for decorative flags)
* `{{pprime|9.7}}` displays a double prime symbol (″) after the given text (e.g. `9.7″`)
* `{{RA|18|11|2}}` formats astronomical Right Ascension coordinates (e.g. `18<sup>h</sup> 11<sup>m</sup> 2<sup>s</sup>`)
* `{{MW|Venusian}}` (and `{{Cite Merriam-Webster}}`) formats a dictionary citation referencing Merriam-Webster
* `{{Hyphen}}` explicitly renders a standard hyphen-minus `-` character
* `{{native phrase|ko|渤海}}` (and `{{native name}}`) displays a term in its native language followed by its language name in parentheses (e.g. *渤海* (Korean))

* `{{harv|Davis|1999}}` and `{{harvnb|Davis|1999}}` render as formatted inline Harvard citations: `(Davis 1999)` and `Davis 1999`
* `{{Collapsible list|title=Title|Item A|Item B}}` renders a title followed by bulleted items on newlines
* `{{Internet Archive short film|id=id|name=Name}}` renders as an external link to the Internet Archive short film details page
* `{{jct|country=JPN|Route|41}}` renders as a formatted road link (e.g. `National Route 41` linked to the Wikipedia article)
* `{{Proto|germanic|erþō}}` renders linguistic proto-language reconstructions: `Proto-Germanic *erþō`
* `{{wktl|grc|γῆ|gē}}` and `{{langr|la|Terra}}` format language-tagged text inside inline spans: `<span lang="grc">γῆ</span>` and `<span lang="la">Terra</span>`
* `{{val|4.5682|0.0002|0.0004}}` and `{{Value|5.97|u=[[Ronnagram|Rg]]}}` render values with uncertainties, ranges, exponents, or units: `4.5682 (+0.0004/-0.0002)` and `5.97 <a href="https://en.wikipedia.org/wiki/Ronnagram">Rg</a>`
* `{{chem2|O2}}` renders chemical formulas using subscripts: `O<sub>2</sub>`
* `{{e|-5}}` renders powers of ten: `× 10<sup>-5</sup>`
* `{{sup|2}}` and `{{sub|x}}` render superscript and subscript spans: `<sup>2</sup>` and `<sub>x</sub>`
* `{{mpl|2010 TK|7}}` renders minor planet names: `2010 TK7`
* `{{columns list|colwidth=22em|...}}` processes and displays positional list items inside standard unordered lists
* `{{annotated link|Celestial sphere}}` translates annotated links into standard wikilinks
* `{{Dp|Ceres}}` and `{{dp|makemake}}` render relative wikilinks to dwarf planet articles: `[[Ceres (dwarf planet)|Ceres]]` and `[[Makemake|makemake]]`
* `{{Visible anchor|Mercury|text=[[Mercury]]}}` and `{{visible anchor|Earth}}` render their visible text: `[[Mercury]]` and `Earth`
* `{{L4}}` and `{{L5}}` render Lagrange points using subscript formatting: `L<sub>4</sub>` and `L<sub>5</sub>`
* `{{Cite EB1911|wstitle=Solar System}}` renders Encyclopaedia Britannica 1911 Wikisource citations: `"Solar System" in ''[[src:1911 Encyclopædia Britannica/Solar System|Encyclopædia Britannica]]'' (11th ed., 1911)`
* `{{spaces|3}}` and `{{spaces}}` render non-breaking spaces (which are collapsed to a standard space)
* `{{mpl-|322756|2001 CK|32}}` renders parenthesized numbered designation for minor planets linked to their Wikipedia article: `[[(322756) 2001 CK32]]`
* `{{chem|H|2|O}}` and `{{chem|CO|3|2-}}` render chemical formulas using subscripts for numbers and superscripts for charges: `H<sub>2</sub>O` and `CO<sub>3</sub><sup>2-</sup>`
* `{{airport codes|MMJ|RJAF}}` formats airport codes: `(IATA: MMJ, ICAO: RJAF)`; supports hiding the outer parentheses using parameter `p=n`
* `{{Airport-dest-list|Airline|Dest}}` renders a wikitext table of airlines and destinations
* `{{NWS-current|RJAF}}` renders a current weather link to the NOAA/NWS page for the given ICAO code
* `{{right|content}}` floats the content to the right; without parameters, it returns `style="text-align:right"|` for table cell alignment
* `{{cite peakbagger|pid=2829|name=Mount Whitney}}` formats Peakbagger.com mountain, list, range, or key col citations
* `{{Wikibooks inline|Work|Custom Label}}` (and its alias `{{wikibooks-inline}}`) formats an inline link to Wikibooks
* `{{refh}}` renders reference column header labels: "Refs." (or "Ref." if `multi=no` is specified)
* `{{M|w|7.2}}` (and its aliases `{{m}}` and `{{earthquake magnitude}}`) formats earthquake magnitude scale labels and values
* `{{cite video|title=My Video}}` (alias for `{{cite AV media}}`) formats a citation to video or media
* `{{cite tweet|user=jack|number=20}}` formats a tweet citation link using user/username and status ID
* `{{cite constitution|country=United States|article=I}}` formats a constitution citation
* `{{cite bioRxiv|biorxiv=10.1101/123456}}` formats bioRxiv preprint citations with their DOI suffix
* `{{Harvard citation text|Smith|2020}}` (alias for `{{harvtxt}}`) formats Harvard text citations
* `{{cite MW|entry=word}}` (alias for `{{cite merriam-webster}}`) formats Merriam-Webster dictionary citations
* `{{term|glossary term}}` and `{{defn|definition}}` format glossary term definitions
* `{{cquote|quote|author}}` formats centered blockquotes
* `{{London Gazette|12345|page=12}}` formats London Gazette issues and pages as styled citations
* `{{US$|123.45}}` formats US dollar amounts
* `{{also|Standard solar model}}` renders like other see-also cross-reference templates
* `{{solar radius|1.2}}` and `{{solar radius}}` render values with the solar radius symbol: `1.2 R<sub>☉</sub>` and `R<sub>☉</sub>`
* `{{±|10|2}}` and `{{±}}` render mathematical plus-minus values or characters: `± 10 2` and `±`
* `{{cite encyclopedia|title=Solar activity|encyclopedia=Scholarpedia}}` renders encyclopedia references exactly like other journal references, formatting the title in quotes and the encyclopedia in italics
* `{{Literal translation|Eastern Capital}}` (or `{{lit}}`) displays the literal meaning of a term prefixed by "lit. "
* `{{Translation|word|literal=yes}}` (or `{{Translation|word}}`) renders translation terms with optional literal annotation, leveraging abbreviation tooltips (e.g. `transl. word` or `lit. transl. word`)
* `{{N/A}}` (or its aliases `{{NA}}` and `{{Not applicable}}`) displays the text "N/A" or the custom parameter provided inside comparison tables
* `{{'"}}` and `{{"'}}` display `'"` and `"'` respectively as ordinary text
* `{{frac2|1|3}}` is an alias for `{{sfrac}}` and formats a diagonal fraction
* `{{vanchor|Mercury}}` is an alias for `{{visible anchor}}` and displays the anchor text
* `{{block indent|content}}` formats an indented block of text as a blockquote
* `{{dfni|term}}` formats a definition term in italics
* `{{radic|expr|degree}}` formats mathematical expressions involving the radical symbol
* `{{diagonal split header|Bottom-Left-Text|Top-Right-Text}}` formats table headers split diagonally
* `{{pipe}}` displays a literal pipe character: `|`
* `{{legend-line|style|label}}` formats a legend row with a colored line, displaying the label
* `{{prime|x}}` formats single-prime symbols: `x′`
* `{{isup|st}}` formats superscripted text with default or custom padding
* `{{CJKV|t=繁|s=简|p=pinyin|l=literal}}` compares Chinese, Japanese, Korean, and Vietnamese terms with standardized labeling
* `{{udl|wrap=content}}` formats an unindented description list wrapping its content
* Wikipedia navigation templates listed in `src/navigations.csv`  are omitted; both `src/navigations.csv` and `src/silent.csv` support comma-separated comments, ignoring any text after the comma in the code; template names are normalized by converting underscores to spaces before checking for matches.
* Wikipedia succession-box templates such as `{{Succession box}}` or those whose names start with `s-`, such as `{{s-start}}`, `{{s-bef}}`, `{{s-ttl}}`, and `{{s-end}}`, are omitted
* Maintenance and metadata templates such as `{{unreferenced section}}`, `{{Excessive citations inline}}`, `{{More citations needed}}`, `{{additional citations needed}}`, `{{Refimprove}}`, `{{FACT}}`, `{{citation needed}}`, `{{cn}}`, `{{huh}}`, `{{when}}`, `{{who}}`, `{{By whom}}`, `{{more cn section}}`, `{{prose}}`, `{{Unreliable source?}}`, `{{Better source needed}}`, `{{Dead link}}`, `{{Page needed}}`, `{{New archival link needed}}`, `{{clear}}`, `{{div}}`, `{{columns-list}}`, `{{location map+}}`, `{{Wide image}}`, `{{Pie chart}}`, `{{ahnentafel}}`, `{{Spoken Wikipedia}}`, `{{very long}}`, `{{long}}`, `{{Explain}}`, `{{Ref}}`, `{{R}}`, `{{Pd-notice}}`, `{{Contains special characters}}`, `{{tree chart}}`, `{{tree chart/start}}`, `{{tree chart/end}}`, `{{tree list}}`, `{{tree list/end}}`, `{{tree list/final branch}}`, `{{tree list/branching}}`, `{{tree list/final branching}}`, `{{chart top}}`, `{{chart bottom}}`, `{{Japanese clan name}}`, `{{-}}`, `{{redirect-several}}`, `{{bots}}`, `{{Div end}}`, `{{Sister bar}}`, `{{Expand section}}`, `{{Unreferencedsect}}`, `{{Clear left}}`, `{{Cleanup}}`, `{{tone}}`, `{{Wikiatlas}}`, `{{update section}}`, `{{party color}}`, `{{category see also}}`, `{{clarify}}`, `{{clarification needed}}`, `{{failed verification}}`, `{{colbegin}}`, `{{colend}}`, `{{POV}}`, `{{dubious}}`, `{{commonscat}}`, `{{Commons-inline}}`, `{{disambiguation}}`, `{{in title}}`, `{{look from}}`, `{{tocright}}`, `{{CS1 config}}`, `{{unsolved}}`, `{{discuss}}`, `{{j-railservice start}}`, `{{j-route}}`, `{{j-rserv}}`, `{{ja-rail-line}}`, `{{pp-dispute}}`, `{{Attribution needed}}`, `{{incomplete short citation}}`, `{{Wikidata fallback link}}`, `{{flagicon image}}`, `{{external media}}`, `{{Wikiquote-inline}}`, `{{wikispecies-inline}}`, `{{IMDb name}}`, `{{PM20}}`, `{{NoteTag}}`, `{{wikisource category}}`, `{{0}}`, `{{end plainlist}}`, `{{surname}}`, `{{sisterlinks}}`, `{{sister links}}`, `{{Wikinews}}`, `{{wikiquote}}`, `{{fv}}`, `{{clear right}}`, `{{clr}}`, `{{empty section}}`, `{{family name hatnote}}`, `{{Bare URL inline}}`, and `{{DEFAULTSORT:...}}` are omitted
* `{{Reflist}}` renders collected `<ref>...</ref>` content as an ordered reference list; grouped reflists such as `{{Reflist|group=n}}` render the matching group when those references are used
* Reference-list wrappers and source metadata such as `{{notelist}}`, `{{notelist-ua}}`, `{{NoteFoot}}`, `{{Refbegin}}`, `{{Refend}}`, `{{SfnRef}}`, and `{{source-attribution}}` are omitted while surrounding list contents are preserved
* Footnote wrappers such as `{{efn|...}}`, `{{efn-ua|...}}`, and `{{refn|...}}` are omitted
* Layout-only column templates such as `{{col-begin}}`, `{{col-break}}`, and `{{col-end}}` are omitted
* Decorative flag image templates such as `{{flagicon|US}}` are omitted
* `== History ==` becomes `<h2>History</h2>`; deeper heading levels use deeper XHTML headings
* Lines starting with `*` become unordered list items
* Lines starting with `#` become ordered list items
* References, unhandled templates, non-wikitable tables, categories, and file/image links are omitted; wikitables are converted into XHTML tables; file/image links are only rendered when `images: true` and the image can be resolved
