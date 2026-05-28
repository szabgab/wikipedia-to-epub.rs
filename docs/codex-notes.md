# Codex Session Notes

## 2026-05-28 Korean language page templates handling and clippy updates

### Summary

This session completed the handling of nine templates used in the Korean Language page: `"IPAslink"`, `"angbr IPA"`, `"unichar"`, `"xlit"`, `"note"`, `"angbr"`, `"fs interlinear"`, `"harvp"`, and `"Tooltip"`. This included addressing potential raw HTML tag-stripping and formatting collisions, and fixing a legacy Clippy warning in the integration test suite.

### Decisions Made

* `harvp` is rendered as a clean, structured Harvard citation wrapped in parentheses supporting page numbers, locations, and multi-author structures.
* `IPAslink` leverages standard `__WIKIPEDIA_TO_EPUB_IPA_START__` formatting.
* `angbr` wraps text inside angle brackets `⟨...⟩`.
* `angbr IPA` wraps the text inside angle brackets and styles the inner text as IPA using the standard `und-fonipa` language tag.
* `unichar` resolves hexadecimal Unicode code points into combined base/glyph strings and appends hexadecimal representations (e.g. `◌͈ (U+0348)`).
* `xlit` routes directly to the existing transliteration renderer.
* `note` renders table footnote labels safely in bold formatting utilizing standard wikitext `'''` notation to avoid raw HTML tag stripping.
* `fs interlinear` renders Foreign Speech Interlinear blocks inside blockquotes using wikitext bold/italics for high EPUB reader styling compatibility, and resolves single quote collisions by converting them to the HTML entity `&#39;`.
* `Tooltip` leverages the standard `__WIKIPEDIA_TO_EPUB_ABBR_START__` parser to produce abbreviations with tooltips.
* Replaced a legacy Clippy expect warning in `tests/books.rs` around line 324 with an explicit `unwrap_or_else` check to ensure strict `clippy --all-targets -- -D warnings` compliance.

### Files Changed

* `src/main.rs`
  * Registered and implemented dispatchers/renderers for all 9 templates.
* `src/tests.rs`
  * Added 9 new exhaustive unit tests for all renderers.
* `tests/books.rs`
  * Resolved a legacy expect-fun-call Clippy warning on line 324.
* `expected/korean-language/`
  * Refreshed book integration Expected XHTML fixtures to include the newly rendered orthography and vowels output.
* `README.md`
  * Documented all nine new template conversion rules.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `./sort.sh`
* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test`

### Pending Follow-Ups

* None.

## 2026-05-27 Japan page templates handling and fixture updates

### Summary

This session completed the handling of templates used in the Japan page: `"redirect-several"`, `"bots"`, `"TOClimit"`, `"nihongo2"`, `"gloss"`, `"xref"`, `"Shy"`, `"color box"`, `"pb"`, and `"OSM relation"`. This included addressing tag-stripping bugs by implementing secure placeholder markers and custom post-stripping restoration logic.

### Decisions Made

* `redirect-several`, `bots`, and `TOClimit` are page-level or control-flow templates and are skipped silently.
* `nihongo2` is rendered using the existing `__WIKIPEDIA_TO_EPUB_LANG_START__` Japanese lang placeholder, ensuring the `<span lang="ja">` block is not stripped during post-processing.
* `gloss` is rendered to wrap text in single quotes, or in parentheses in definition mode (`mode=def`).
* `xref` is processed as a passthrough template.
* `Shy` renders discretionary soft hyphens (`\u{00ad}`) to guide hyphenation behavior.
* `color box` is rendered using a custom `__WIKIPEDIA_TO_EPUB_COLOR_BOX_START__` placeholder that is restored to `<span style="color: {color};">■</span>` after standard tag stripping.
* `pb` (paragraph break) is rendered using a custom `__WIKIPEDIA_TO_EPUB_PB__` placeholder, restored to `<br /><br />` after standard tag stripping.
* `OSM relation` renders OpenStreetMap relation link text using existing OpenStreetMap relation rendering helpers.

### Files Changed

* `src/main.rs`
  * Registered and implemented dispatchers/renderers for `nihongo2`, `gloss`, `Shy`, `color box`, `pb`, `xref`, and `OSM relation`.
  * Added custom `restore_color_box_spans` and `restore_pb_spans` post-stripping restorers.
* `src/silent.csv`
  * Registered `redirect-several`, `bots`, and `TOClimit` as recognized silent templates.
* `expected/japan/`
  * Updated integration expected fixtures (e.g., `OEBPS/chapter-1.xhtml`) using a temporary `UPDATE_FIXTURES` helper injection in the integration test suite.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `./sort.sh`
* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test`

### Pending Follow-Ups

* None.

## 2026-05-27 Han Dynasty templates handling and file logging

### Summary

This session added handling for templates used in Han Dynasty page: `floruit`, `fraction`, `Library resources box`, and `Spoken Wikipedia`. It also updated the tracing configuration to also write all log outputs to a plain-text file called `report.log` without ANSI color escape codes.

### Decisions Made

* `floruit` renders as `fl. <text>` using the first parameter. An empty `floruit` renders as `fl.`.
* `fraction` works as an alias to `frac` rendering positional parameters as reader friendly fractions.
* `Library resources box` and `Spoken Wikipedia` are page-level resources or media metadata templates and are skipped silently.
* The global logging initialization was updated to configure a layered subscriber: a standard output formatter and a file-writing formatter targeting `report.log` with ANSI formatting disabled (`with_ansi(false)`).

### Files Changed

* `src/main.rs`
  * Registered `floruit` and `fraction` in `is_handled_template_name` and dispatched them.
  * Implemented `render_floruit_template`.
  * Updated `init_logging` to write plain-text logs without ANSI escapes to `report.log` using a layered subscriber.
* `src/silent.csv`
  * Added `Library resources box` and `Spoken Wikipedia`.
* `src/tests.rs`
  * Added `render_wikitext_formats_han_dynasty_templates` test.
* `expected/korea/OEBPS/chapter-10.xhtml`
  * Refreshed Han Dynasty expected output after the new template rendering (`floruit` and `fraction`).
* `README.md`
  * Documented the new template conversion and omission rules.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `./sort.sh`
* `cargo test render_wikitext_formats_han_dynasty_templates`
* `cargo test`

### Pending Follow-Ups

* None.

## 2026-05-26

### Summary

This session added handling for Hangul-page templates `tlit`, `crossreference`, `slink`, `nobold`, `Arrow`, `efn-ua`, `notelist-ua`, `col-begin`, `col-break`, and `col-end`, then refreshed the Hangul expected EPUB fixture.

### Decisions Made

* `tlit` renders transliterated text as a Latin-script language span using the first parameter as the language code and the final positional parameter as the visible text.
* `crossreference` renders its inline content, including nested `slink` section links.
* `slink` renders section links for current-page and article-section forms.
* `nobold` is treated as a passthrough wrapper so visible nested content is preserved.
* `Arrow` renders directional arrow glyphs, with `r` rendering as `→`.
* `efn-ua` and `notelist-ua` are footnote/reference-list wrappers and are skipped silently.
* `col-begin`, `col-break`, and `col-end` are layout-only column templates and are skipped silently.

### Files Changed

* `src/main.rs`
  * Added dispatch, handled-name recognition, and renderers for `tlit`, `crossreference`, `slink`, `nobold`, and `Arrow`.
* `src/silent.csv`
  * Added `efn-ua`, `notelist-ua`, `col-begin`, `col-break`, and `col-end`.
* `src/tests.rs`
  * Added unit coverage for Hangul inline templates and extended silent-template coverage.
* `README.md`
  * Documented the new conversion and omission rules.
* `expected/korea/OEBPS/chapter-9.xhtml`
  * Refreshed the Hangul expected output after the new template rendering.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `cargo fmt`
* `cargo test render_wikitext_formats_hangul_inline_templates`
* `cargo test render_wikitext_silently_skips_metadata_templates`
* `cargo test --test books`
* `cargo test`

### Pending Follow-Ups

* None.

## 2026-05-25

### Summary

This session updated Korean template rendering so Hangul/Hanja values are explicitly labelled and `ko_ipa` pronunciation values are shown, then refreshed the affected English Korea expected EPUB fixtures.

### Decisions Made

* `render_korean_template` now prefixes Hangul output with `Korean:` and Hanja output with `Hanja:`.
* `ko_ipa=` renders as visible pronunciation text such as `pronounced [pusʰa̠n]`.
* The new labels apply to both `Korean` and `Korean/auto`; existing Korean auto marker cleanup remains in place.

### Files Changed

* `src/main.rs`
  * Added `ko_ipa` parsing and labelled Korean/Hanja output in `render_korean_template`.
* `src/tests.rs`
  * Updated Korean template unit coverage for labels and Busan pronunciation.
* `README.md`
  * Updated Korean template conversion rules.
* `expected/korea/OEBPS/*.xhtml`, `expected/korea/OEBPS/content.opf`, `expected/korea/OEBPS/toc.ncx`
  * Refreshed the English Korea expected EPUB output after the Korean rendering change.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `cargo fmt`
* `cargo test render_wikitext_formats_korean_templates`
* `cargo test --test books`
* `cargo test`

### Pending Follow-Ups

* None.

## 2026-05-25

### Summary

This session added handling for the `mdash`, `legend`, `circa`, `cite web`, `SfnRef`, `Britannica`, and `source-attribution` Wikipedia templates observed in `pages/North_Korea.json`, documented the conversion rules, and refreshed the North Korea expected fixture.

### Decisions Made

* `mdash` renders as an em dash so prose no longer loses punctuation.
* `circa` renders as `c.` with an optional following value.
* `legend` keeps the visible legend label and omits the color swatch metadata.
* `cite web` renders as compact bibliography prose with authors, linked title when `url=` is present, website/work, publisher, date, and page details.
* `SfnRef` and `source-attribution` are non-visible citation/source metadata and are skipped silently.
* `Britannica` renders as a visible external link using the Britannica article id.

### Files Changed

* `src/main.rs`
  * Added dispatch, handled-name recognition, and renderers for `mdash`, `legend`, `circa`, `cite web`, and `Britannica`.
* `src/silent.csv`
  * Added `SfnRef` and `source-attribution` as recognized silent templates.
* `src/tests.rs`
  * Added unit coverage for the new inline and web-source template behavior.
* `README.md`
  * Documented the new conversion rules.
* `expected/korea/OEBPS/chapter-8.xhtml`
  * Refreshed the North Korea expected output for the newly rendered templates.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `cargo fmt`
* `cargo test render_wikitext_formats_web_source_templates`
* `cargo test render_wikitext_formats_simple_inline_templates`
* `cargo test --test books`
* `cargo test`

### Pending Follow-Ups

* Broaden `Britannica` rendering if future pages provide article titles or require modern slug URLs instead of id-based links.

## 2026-05-25

### Summary

This session added handling for the `sic`, `Nowrap`, `Smaller`, and `ROKS` Wikipedia templates observed in `pages/South_Korea.json`, documented the conversion rules, and refreshed the South Korea expected fixture.

### Decisions Made

* `sic` preserves its visible correction text and appends `[sic]`; an empty `sic` renders as `[sic]`.
* `Nowrap` is rendered as normal inline text for EPUB output.
* `Smaller` keeps the text visible and wraps it in `<small>...</small>` through the placeholder restore path.
* `ROKS` renders as a link to the ship article with the visible `ROKS` prefix and italic ship name.
* Existing raw HTML handling remains unchanged; raw `<sup>` tags in source content are stripped before XHTML output.

### Files Changed

* `src/main.rs`
  * Added dispatch, handled-name recognition, and renderers for `sic`, `Nowrap`, `Smaller`, and `ROKS`.
  * Added restoration for the internal small-text placeholder.
* `src/tests.rs`
  * Added unit coverage for the new inline template renderers.
* `README.md`
  * Documented the new conversion rules.
* `expected/korea/OEBPS/chapter-7.xhtml`
  * Refreshed the South Korea expected output for the newly rendered templates.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `cargo fmt`
* `cargo test render_wikitext_formats_simple_inline_templates`
* `cargo test --test books`
* `cargo test`

### Pending Follow-Ups

* Broaden `ROKS` behavior if future pages need additional display modes beyond the currently observed South Korea usage.

## 2026-05-25

### Summary

This session added support for the Wikipedia `frac` template so common fractions and mixed numbers render as visible inline text instead of disappearing from EPUB output.

### Decisions Made

* `frac` should render its positional parameters as plain text fractions for EPUB readability.
* Two-parameter forms render as `numerator/denominator`, and three-parameter forms render as mixed numbers like `1 1/2`.
* Nested handled templates inside `frac` parameters should be rendered before the fraction text is assembled.

### Files Changed

* `src/main.rs`
  * Added `frac` to template dispatch and implemented basic positional fraction rendering.
* `src/tests.rs`
  * Added unit coverage for simple, mixed-number, and nested-template `frac` forms.
* `README.md`
  * Documented the `frac` conversion rule.
* `expected/korea/OEBPS/chapter-2.xhtml`
  * Updated the Seoul fixture so `{{frac|2|3}}` now appears as `2/3` in the fortress-wall paragraph.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `rustfmt --edition 2024 src/main.rs src/tests.rs`
* `cargo test render_wikitext_formats_frac_templates`
* `cargo test`

### Pending Follow-Ups

* Extend `frac` support if future pages need more specialized formatting than the common positional forms handled here.

## 2026-05-25

### Summary

This session added support for the Wikipedia `Historical populations` template, rendering year/population entries as visible EPUB-friendly list content and updating the Seoul fixture accordingly.

### Decisions Made

* `Historical populations` should render visible text rather than being dropped with table-like metadata.
* Numeric parameter pairs are interpreted in order as year/population entries, while layout metadata such as `align=` and empty `source=` values are ignored.
* Plain integer population values are formatted with thousands separators for readability in EPUB output.

### Files Changed

* `src/main.rs`
  * Added `Historical populations` to template dispatch and implemented parsing/rendering helpers for year/population entry pairs.
* `src/tests.rs`
  * Added unit coverage for `Historical populations` rendering and metadata omission.
* `README.md`
  * Documented the `Historical populations` conversion rule.
* `expected/korea/OEBPS/chapter-2.xhtml`
  * Updated the Seoul fixture to include the rendered historical population list in the Demographics section.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `rustfmt --edition 2024 src/main.rs src/tests.rs`
* `cargo test render_wikitext_formats_historical_populations_templates`
* `cargo test`

### Pending Follow-Ups

* Extend `Historical populations` support if future pages use non-numeric population values or additional labels that should be surfaced in the rendered output.

## 2026-05-25

### Summary

This session added handling for the Wikipedia `Coord` template so common inline coordinate forms render as readable text while title-only and Wikidata `qid=` metadata cases stay omitted.

### Decisions Made

* `Coord` should render visible text only when `display=` is absent or includes `inline`; `display=title` remains omitted.
* Common positional latitude/longitude forms are supported: degrees/minutes/seconds with hemispheres and signed decimal latitude/longitude pairs.
* Trailing coord metadata such as `region:` or `type:` positional arguments are ignored after the coordinate values are parsed.
* `qid=`-only `Coord` usages are treated as metadata and omitted.

### Files Changed

* `src/main.rs`
  * Added `Coord` to template dispatch and implemented inline coordinate rendering helpers.
* `src/tests.rs`
  * Added unit coverage for DMS, decimal, title-only, and `qid=` `Coord` forms.
* `README.md`
  * Documented the new `Coord` conversion behavior.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `rustfmt --edition 2024 src/main.rs src/tests.rs`
* `cargo test render_wikitext_formats_coord_templates`
* `cargo test`

### Pending Follow-Ups

* Extend `Coord` support if future pages need additional named-parameter forms beyond the currently handled positional formats.

## 2026-05-25

### Summary

This session improved the local-book integration test failure output so EPUB fixture mismatches report the first differing location with short context instead of a huge full-string diff.

### Decisions Made

* Use the `similar` crate as a dev-only dependency to locate the first changed character span instead of maintaining custom diff logic.
* Keep the integration assertion focused on the first differing area, including line, column, nearby context, and total string lengths.
* Leave the underlying Korea fixture mismatch unchanged; the goal here was to make the failure easier to inspect.
* Leave the existing `cargo fmt --check` failure in `src/tests.rs` untouched because it predates this change.

### Files Changed

* `Cargo.toml`
  * Added `similar` as a test-only dependency.
* `Cargo.lock`
  * Recorded the new dev dependency.
* `tests/books.rs`
  * Replaced the raw string equality assertion with a helper that reports the first mismatch location and surrounding context for EPUB entry comparisons.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `cargo fmt --check` *(fails on a pre-existing formatting diff in `src/tests.rs`)*
* `rustfmt tests/books.rs`
* `cargo test render_wikitext_formats_excerpt_templates`
* `cargo test generate_korea_book_from_local_page_dumps -- --exact --nocapture`
* `cargo test`

### Pending Follow-Ups

* Investigate the remaining Korea fixture mismatch now that the failure points to line 41 in `OEBPS/chapter-1.xhtml` near the `Goryeo dynasty` section.

## 2026-05-25

### Summary

Handled Wikipedia `rp` reference-page templates so source page markers are preserved in EPUB output, rendered `Official website` and `Largest cities` templates, skipped additional metadata/layout templates, updated documentation, and refreshed affected expected fixtures.

### Decisions Made

* `{{rp|...}}` and case variants render as inline page locators.
* A single positional page value renders as `p. ...`; multiple positional values render as `pp. ...`.
* The renderer includes a leading space so page markers do not stick to the preceding sentence after `<ref>` tags are removed.
* Nested handled templates inside `rp` parameters are rendered before the page marker text is produced.
* `Official website` renders as a direct external link, using the first positional or `url=` value as the URL and `name=`, `title=`, or the second positional value as the label.
* `Largest cities` renders as an EPUB-friendly heading and bullet list, linking each city and including division and population when present.
* `location map+` is map/layout metadata and is skipped silently, including nested map marker templates inside it.
* `Wikisource-inline`, `Unreliable source?`, `Wide image`, `Pie chart`, `Better source needed`, and `ahnentafel` are layout, provenance, or metadata templates and are skipped silently.

### Files Changed

* `src/main.rs`
  * Added `rp` to handled template dispatch and implemented reference-page rendering.
  * Added `Official website` handling and direct external URL link support.
  * Added `Largest cities` handling that converts city rows into linked list items.
  * Added `location map+`, `Wikisource-inline`, `Unreliable source?`, `Wide image`, `Pie chart`, `Better source needed`, and `ahnentafel` to the silent template list.
* `src/tests.rs`
  * Added unit coverage for single-page, multi-page, case-insensitive, and nested-template `rp` rendering.
  * Added unit coverage for `Official website` URL, label, and protocol-normalization behavior.
  * Added unit coverage for `Largest cities` rendering.
  * Extended metadata skip coverage for `location map+`, `Wikisource-inline`, `Unreliable source?`, `Wide image`, `Pie chart`, `Better source needed`, and `ahnentafel`.
* `README.md`
  * Documented `rp`, `Official website`, `Largest cities`, and additional omitted template conversion rules.
* `expected/korea/OEBPS/chapter-1.xhtml`
  * Updated the Korea fixture so the official Korea website appears as an external link and the largest-cities table appears as a city list.
* `expected/korea/OEBPS/chapter-2.xhtml`
  * Updated the Seoul fixture to include visible `p. 96–111` and `p. 90–100` markers.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `cargo test render_wikitext_formats_reference_page_templates`
* `cargo test render_wikitext_formats_official_website_templates`
* `cargo test render_wikitext_formats_largest_cities_templates`
* `cargo test render_wikitext_silently_skips_metadata_templates`
* `cargo test --test books`
* `cargo test`

### Pending Follow-Ups

* Broaden template support as new unhandled templates appear in source pages.

## 2026-05-24

### Summary

This session focused on expanding Wikipedia template rendering for EPUB output, specifically handling "Succession box", "For timeline", and maintenance templates, updating documentation, and verifying behavior with tests.

### Decisions Made

* Template handling should happen through the shared `{{...}}` parser, with only unhandled templates logged.
* Metadata or maintenance templates are skipped silently when they should not appear in EPUB output.
* `DEFAULTSORT` templates are page-sorting metadata and are skipped silently, including variants that start with `DEFAULTSORT`.
* `Commons category` is treated as a sister-project/category metadata box and skipped silently.
* "Succession box" templates should be handled the same way as "s-" templates, meaning they are silently skipped to avoid unhandled template logs.
* "For timeline" templates are rendered as visible hatnote-style prose to guide the reader to relevant timelines.
* Maintenance templates like "unreferenced section" should be skipped silently.
* Language-specific templates should render EPUB-friendly HTML using `lang` attributes where appropriate.
* Known Korean transliteration templates are rendered directly instead of leaking template syntax into the book.
* `harvc` is treated as a compact chapter/contribution citation; EPUB output keeps the contributor, quoted contribution title, enclosing source key/year, and optional page/location details.
* `As of` is rendered as visible prose, preserving the capitalization option `lc=y` and basic year/month/day date forms.
* `Blockquote` is rendered as block-level XHTML with quote text in `<blockquote><p>...` and optional source text in `p.blockquote-source`.
* `Further` is rendered as visible hatnote-style prose: `Further information:` plus article links, with `topic=` becoming `Further information about ...:`.
* Wikipedia succession-box templates such as `{{Succession box}}` or those whose names start with `s-`, such as `{{s-start}}`, `{{s-bef}}`, `{{s-ttl}}`, and `{{s-end}}`, are treated as navigation/metadata and skipped silently.
* `sclass` is rendered as a ship-class link helper, including italic class names and supported format parameters.
* `For timeline` renders as visible hatnote-style prose: `For a timeline, see:` plus article links.
* `Excerpt` renders as visible hatnote-style prose: `Excerpt from:` plus article links.
* `Reflist`, `notelist`, `Refbegin`, and `Refend` are reference/bibliography layout wrappers; they are skipped silently while preserving surrounding reference-list contents.
* `refn` is treated like other footnote wrappers such as `efn`; it is skipped silently so note text does not appear inline in the EPUB body.
* `flagicon` is treated as decorative image markup and skipped silently; nearby country/city prose remains visible.
* `Wiktionary` renders as visible sister-project prose, linking to the requested Wiktionary entry through the existing external-link path.
* `Wikivoyage` renders as visible sister-project prose, linking to the requested Wikivoyage entry through a dedicated external-link path.
* `Free access` is treated like `Open access`, rendering as the same open-lock marker.
* The README should describe conversion rules with concrete before/after examples.

### Files Changed

* `src/main.rs`
  * Added or extended rendering for templates including `ill`, `Reign`, `lang`, `langx`, `Percentage`, `UN Population`, `Korean/auto`, `Ko-translit`, `Cite report`, `harvc`, `As of`, `Blockquote`, `Further`, `For timeline`, `Wiktionary`, `Wikivoyage`, `sclass`, and `Free access`.
  * Added block-level handling for rendered blockquote markers so quotes are not flattened into ordinary paragraphs.
  * Updated citation author collection so unnumbered `last`/`first` can combine correctly with numbered coauthors such as `last2`/`first2`.
  * Added silent skipping for templates such as `Redirect`, `pp-semi-indef`, `Sfn`, `efn`, `refn`, `Reflist`, `notelist`, `Refbegin`, `Refend`, `flagicon`, `unreferenced section`, `Excessive citations inline`, `DEFAULTSORT`, `Commons category`, `Portal bar`, `Portal`, `Authority control`, `Seoul`, `Seoul weatherbox`, `Seoul landmarks`, `Navboxes`, succession templates prefixed with `s-`, and `Succession box`.
  * Added tests for template rendering behavior, including the restored example fixture and Korean transliteration cases.
* `README.md`
  * Added notes describing wiki-to-HTML conversion rules and template rendering examples.
* `expected/korea/OEBPS/chapter-1.xhtml`
  * Updated the Korea fixture so visible `As of 2023` prose is preserved and the external-links Wiktionary/Wikivoyage templates become real links.
* `expected/korea/OEBPS/chapter-2.xhtml`
  * Updated the Seoul fixture so visible `As of`, `Further`, blockquote prose, and the external-links Wikvoyage template are preserved.
* `expected/korea/OEBPS/chapter-3.xhtml`
  * Updated the Sejong fixture after citation-template, blockquote, `sclass`, and `Free access` rendering changed the generated EPUB output.
* `src/tests.rs`
  * Added unit coverage for `Cite report`, `harvc`, `As of`, `Blockquote`, `Further`, `For timeline`, `Wiktionary`, `Wikivoyage`, `sclass`, `Open access`/`Free access`, `Reflist`/`notelist`, `Refbegin`/`Refend`, `efn`/`refn`, `flagicon`, `unreferenced section`, `Excessive citations inline`, `DEFAULTSORT`, `Commons category`, `Portal bar`, `Portal`, `Authority control`, `Seoul`, `Navboxes`, silent `s-` template handling, and `Succession box`.

### Tests Run

* `cargo test generate_korea_book_from_local_page_dumps`
* `cargo test render_wikitext_silently_skips_metadata_templates`
* `cargo test render_wikitext_formats_blockquote_templates`
* `cargo test render_wikitext_formats_further_templates`
* `cargo test render_wikitext_formats_for_timeline_templates`
* `cargo test render_wikitext_formats_wiktionary_templates`
* `cargo test render_wikitext_formats_wikivoyage_templates`
* `cargo test render_wikitext_formats_ship_class_templates`
* `cargo test render_wikitext_formats_open_access_templates`
* `cargo test --test books`
* `cargo test`

Latest verification passed:

* 49 unit tests passed.
* 4 local book integration tests passed.
* 1 real Wikipedia API test remains ignored by default.

### Pending Follow-Ups

* Broaden template support as new unhandled templates appear in source pages.
* Keep expected EPUB fixtures synchronized whenever rendering behavior intentionally changes.

## 2026-05-26 Image Embedding Session

### Decisions Made

* Added a top-level YAML `images` field, defaulting to `false`, so existing configs continue omitting `[[File:...]]` and `[[Image:...]]` links unless image embedding is explicitly enabled.
* When `images: true`, resolvable file/image links render as XHTML image blocks with optional captions; missing images are warned about and omitted.
* Live runs resolve image metadata through the Wikipedia API and download bounded thumbnails; local `--local` runs use `pages/images/manifest.json` fixture mappings to avoid network access in tests.
* EPUB image assets are written under `OEBPS/images/` and added to the OPF manifest, while chapter XHTML references them with relative `images/...` paths.

### Files Changed

* `src/main.rs`
  * Added image config parsing, image registry/resolution, file-link rendering, EPUB asset writing, OPF manifest entries, and image CSS.
* `src/tests.rs`
  * Added unit coverage for `images` config defaults/explicit enablement and enabled image rendering from local fixtures.
* `tests/books.rs`
  * Added a Busan image-embedding integration test.
* `examples/*.yaml`
  * Added explicit `images: false` to existing examples and added `examples/busan-images.yaml`.
* `pages/images/`
  * Added a local image fixture manifest and small SVG fixtures for Busan integration coverage.
* `expected/busan-images/`
  * Added the new expected EPUB fixture with embedded image assets.
* `expected/*/OEBPS/style.css`
  * Updated expected CSS fixtures for the shared image styles.
* `README.md`
  * Documented the `images` field and the file/image conversion rule.

### Tests Run

* `cargo test render_wikitext_embeds_resolved_file_links_when_images_are_enabled -- --nocapture`
* `cargo test book_config -- --nocapture`
* `cargo test --test books -- --nocapture`
* `cargo test`

Latest verification passed: 78 unit tests passed, 5 local book integration tests passed, and 1 real Wikipedia API integration test remains ignored by default.

### Pending Follow-Ups

* Consider expanding `pages/images/manifest.json` with more fixture images if broader image coverage is useful.
* Remote thumbnail downloads are implemented but are not exercised by default tests because the real Wikipedia API test remains ignored.
* Consider adding more README examples for newly supported templates when their behavior becomes user-visible.

## 2026-05-26 Image Download Logging Session

### Decisions Made

* Image registry entries now track the source article titles that referenced each image, including repeated use from multiple articles.
* Remote image downloads now emit an `info` level log immediately before the HTTP download request, including the resolved image URL and comma-separated source page titles.

### Files Changed

* `src/main.rs`
  * Added `source_pages` tracking to `BookImage`.
  * Passed the chapter title through file-link processing into image registration.
  * Added the pre-download `info` log with `image_url` and `source_pages`.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `cargo test render_wikitext_embeds_resolved_file_links_when_images_are_enabled -- --nocapture`
* `cargo test`

Latest verification passed: 78 unit tests passed, 5 local book integration tests passed, and 1 real Wikipedia API integration test remains ignored by default.

### Pending Follow-Ups

* No known pending follow-ups for this logging change.

## 2026-05-26 Download Cache Session

### Decisions Made

* Live Wikipedia downloads are cached in the OS user cache directory under `wikipedia-to-epub/`.
* `--refresh-cache` forces live page JSON, image metadata JSON, and image file downloads to refresh existing cache entries.
* Local `--local` fixture mode ignores the live download cache.
* Cache entry filenames use fixed-length deterministic hash keys so long image URLs cannot exceed filesystem filename limits.
* Cached image hits log the original image URL and the cache filename before returning the cached bytes.
* Live runs log final JSON-file and image-file counts for needed, cache hit, downloaded, and failed files.
* The `downloading image` log includes the running image download request count.

### Files Changed

* `src/main.rs`
  * Added `--refresh-cache`, download cache path helpers, and read/fetch/write helpers for text and byte cache entries.
  * Cached live article JSON, image metadata JSON, and remote image bytes.
  * Refreshes cached JSON when it exists but cannot be parsed.
  * Switched cache keys from full hex-encoded input strings to fixed-length hashes after long image URLs produced `File name too long`.
  * Added `info` logging for cached image hits with `image_url` and `cached_filename`.
  * Added download statistics for JSON files and image files, including the final `download cache report`.
  * Added `image_download_request_count` to the `downloading image` log.
* `src/tests.rs`
  * Added CLI and cache helper tests for miss, hit, refresh, bytes, and non-ASCII cache keys.
  * Added long-image-URL cache path coverage.
  * Extended byte-cache tests to verify hit/refreshed source reporting.
  * Added download-stat counter coverage for cache miss, cache hit, and failed download paths.
* `README.md`
  * Documented the cache location, refresh flag, local-mode behavior, and final cache report.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `cargo test cache -- --nocapture`
* `cargo test download_cache_paths_are_safe_for_non_ascii_titles -- --nocapture`
* `cargo test read_or_fetch_helpers_update_download_stats -- --nocapture`
* `cargo test --test books -- --nocapture`
* `cargo test`

Latest verification passed: 85 unit tests passed, 5 local book integration tests passed, and 1 real Wikipedia API integration test remains ignored by default.

### Pending Follow-Ups

* No known pending follow-ups for this cache change.

## 2026-05-27 Han Dynasty Template Session

### Decisions Made

* `Pp-pc` is a protection metadata template and is skipped silently.
* `snd` renders as a spaced en dash.
* `died-in` renders compact biographical text such as `d. 202 BC`.
* `zh` and `zhi` reuse the Chinese-language renderer, including pinyin when present.
* `c.` and `cx` are aliases of the existing circa renderer.
* `numero` renders as `No. N`, `anl` renders as a normal article link, and `Wikibooks` renders as a Wikibooks sister-project link.

### Files Changed

* `src/main.rs`
  * Added dispatch, handled-template recognition, and renderers for the Han Dynasty templates.
  * Added Wikibooks URL handling for `b:` links.
* `src/silent.csv`
  * Added `Pp-pc`.
* `src/tests.rs`
  * Added unit coverage for the new inline and sister-project rendering and extended silent-template coverage.
* `README.md`
  * Documented the new conversion rules.
* `expected/korea/OEBPS/chapter-10.xhtml`
  * Refreshed Han Dynasty expected output after the new template rendering.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `cargo test render_wikitext_formats -- --nocapture`
* `cargo test render_wikitext_silently_skips_metadata_templates -- --nocapture`
* `cargo test --test books -- --nocapture`
* `cargo test`

Latest verification passed: 85 unit tests passed, 5 local book integration tests passed, and 1 real Wikipedia API integration test remains ignored by default.

### Pending Follow-Ups

* No known pending follow-ups for this template change.

## 2026-05-25

### Summary

This session added rendering for the `Official website`, `Largest cities`, `linktext`, `Excerpt`, `For`, `URL`, `Webarchive`, `in lang`, `lit`, `ISBN`, `Wikisource`, `Nihongo`, `nbsp`, `cvt`, `osmrelation-inline`, `climate chart`, `IPAc-en`, `Respell`, and `cite ECCP` Wikipedia templates, added per-article and total skipped-template logging, updated README conversion notes, refreshed affected Korea EPUB fixtures, and verified the full test suite.

### Decisions Made

* `Official website` renders as an external link, preserving explicit URL parameters and normalizing bare domains to `https://`.
* `Largest cities` renders as a compact visible list of city links instead of keeping table/navigation markup.
* `linktext` concatenates positional parameters as inline text and renders nested handled templates or links inside those parameters.
* `lang` template text is passed through template rendering before the final language span is emitted, so nested content like `{{lang|zh-hant|{{linktext|漢}}}}` becomes `<span lang="zh-hant">漢</span>`.
* `Excerpt` renders as visible hatnote-style prose: `Excerpt from:` plus article links.
* `For` renders as visible hatnote-style prose: `For <topic>, see:` plus article links.
* `URL` renders as an external link, using parameter `2` as the visible label when available and normalizing bare domains to `https://`.
* `Webarchive` renders as an external archive link labelled `Archived on <date>` when `date=` is present, otherwise `Archived copy`; older positional URL forms are also supported.
* `in lang` renders as visible source-language prose such as `(in Korean)`, including joined output for multiple language codes.
* `lit` renders as inline literal-translation prose such as `lit. Vernacular Script Commission`, preserving nested inline markup.
* `ISBN` renders as inline bibliography prose such as `ISBN 0-8248-0673-5`, preserving nested inline markup.
* `Wikisource` renders as visible sister-project prose: `Wikisource:` plus a link to `https://en.wikisource.org/wiki/...`, preserving subpage slashes in the URL path.
* `Nihongo` renders like `Nihongo4`, including Japanese-language spans and `extra=` content such as nested `lang` output.
* `nbsp` renders as a space so adjacent words are not joined after template removal.
* `cvt` renders as an alias of `convert`.
* `osmrelation-inline` renders as a visible external OpenStreetMap relation link.
* `climate chart` renders as a compact monthly list of low/high temperatures and precipitation values for EPUB readability.
* `IPAc-en` renders as an International Phonetic Alphabet span, joining IPA component parameters while ignoring control words such as `lang`.
* `Respell` renders positional syllables joined with hyphens.
* `cite ECCP` renders compact bibliography text for entries from `Eminent Chinese of the Ch'ing Period`.
* Template skip counting tracks recognized skipped templates separately from unknown skipped templates; per-article and total counts are logged at `info` level, and the final totals are printed after EPUB creation.
* Exact silent-template and observed-navigation template names are stored in `src/silent.csv` and `src/navigations.csv`, then embedded with `include_str!`; prefix-based rules remain in Rust code.
* `columns-list`, `Commons and category`, `Dead link`, `Page needed`, `More citations needed`, `Refimprove`, `FACT`, `citation needed`, `cn`, `anchor`, `huh`, `when`, `more cn section`, `cbignore`, `prose`, `New archival link needed`, `TOC limit`, `NoteFoot`, `clear`, `div`, `Sister project links`, `Busan`, `Busan weatherbox`, and `History of Asia` are layout, maintenance, bot-control, invisible-anchor, or navigation templates and are skipped silently.

### Files Changed

* `src/main.rs`
  * Added `Official website`, `Largest cities`, `linktext`, `Excerpt`, `For`, `URL`, `Webarchive`, `in lang`, `lit`, `ISBN`, `Wikisource`, `Nihongo`, `nbsp`, `cvt`, `osmrelation-inline`, `climate chart`, `IPAc-en`, `Respell`, and `cite ECCP` template rendering.
  * Added external URL link support for official-site rendering.
  * Added OpenStreetMap relation URL support and Japanese interlanguage article URL support.
  * Updated `lang` rendering to resolve nested handled templates in the text parameter.
  * Added silent skipping for the newly observed Busan and Joseon maintenance, layout, and navigation templates.
  * Added skipped-template counters and `info` logs for each article plus aggregate totals.
  * Replaced inline exact silent/navigation template lists with CSV-backed lookup via `include_str!`.
* `src/silent.csv`
  * Added the exact silent-template names previously hard-coded in `is_silent_template_name`.
* `src/navigations.csv`
  * Added the observed navigation-template names previously hard-coded in `is_observed_navigation_template_name`.
* `src/tests.rs`
  * Added unit coverage for `Official website`, `Largest cities`, `linktext`, `Excerpt`, `For`, `URL`, `Webarchive`, `in lang`, `lit`, `ISBN`, `Wikisource`, `Nihongo`, `nbsp`, `cvt`, `osmrelation-inline`, `climate chart`, `IPAc-en`, `Respell`, and `cite ECCP`.
  * Extended silent-template coverage for the newly observed Busan and Joseon maintenance, layout, and navigation templates.
  * Added unit coverage for skipped-template counts and extended the silent-template test to verify its recognized and unknown skip totals.
* `tests/books.rs`
  * Updated CLI stdout assertions to allow the final skipped-template totals line.
* `README.md`
  * Added conversion-rule examples for the new template rendering.
  * Documented that the newly observed Busan maintenance, layout, and navigation templates are omitted.
* `expected/korea/OEBPS/chapter-1.xhtml`
  * Updated expected output for official website, largest-cities, and nested `linktext` rendering.
* `expected/korea/OEBPS/chapter-2.xhtml`
  * Updated expected output for nested `linktext` rendering inside a Chinese language span, visible `URL` links in the Seoul official-sites section, and the `(in Korean)` source-language marker.
* `expected/korea/OEBPS/chapter-3.xhtml`
  * Updated expected output for Sejong's `lit` template around `Ŏnmunch'ŏng`.
* `expected/korea/OEBPS/chapter-4.xhtml`
  * Updated expected output for History of Korea's inline `ISBN` template in the historiography bibliography, visible `Webarchive` links in external links, the top `For` hatnote, and the visible `Wikisource` sister-project link.
* `expected/korea/OEBPS/chapter-5.xhtml`
  * Updated expected output for Busan's `nbsp`, `cvt`, `Nihongo`, `osmrelation-inline`, and `climate chart` rendering.
* `expected/korea/OEBPS/chapter-6.xhtml`
  * Updated expected output for Joseon's `IPAc-en`, `Respell`, and `cite ECCP` rendering.

### Tests Run

* `cargo test render_wikitext_formats_linktext_templates`
* `cargo test render_wikitext_formats_excerpt_templates`
* `cargo test render_wikitext_formats_for_templates`
* `cargo test render_wikitext_formats_in_lang_templates`
* `cargo test render_wikitext_formats_literal_templates`
* `cargo test render_wikitext_formats_isbn_templates`
* `cargo test render_wikitext_formats_wikisource_templates`
* `cargo test render_wikitext_formats_webarchive_templates`
* `cargo test render_wikitext_formats_climate_chart_templates`
* `cargo test render_wikitext_formats_`
* `cargo test render_wikitext_formats`
* `cargo test render_wikitext_silently_skips_metadata_templates`
* `cargo test render_wikitext_reports_template_skip_counts`
* `cargo test --test books`
* `target/debug/wikipedia-to-epub examples/korea.yaml --local pages --log INFO`
* `cargo test`

Latest verification passed:

* 72 unit tests passed.
* 4 local book integration tests passed.
* 1 real Wikipedia API test remains ignored by default.

### Pending Follow-Ups

* Broaden template support as new unhandled templates appear in source pages.
* Keep expected EPUB fixtures synchronized whenever rendering behavior intentionally changes.

## Session Note: 2026-05-27 - Korean War Template Handling

### Decisions Made

* Implemented handling for 7 templates observed in the `pages/Korean_War.json` dump:
  * `For-multi`: alternating topic/link parameters to display clean hatnotes.
  * `Inflation`: calculating US CPI adjustments from 1950 to 2023.
  * `Inflation/year`: returning "2023" to align with our CPI calculations.
  * `stack`: generic passthrough wrapper preserving nested wikitext/links.
  * `USS` / `HMS`: formatted and italicized ship names with links to Wikipedia articles.
  * `Collapsible list`: structured title followed by bulleted items on newlines.
  * `Internet Archive short film`: external link to the Internet Archive short film details.
* Added 8 silent templates to `src/silent.csv` to suppress warning noise: `very long`, `additional citations needed`, `long`, `who`, `R`, `Explain`, `Ref`, and `Pd-notice`. Kept CSVs alphabetically sorted via `./sort.sh`.
* Verified and updated expected integration fixture `expected/korea/OEBPS/chapter-11.xhtml` to account for calculated inflation values, fully resolved ship names, bulleted UN casualties, and film archive links.

### Files Changed

* `src/main.rs`
  * Added handlings, dispatch matches, and robust renderers for `For-multi`, `Inflation`, `Inflation/year`, `stack`, `USS`/`HMS`, `Collapsible list`, and `Internet Archive short film`.
* `src/silent.csv`
  * Added 8 silent templates.
* `src/tests.rs`
  * Implemented a focused unit test `render_wikitext_formats_korean_war_templates` verifying all new rendered and silent templates.
* `README.md`
  * Documented all 7 rendered and 8 silent templates.
* `expected/korea/OEBPS/chapter-11.xhtml`
  * Updated with generated CPI calculations, ship links, lists, and external links.

### Tests Run

* `cargo test render_wikitext_formats_korean_war_templates` (Focused unit test covering all new templates).
* `cargo test --test books` (Integration book test suite).
* `cargo test` (Full test suite of 87 unit tests and 6 integration tests, all passed successfully).

### Pending Follow-Ups

* Keep monitoring and implementing more templates as the book contents evolve.

## Session Note: 2026-05-27 - Hangul & Han Dynasty Template Handling

### Decisions Made

* Implemented handling for three templates requested from `pages/Hangul.json` and `pages/Han_dynasty.json`:
  * `Contains special characters`: Added to `src/silent.csv` to suppress warnings and skip silently. Sorted the CSV using `./sort.sh`.
  * `okina`: Renders Polynesian glottal stop character `ʻ` (U+02BB).
  * `'s`: Renders `'s`.
* Verified and updated expected integration fixtures:
  * `expected/korea/OEBPS/chapter-9.xhtml`: Now renders `University of Hawaiʻi Press` instead of `University of Hawaii Press`.
  * `expected/korea/OEBPS/chapter-10.xhtml`: Now renders `Shiji's` instead of `Shiji` account.

### Files Changed

* `src/main.rs`
  * Handled rendering of `okina` as `ʻ` and `'s` as `'s`, and registered them as handled.
* `src/silent.csv`
  * Added `Contains special characters` to the ignored templates list.
* `src/tests.rs`
  * Added unit test assertions inside `render_wikitext_formats_han_dynasty_templates` covering all three templates.
* `README.md`
  * Documented `okina`, `'s` under inline conversion rules, and `Contains special characters` under maintenance and metadata templates.
* `expected/korea/OEBPS/chapter-9.xhtml`
  * Updated with correct ʻokina character rendering.
* `expected/korea/OEBPS/chapter-10.xhtml`
  * Updated with possessive `'s` template rendering.

### Tests Run

* `cargo test` (Run twice: once with `UPDATE_FIXTURES=1` to update expected integration fixtures, and once normally to verify all 87 unit tests and 5 integration tests pass successfully).

### Pending Follow-Ups

* Continue monitoring pages for unrecognized templates.

## Session Note: 2026-05-27 - Parhae Template Handling

### Decisions Made

* Implemented handling for four templates from `pages/Parhae.json`:
  * `tree chart`, `tree chart/start`, `tree chart/end`: Added to `src/silent.csv` as silent templates to skip visual tree layouts.
  * `-`: Added to `src/silent.csv` as a layout clearing redirect template to skip silently.
  * `cite conference`: Rendered using the robust generic `render_citation_template` function.
  * `worldhistory`: Rendered using `render_worldhistory_template` to output formatted quotes with book citation references.
* Kept `silent.csv` sorted alphabetically via `./sort.sh`.
* Verified and updated expected integration fixture `expected/korea/OEBPS/chapter-12.xhtml` to account for the newly rendered conference bibliography items.

### Files Changed

* `src/main.rs`
  * Added rendering of `cite conference` using `render_citation_template` and defined the `render_worldhistory_template` function. Registered both as handled templates.
* `src/silent.csv`
  * Added `tree chart`, `tree chart/start`, `tree chart/end`, and `-` to the ignored templates.
* `src/tests.rs`
  * Added `render_wikitext_formats_parhae_templates` unit test.
* `README.md`
  * Documented all new conversion rules and omitted templates.
* `expected/korea/OEBPS/chapter-12.xhtml`
  * Updated with correctly formatted conference publication details.

### Tests Run

* `cargo test` (88 unit tests and 5 integration tests pass successfully).

### Pending Follow-Ups

* Continue monitoring pages for unrecognized templates.

## Session Note: 2026-05-27 - Unit Test Refactoring

### Decisions Made

* Refactored unit tests in `src/tests.rs` to break apart file-based grouped test functions (like `render_wikitext_formats_parhae_templates` and the Han Dynasty additions) into separate, template-specific, and granular unit tests.
* Kept all existing assertions completely preserved but separated them into dedicated test cases with names reflecting the exact templates they test.

### Files Changed

* `src/tests.rs`
  * Extracted grouped tests into `render_wikitext_formats_okina_template`, `render_wikitext_formats_possessive_s_template`, `render_wikitext_silently_skips_contains_special_characters_template`, `render_wikitext_formats_cite_conference_template`, `render_wikitext_formats_worldhistory_template`, and `render_wikitext_silently_skips_tree_chart_and_hyphen_templates`.

### Tests Run

* `cargo test` (93 unit tests and 5 integration tests pass successfully with 100% success rate).

### Pending Follow-Ups

* Keep tests granular and specific when adding new templates in future sessions.

## Session Note: 2026-05-27 - Cargo Clippy Lint Fixes

### Decisions Made

* Resolved all 5 `cargo clippy` compiler warnings/errors under `-D warnings` to clean up the codebase and prevent regressions:
  * Collapsed identical if blocks for `cite conference` and `citation` using the `||` operator.
  * Collapsed nested if blocks using the `let_chains` feature in `render_for_multi_template` and `strip_file_links`.
  * Replaced consecutive `replace` calls in `parse_template_number` with a single `replace([',', ' '], "")` call.
  * Removed needless borrow of `title` in remote image processing.

### Files Changed

* `src/main.rs`
  * Applied all clippy changes to clean up borrows, collapsible if blocks, replace calls, and identical arms.

### Tests Run

* `cargo clippy --all-targets -- -D warnings` (Successfully passed with 0 warnings/errors).
* `cargo test` (93 unit tests and 5 integration tests pass successfully with 100% success rate).

### Pending Follow-Ups

* Keep code clean and continue running cargo clippy regularly to verify lint rules are followed.
