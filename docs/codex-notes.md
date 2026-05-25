# Codex Session Notes

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
* Consider adding more README examples for newly supported templates when their behavior becomes user-visible.

## 2026-05-25

### Summary

This session added rendering for the `Official website`, `Largest cities`, `linktext`, and `Excerpt` Wikipedia templates, updated README conversion notes, refreshed affected Korea EPUB fixtures, and verified the full test suite.

### Decisions Made

* `Official website` renders as an external link, preserving explicit URL parameters and normalizing bare domains to `https://`.
* `Largest cities` renders as a compact visible list of city links instead of keeping table/navigation markup.
* `linktext` concatenates positional parameters as inline text and renders nested handled templates or links inside those parameters.
* `lang` template text is passed through template rendering before the final language span is emitted, so nested content like `{{lang|zh-hant|{{linktext|漢}}}}` becomes `<span lang="zh-hant">漢</span>`.
* `Excerpt` renders as visible hatnote-style prose: `Excerpt from:` plus article links.

### Files Changed

* `src/main.rs`
  * Added `Official website`, `Largest cities`, `linktext`, and `Excerpt` template rendering.
  * Added external URL link support for official-site rendering.
  * Updated `lang` rendering to resolve nested handled templates in the text parameter.
* `src/tests.rs`
  * Added unit coverage for `Official website`, `Largest cities`, `linktext`, and `Excerpt`.
* `README.md`
  * Added conversion-rule examples for the new template rendering.
* `expected/korea/OEBPS/chapter-1.xhtml`
  * Updated expected output for official website, largest-cities, and nested `linktext` rendering.
* `expected/korea/OEBPS/chapter-2.xhtml`
  * Updated expected output for nested `linktext` rendering inside a Chinese language span.

### Tests Run

* `cargo test render_wikitext_formats_linktext_templates`
* `cargo test render_wikitext_formats_excerpt_templates`
* `cargo test --test books`
* `cargo test`

Latest verification passed:

* 53 unit tests passed.
* 4 local book integration tests passed.
* 1 real Wikipedia API test remains ignored by default.

### Pending Follow-Ups

* Broaden template support as new unhandled templates appear in source pages.
* Keep expected EPUB fixtures synchronized whenever rendering behavior intentionally changes.
