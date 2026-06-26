# Codex Session Notes

## 2026-06-26 Handle I-Series Wikipedia Templates

### Summary
Handled all 102 templates listed in `i.txt` by checking their `Template:NAME` definitions on English Wikipedia, adding active renderers for behavior-bearing templates, registering layout/navigation/maintenance templates as recognized silent templates, adding per-template unit tests, and updating conversion documentation.

### Decisions Made
- Used the `handle-template` skill for Wikipedia template rendering work.
- Added active renderers and aliases for I-series transliteration/IPA, external-link, country-code, sports-link, rupee-formatting, indentation, interpolation, ISBN-wrapper, and trend-marker templates.
- Registered navigation, sidebar, layout-only image-label, maintenance, and metadata templates in `src/silent.csv` so they are recognized and omitted instead of counted as unknown.
- Ran `./tools/sort.sh`, which sorted CSV files under `src`, including `src/navigations.csv`.

### Files Changed
- `DEVELOPMENT.md`
- `src/silent.csv`
- `src/navigations.csv`
- `src/templates/formatting.rs`
- `src/templates/lang.rs`
- `src/templates/mod.rs`
- `docs/codex-notes.md`

### Tests Run
- `cargo test templates::tests`
- `cargo fmt`
- `./tools/sort.sh`
- `cargo test`
- `cargo check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --locked -- --ignored`

### Pending Follow-Ups
- None.

## 2026-06-26 Handle Eleventh Batch of Wikipedia Templates

### Summary
Added support for 137 Wikipedia templates listed in `f.txt` by registering them in template databases (silent/navigations), implementing active handlers and aliases in Rust, writing separate unit tests for all active templates, and updating documentation rules.

### Decisions Made
- **Created Active Renderers & Aliases in Rust**:
  - Implemented `render_f1_template` (Formula One season link based on year boundary > 1980).
  - Implemented `render_f2_template` (Formula 2 Championship link based on year boundary > 2016).
  - Implemented `render_f1_gp_template` (Formula One Grand Prix link formatter).
  - Implemented `render_facebook_template` (Facebook profile link via official-url protocol).
  - Implemented `render_fifa_player_template` (FIFA player profile link via official-url protocol).
  - Implemented `render_failure_template` (table cell status template with red background styling).
  - Implemented flag template helpers `render_fb_template`, `render_fbw_template`, `render_fsw_template`, `render_futsal_template` (national men's/women's football/futsal teams linking wrappers).
  - Implemented `render_fbu_template`, `render_fbwu_template` (national youth under-NN football teams).
  - Implemented `render_fba_template` (national football association link).
  - Implemented `render_flag_plus_link_template` (prefix combined with country link helper).
  - Implemented `render_flag_athlete_template` (sports competitor with country code annotation).
  - Implemented `render_flagg_template` (general flag linker helper).
  - Implemented `render_flag_ioc_template` (Olympic national country link using `resolve_ioc_code_to_name` helper).
  - Implemented `render_flag_ioc_medalist_template` (Olympic medalist with country name).
  - Implemented `render_flaglink_template` (country subpage suffix linker).
  - Implemented `render_flaglist_template` (list aligned flag country linker).
  - Implemented `render_flagu_template` (unlinked country flag helper).
  - Implemented `render_fs_player_template` (football squad player bulleted list item formatter).
  - Implemented `render_football_box_template` (football match summary details layout).
  - Implemented `render_format_price_template` (formats numerical prices with digit multipliers: thousand/million/billion/etc.).
  - Registered country flag shorthand wrappers: `render_fin_template` (Finland), `render_fji_template` (Fiji), `render_fr_template`/`render_fra_template` (France), `render_frg_template` (West Germany), `render_fsm_template` (Micronesia).
  - Mapped `"f1 race"` to generic infobox renderer, `"font"` / `"font"` to passthrough renderer, and `"further information"` to further template.
  - Registered all active templates and aliases in the dispatch table in `formatting.rs` (matching lowercase forms).
- **Classified Skipped Templates**:
  - Classified 23 templates as silent/maintenance in `silent.csv` (e.g. Featured list, Fix, Flora Europaea, Fs start/mid/end, etc.).
  - Classified 64 templates as navigations/sidebars in `navigations.csv` (e.g. Fejér, Fencing, FIFA navbox, Formula One, FRG 1989, etc.).
  - Sorted both CSV databases using `./tools/sort.sh`.
- **Added Unit Tests**:
  - Wrote 33 separate unit test cases in `src/tests.rs` covering all active templates and aliases. Corrected test expectations for link targets to ensure consistency with standard wikitext parsing.
- **Updated Documentation**:
  - Added conversion rules for all new templates in `DEVELOPMENT.md`.

### Files Changed
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
- [src/navigations.csv](file:///opt/src/navigations.csv) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked formatting: `cargo fmt` (passed cleanly).
- Checked compilation: `cargo check` (passed cleanly).
- Checked warning lints: `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified unit and integration tests: `cargo test` (all passed successfully).
- Verified ignored tests: `cargo test --locked -- --ignored` (all passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-26 Handle Tenth Batch of Wikipedia Templates

### Summary
Added support for 149 Wikipedia templates listed in `e.txt` by registering them in template databases (silent/navigations), implementing custom rendering logic, writing separate unit tests, and updating documentation rules.

### Decisions Made
- **Created Renderers & Aliases in Rust**:
  - Implemented `render_ethnologue_citation` helper to support language references and implemented specific wrappers for `render_e18_template`, `render_e19_template`, `render_e21_template`, `render_e25_template`, and `render_e27_template`.
  - Implemented `render_efloras_template` (botanical taxon citation), `render_etymology_template` (formatted word origin listings), `render_estimate_template` (estimated values with confidence intervals), `render_estimation_template` (abbreviated estimated numbers), `render_equation_ref_template` (equation reference anchor labels), and `render_euro_template` (formats currency values with the Euro sign €, and optionally links to the Euro article).
  - Implemented `render_encyclopaedia_iranica_template`, `render_encyclopaedia_of_islam_new_edition_template`, and `render_ei3_template` to format encyclopedia citations.
  - Implemented flag template helpers `render_egy_template` (Egypt), `render_eri_template` (Eritrea), `render_esa_template` (El Salvador), `render_esp_template` (Spain), `render_estonia_flag_template` (Estonia), `render_eth_template` (Ethiopia), `render_eu_template` (European Union), and `render_ecu_template` (Ecuador).
  - Registered all new active template handlers in `get_dispatch_table()` and `get_dispatch_template_params()` in [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) and [src/templates/citation.rs](file:///opt/src/templates/citation.rs).
  - Resolved name collision between Estonia (`EST` flag) and the estimate abbreviation (`est`/`est.`) by implementing a case-sensitive routing function `render_est_dispatch_template` registered in `get_dispatch_template_params()`.
  - Appended fixed mappings for `"emdash"` and `"eunum"` in `get_fixed()` in [src/templates/mod.rs](file:///opt/src/templates/mod.rs).
- **Classified Skipped Templates**:
  - Appended 23 templates to [src/silent.csv](file:///opt/src/silent.csv) (e.g. eastern name order, EB1911 poster, editorializing, efn-lg, Emblem table, EMedicine, end Div col, Endorsements box, Exchange Rate, Expand list, etc.).
  - Appended 99 navigation and sidebar templates to [src/navigations.csv](file:///opt/src/navigations.csv) (covering Early Modern Europe, Eastern Bloc sidebar, economics, education, etc.).
  - Sorted both CSV databases using `./tools/sort.sh`.
- **Added Unit Tests**:
  - Added 25 separate unit test cases in [src/tests.rs](file:///opt/src/tests.rs) (covering each newly added template/alias).
- **Updated Documentation**:
  - Documented conversion rules for the new active templates in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
- [src/templates/citation.rs](file:///opt/src/templates/citation.rs) [MODIFY]
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
- [src/navigations.csv](file:///opt/src/navigations.csv) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked formatting: `cargo fmt` (passed cleanly).
- Checked compilation: `cargo check` (passed cleanly).
- Checked warning lints: `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified unit and integration tests: `cargo test` (all passed successfully).
- Verified ignored tests: `cargo test --locked -- --ignored` (all passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-26 Handle Ninth Batch of Wikipedia Templates

### Summary
Added support for 82 Wikipedia templates listed in `d.txt` by registering them in template databases (silent/navigations), implementing custom rendering logic, writing unit tests for each template, and updating documentation rules.

### Decisions Made
- **Created Renderers & Aliases in Rust**:
  - Implemented `render_date_template` (formats date parameters), `render_daterangedash_template` (formats date range with dash), `render_death_date_template` (formats death date), `render_death_date_and_age_template` (formats death date and age at death), `render_decimal_cell_template` (table cell decimal align value passthrough), `render_decrease_template` (renders trend down arrow ▼), `render_details_template` (summary style hatnote link list), `render_details_link_template` (details link shorthand), and `render_d_out_template` (gray table cell debate status text).
  - Implemented flag template helpers `render_den_template`/`render_dnk_template` (Denmark), `render_deu_template` (Germany), `render_dji_template` (Djibouti), `render_dma_template` (Dominica), `render_dom_template` (Dominican Republic), and `render_dza_template` (Algeria).
  - Registered all new active templates and aliases (including `"date table sorting"` mapping to `render_dts_template`, and `"down"`, `"decreaseneutral"`, `"decreasepositive"` mapping to `render_decrease_template`) in `get_dispatch_table()` in [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
- **Classified Skipped Templates**:
  - Appended 13 templates to [src/silent.csv](file:///opt/src/silent.csv) (e.g. Data missing, date?, DDB, Discogs artist, Disputed, etc.).
  - Appended 48 navigation and sidebar templates to [src/navigations.csv](file:///opt/src/navigations.csv) (e.g. Dacian cities, Dacia series, Danube Rectors Conference, democracy, etc.).
  - Sorted both CSV databases using `./tools/sort.sh`.
- **Added Unit Tests**:
  - Added 17 separate unit test cases in [src/tests.rs](file:///opt/src/tests.rs) (covering each newly added template).
- **Updated Documentation**:
  - Documented conversion rules for the new active templates in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
- [src/navigations.csv](file:///opt/src/navigations.csv) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked formatting: `cargo fmt` (passed cleanly).
- Checked compilation: `cargo check` (passed cleanly).
- Checked warning lints: `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified unit and integration tests: `cargo test` (all passed successfully).
- Verified ignored tests: `cargo test --locked -- --ignored` (all passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-26 Handle Eighth Batch of Wikipedia Templates

### Summary
Added support for 59 Wikipedia templates listed in `c.txt` by registering them in template databases (silent/navigations), implementing custom rendering logic, writing separate unit tests, and updating documentation rules.

### Decisions Made
- **Created Renderers & Aliases in Rust**:
  - Implemented `render_ce_template` (formats CE/BCE suffix), flag template helpers `render_caf_template` (Central African Republic), `render_cam_template` (Cambodia), `render_can_template` (Canada), `render_cha_template` (Chad), and `render_che_template` (Switzerland).
  - Implemented `render_celex_template` (formats EUR-Lex query string links) and `render_census_2021_aus_template` (citation QuickStats / community profiles URL formatting for Census 2021 Australia).
  - Registered `"centre"` to `render_passthrough_template` alias.
  - Implemented `render_cath_ency_template` to format Wikisource public-domain citations of the 1913 Catholic Encyclopedia, and registered `"cathency"`, `"catholic encyclopedia"`, and `"ce1913"` mapping to it.
  - Registered all new active template handlers in `get_dispatch_table()` in [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) and [src/templates/citation.rs](file:///opt/src/templates/citation.rs).
- **Classified Skipped Templates**:
  - Appended 3 templates to [src/silent.csv](file:///opt/src/silent.csv): "Chart", "CCBYSASource", "CC-notice".
  - Appended 43 navigation and sidebar templates to [src/navigations.csv](file:///opt/src/navigations.csv) (covering Calvinism, Capitalism, Catholic hierarchy, Central banks, Central Intelligence Agency, etc.).
  - Sorted both CSV databases using `./tools/sort.sh`.
- **Added Unit Tests**:
  - Added 15 new unit test cases in [src/tests.rs](file:///opt/src/tests.rs) (covering each newly active/silent template batch).
- **Updated Documentation**:
  - Documented conversion rules for the new active templates in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
- [src/templates/citation.rs](file:///opt/src/templates/citation.rs) [MODIFY]
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
- [src/navigations.csv](file:///opt/src/navigations.csv) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked formatting: `cargo fmt -- --check` (passed cleanly).
- Checked compilation and warning lints: `cargo check` and `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified unit and integration tests: `cargo test` (all passed successfully).
- Verified ignored tests: `cargo test --locked -- --ignored` (all passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-26 Handle Seventh Batch of Wikipedia Templates

### Summary
Added support for 42 Wikipedia templates listed in `a.txt` by registering them in template databases (silent/navigations), implementing custom rendering logic, adding country flag aliases, writing unit tests, and updating documentation rules.

### Decisions Made
- **Created Renderers & Aliases in Rust**:
  - Implemented `render_age_in_years_months_days_template` (date duration formatting as "X years, Y months and Z days"), `render_aircontent_template` (standardized see-also/related aircraft sections formatting), `render_aircraft_specs_template` (formatted characteristics/engine/performance/armament specs), `render_aljazeera_topic_template` (formatted external link), `render_a_or_an_template` (evaluating correct indefinite article choice), `render_bar_box_template` (bar graph container list layout), `render_bar_chart_template` (multi-column bar graph values list layout), `render_bartable_template` (inline bar value and unit string), and `render_bce_template` (date BC suffix formatting).
  - Implemented flag template helpers `render_ban_template` (Bangladesh), `render_bel_template` (Belgium), and `render_bdi_template` (Burundi) calling `render_country_flag_template`.
  - Registered all new templates in `get_dispatch_table()` in [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
- **Classified Skipped Templates**:
  - Appended 3 templates to [src/silent.csv](file:///opt/src/silent.csv): "Airport-Statistics", "Airport statistics", "Being merged from".
  - Appended 27 templates to [src/navigations.csv](file:///opt/src/navigations.csv): "AARC", "Air forces", "Air forces in Europe", "Airports in Hungary", "Allied Air Command", "Allied Land Command", "anti-war", "Bács-Kiskun", "Baja District", "Balkan Wars", "Banks of Hungary", "Banska Stiavnica District", "Baptist", "Baranya", "Barbarian kingdoms", "Bard college", "basic forms of government", "Battle of Stalingrad", "BBC", "BBC Local TV", "BBC News", "BBC Online", "BBC sidebar", "BBC World Service", "Bekes", "Békés", "Békéscsaba District".
  - Sorted both CSV databases using `./tools/sort.sh`.
- **Added Unit Tests**:
  - Added 15 new unit test cases in [src/tests.rs](file:///opt/src/tests.rs) to cover each newly implemented template.
- **Updated Documentation**:
  - Added conversion rules for the new templates in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
- [src/navigations.csv](file:///opt/src/navigations.csv) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked formatting: `cargo fmt -- --check` (passed cleanly).
- Checked compilation and warning lints: `cargo check` and `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified unit and integration tests: `cargo test` (all passed successfully).
- Verified ignored tests: `cargo test --locked -- --ignored` (all passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-25 Clean and Format Math Tags and Templates

### Summary
Fixed math tag processing and `{{tmath}}` template rendering to ensure formulas are cleaned of raw LaTeX/TeX formatting commands and output as human-readable mathematical unicode/plain-text. Regenerated expected book integration fixtures and sorted CSV databases.

### Decisions Made
- **Cleaned and Formatted LaTeX**:
  - Implemented `clean_math_latex(latex: &str) -> String` to sanitize and strip TeX syntax (backslashes, spacing commands, fraction/binomial structures, square roots, double-struck characters, and general symbols) into clear plain-text mathematical strings.
  - Implemented `wrap_frac_term(term: &str) -> String` to cleanly format fraction numerators and denominators.
- **Modified Math Tags and tmath Processing**:
  - Preserved `<math>` tags in `remove_some_html_tags` inside [src/wikitext.rs](file:///opt/src/wikitext.rs) and cleaned their contents using `clean_math_latex`.
  - Updated `render_tmath_template` in [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) to clean its contents via `clean_math_latex`.
- **Regenerated Expected Book Fixtures**:
  - Re-ran the book regeneration scripts to update expected integration fixtures for affected books: `Binomial_distribution`, `Normal_distribution`, `Standard_deviation`, `Variance`, `Statistical_model`, `Statistics`, and `planets`.
- **Sorted CSV Files**:
  - Ran `./tools/sort.sh` on CSV databases in `src/` to ensure consistent alphabetical ordering.
- **Updated Documentation**:
  - Documented math conversion rules in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/tools.rs](file:///opt/src/tools.rs) [MODIFY]
- [src/wikitext.rs](file:///opt/src/wikitext.rs) [MODIFY]
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- [src/navigations.csv](file:///opt/src/navigations.csv) [MODIFY]
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
- [tests/books.rs](file:///opt/tests/books.rs) [MODIFY]
- expected integration fixtures for `Binomial_distribution`, `Normal_distribution`, `Standard_deviation`, `Variance`, `Statistical_model`, `Statistics`, and `planets` [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked formatting: `cargo fmt` (passed cleanly).
- Checked compilation and warning lints: `cargo check` and `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified unit and integration tests: `cargo test` (all passed successfully).
- Verified ignored tests: `cargo test --locked -- --ignored` (all passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-24 Handle Sixth Batch of Wikipedia Templates

### Summary
Added support for 67 Wikipedia templates listed in `x.txt` (Country Flags, Active Renderers, Active Aliases, Silent/Maintenance, and Navigation/Sidebar boxes).

### Decisions Made
- **Created Renderers & Aliases in Rust**:
  - Registered Country Flag templates (`ALB`, `ALG`, `AND`, `ARE`, `ARG`, `ARM`, `ATG`, `AUS`, `AUT`, `AZE`) mapping to `render_country_flag_template` in [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
  - Implemented `render_army_template` (supporting national adjective matching and custom name overriding), `render_aud_template` (Australian Dollar currency formatting), `render_anli_template` (annotated link wrapper), and `render_annotated_image_template` (resolving to standard wikitext file markup) in [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
  - Registered `"asof"` (aliased to `render_as_of_template`) and `"awrap"` (aliased to `render_passthrough_template`) in [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
  - Registered `"angle bracket"` (aliased to `render_angbr_template`) in [src/templates/lang.rs](file:///opt/src/templates/lang.rs).
  - Registered `"arxiv"` (implementing `render_arxiv_link_template`) and `"asn accident"` (implementing `render_asn_accident_template`) in [src/templates/citation.rs](file:///opt/src/templates/citation.rs).
  - Added `"asterisk"` mapping (resolving to literal `*`) in `get_fixed()` in [src/templates/mod.rs](file:///opt/src/templates/mod.rs).
- **Classified Skipped Templates**:
  - Classified 5 templates as silent/maintenance in [src/silent.csv](file:///opt/src/silent.csv): "AN chess", "As of?", "Authority control (arts)", "Automatic taxobox", "Automotive engine".
  - Classified 42 templates as navigations/sidebars in [src/navigations.csv](file:///opt/src/navigations.csv): "Albanian bread", "Alcoholic beverages", "alcoholic drinks", "AMS Presidents", "Anabaptist vertical", "Ancient Egypt dynasties sidebar", "Ancient Egypt graphical timeline", "Ancient Egypt topics", "Ancient Roman Wars", "Ancient Rome military sidebar", "Ancient Rome topics", "Ancient seafaring", "Anglicanism", "Anti-communism", "Antique Kings of Italy", "Antisemitism", "Antisemitism topics", "Application of wind energy", "Archhistory", "Architecture in the United States", "Archival records", "Areas of London", "Armenian language", "Armenians", "Armenia topics", "Armies in Europe", "Army Group Rear Area (Wehrmacht)", "Articles on first-level administrative divisions of European countries", "Articles on second-level administrative divisions of European countries", "Art of Europe", "Aspects of capitalism", "Association football tactics and skills", "Association football terminology", "Atatürk sidebar", "Augustus", "Austrian archdukes", "Austria topics", "Austro-Hungarian claimants", "authoritarian", "authoritarian types of rule", "Autonomous types of first-tier administration", "Avant-garde".
  - Sorted both CSV databases alphabetically using `./tools/sort.sh`.
- **Added Unit Tests**:
  - Wrote 67 separate unit test cases in [src/tests.rs](file:///opt/src/tests.rs) (one for each template) to verify exact behavior.
- **Updated Documentation**:
  - Documented new template conversion rules in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
- [src/templates/lang.rs](file:///opt/src/templates/lang.rs) [MODIFY]
- [src/templates/citation.rs](file:///opt/src/templates/citation.rs) [MODIFY]
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
- [src/navigations.csv](file:///opt/src/navigations.csv) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and formatting: `cargo fmt`, `cargo check`, and `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified unit and integration tests: `cargo test` (all passed successfully).
- Verified ignored tests: `cargo test --locked -- --ignored` (all passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-24 Handle Fifth Batch of Wikipedia Templates

### Summary
Added support for 14 additional Wikipedia templates: "about other people", "About year", "ABW", "according to whom", "Additional citation needed", "AE", "AFG", "age in years", "yes", "yes2", "AGO", "AIA", "align", "AllMusic".

### Decisions Made
- **Created Renderers & Aliases in Rust**:
  - Implemented `render_country_flag_template` (supporting name overrides and historical flag variants) and helper methods `render_abw_template`, `render_afg_template`, `render_ago_template`, and `render_aia_template` in [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
  - Implemented `render_align_template` (handling alignment and custom content) in [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
  - Implemented `render_yes_template` and `render_yes2_template` (formatting table cell approval tags with background color styling) in [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
  - Implemented `render_ae_template` (Avestan language wrapper) in [src/templates/lang.rs](file:///opt/src/templates/lang.rs).
  - Implemented `render_allmusic_template` (formatting artist/album links to AllMusic database) in [src/templates/citation.rs](file:///opt/src/templates/citation.rs).
  - Registered `"abw"`, `"afg"`, `"ago"`, `"aia"`, `"align"`, `"yes"`, `"yes2"`, and `"age in years"` (aliasing `render_age_template`) in `get_dispatch_table()` in [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
  - Registered `"ae"` in `get_dispatch_table()` in [src/templates/lang.rs](file:///opt/src/templates/lang.rs).
  - Registered `"allmusic"` in `get_dispatch_table()` in [src/templates/citation.rs](file:///opt/src/templates/citation.rs).
- **Classified Skipped Templates**:
  - Appended 4 templates to [src/silent.csv](file:///opt/src/silent.csv): "about other people", "About year", "according to whom", "Additional citation needed".
  - Sorted CSV databases using `./tools/sort.sh`.
- **Added Unit Tests**:
  - Wrote 14 separate unit test cases in [src/tests.rs](file:///opt/src/tests.rs) (one for each template) to verify exact behavior.
- **Updated Documentation**:
  - Documented new template conversion rules in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
- [src/templates/lang.rs](file:///opt/src/templates/lang.rs) [MODIFY]
- [src/templates/citation.rs](file:///opt/src/templates/citation.rs) [MODIFY]
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and formatting: `cargo fmt`, `cargo check`, and `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified unit and integration tests: `cargo test` (all passed successfully).
- Verified ignored tests: `cargo test --locked -- --ignored` (all passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-24 Upgrade reqwest Crate

### Summary
Upgraded `reqwest` crate dependency to version `0.13.4`.

### Decisions Made
- **Updated Cargo.toml**:
  - Upgraded `reqwest` dependency version to `"0.13.4"`.
  - Replaced the deprecated `rustls-tls` feature with `rustls` (new naming in `reqwest` v0.13).
  - Explicitly enabled the `"query"` feature on `reqwest` since the `.query(...)` method on `RequestBuilder` has been split out into a separate feature flag.

### Files Changed
- [Cargo.toml](file:///opt/Cargo.toml) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation: `cargo check` (passed cleanly).
- Ran standard formatting, warnings check, and lints: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified unit and integration tests: `cargo test` (all passed successfully).
- Verified ignored/live API tests: `cargo test --locked -- --ignored` (all passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-24 Handle Fourth Batch of Wikipedia Templates

### Summary
Added support for 22 additional Wikipedia templates: "cite periodical", "co2", "East Japan Railway Company Lines", "Fukuoka Stock Exchange", "Fukuoka transit", "Hair space", "HakoneFujiIzuTransit", "Hokuriku Main Line RDT", "Hokuriku_Main_Line_(undivided)", "JRSSN", "Kyushu Railway Company Lines", "Nagoya transit", "Okayama transit", "Osaka transit", "Primary sources", "Rapid transit OSM map", "Round", "Shikoku transit", "Tohoku Shinkansen graphical timeline", "TOPIX 100", "Update", "West Japan Railway Company Lines".

### Decisions Made
- **Created Renderers & Aliases in Rust**:
  - Implemented `render_co2_template`, `render_fukuoka_stock_exchange_template`, and `render_round_template` in [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
  - Registered `"co2"`, `"fukuoka stock exchange"`, `"round"`, and `"jrssn"` (aliasing `render_jrksn_template`) in `get_dispatch_table()` in [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
  - Registered `"cite periodical"` (aliasing `render_cite_journal_template`) in [src/templates/citation.rs](file:///opt/src/templates/citation.rs).
  - Registered `"hair space"` (mapping to `\u{200a}`) in `get_fixed()` in [src/templates/mod.rs](file:///opt/src/templates/mod.rs).
- **Classified Skipped Templates**:
  - Appended 3 templates to [src/silent.csv](file:///opt/src/silent.csv): "Primary sources", "Rapid transit OSM map", "Update".
  - Appended 14 templates to [src/navigations.csv](file:///opt/src/navigations.csv): "East Japan Railway Company Lines", "Fukuoka transit", "HakoneFujiIzuTransit", "Hokuriku Main Line RDT", "Hokuriku Main Line (undivided)", "Hokuriku_Main_Line_(undivided)", "Kyushu Railway Company Lines", "Nagoya transit", "Okayama transit", "Osaka transit", "Shikoku transit", "Tohoku Shinkansen graphical timeline", "TOPIX 100", "West Japan Railway Company Lines".
  - Sorted both CSV files using `./tools/sort.sh`.
- **Added Unit Tests**:
  - Wrote 22 separate unit tests in [src/tests.rs](file:///opt/src/tests.rs) (one for each template) to verify exact behavior.
- **Updated Documentation**:
  - Documented new template conversion rules in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
- [src/templates/citation.rs](file:///opt/src/templates/citation.rs) [MODIFY]
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
- [src/navigations.csv](file:///opt/src/navigations.csv) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and formatting: `cargo fmt`, `cargo check`, and `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified unit and integration tests: `cargo test` (all passed successfully).
- Verified ignored tests: `cargo test --locked -- --ignored` (all passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-23 Handle Third Batch of Wikipedia Templates

### Summary
Added support for 28 additional Wikipedia templates: "MacTutor", "main category", "Math proof", "Math theorem", "Misuse of statistics", "no footnotes", "notelist-lr", "NumBlk", "open-closed", "open-open", "overline", "Peter Gustav Lejeune Dirichlet", "PlanetMath", "Probability distribution", "Public health", "Qualitative forecasting methods", "Quantitative forecasting methods", "request quotation", "satellite navigation systems", "sfnmp", "Social surveys", "SQL sidebar", "Start date and age", "Statistics topics sidebar", "Unfocused", "verification needed", "Why?", "Wikifunctions".

### Decisions Made
- **Created Renderers & Aliases in Rust**:
  - Implemented `render_mactutor_template` and `render_planetmath_template` in [src/templates/citation.rs](file:///opt/src/templates/citation.rs).
  - Registered `"mactutor"` and `"planetmath"` in the dispatch table in `src/templates/citation.rs`.
  - Implemented `render_math_proof_template`, `render_math_theorem_template`, `render_numblk_template`, `render_open_closed_template`, `render_open_open_template`, and `render_start_date_and_age_template` in [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
  - Registered `"math proof"`, `"math theorem"`, `"numblk"`, `"open-closed"`, `"open-open"`, `"overline"` (as `render_passthrough_template` alias), and `"start date and age"` in the dispatch table in `src/templates/formatting.rs`.
- **Classified Skipped Templates**:
  - Appended 9 templates to [src/silent.csv](file:///opt/src/silent.csv): "main category", "no footnotes", "notelist-lr", "request quotation", "sfnmp", "Unfocused", "verification needed", "Why?", "Wikifunctions".
  - Appended 10 templates to [src/navigations.csv](file:///opt/src/navigations.csv): "Misuse of statistics", "Peter Gustav Lejeune Dirichlet", "Probability distribution", "Public health", "Qualitative forecasting methods", "Quantitative forecasting methods", "satellite navigation systems", "Social surveys", "SQL sidebar", "Statistics topics sidebar".
  - Sorted both CSV files using `./tools/sort.sh`.
- **Added Unit Tests**:
  - Wrote 28 separate unit tests in [src/tests.rs](file:///opt/src/tests.rs) (one for each template) to verify exact behavior.
- **Updated Documentation**:
  - Documented new template conversion rules in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/citation.rs](file:///opt/src/templates/citation.rs) [MODIFY]
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
- [src/navigations.csv](file:///opt/src/navigations.csv) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and formatting: `cargo check`, `cargo fmt`, and `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified unit and integration tests: `cargo test` (all passed successfully).
- Verified ignored tests: `cargo test --locked -- --ignored` (all passed successfully).

### Pending Follow-Ups
- None.


## 2026-06-23 Handle Second Batch of Wikipedia Templates

### Summary
Added support for 34 additional Wikipedia templates by registering them in template databases (silent/navigations), adding aliases, or implementing rendering logic: "1/2", "Abramowitz Stegun ref", "Artificial intelligence navbox", "Biases", "big", "brace", "broader", "Cite interview", "cite SEP", "closed-closed", "cmn", "col div", "confused", "Confusing", "control theory", "date missing", "EB1911", "efn-lr", "em dash", "Equation box 1", "EquationNote", "Essay", "Explanatory footnote", "Fallacies", "font color", "ghat", "Globalize", "Incidence structures", "i sup", "italics correction", "least squares and regression analysis", "Least_squares_and_regression_analysis", "Machine learning", "Machine learning bar".

### Decisions Made
- **Created Renderers & Aliases in Rust**:
  - Registered fixed template string replacements for `"1/2"` and `"em dash"` in [src/templates/mod.rs](file:///opt/src/templates/mod.rs).
  - Implemented `render_brace_template`, `render_broader_template`, `render_closed_closed_template`, `render_equation_box_1_template`, `render_equation_note_template`, and `render_font_color_template` in [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
  - Registered `"abramowitz stegun ref"` (alias of `render_as_ref_template`), `"big"` (alias of `render_passthrough_template`), `"ghat"` (alias of `render_passthrough_template`), `"i sup"` (alias of `render_isup_template`), and `"italics correction"` (alias of `render_passthrough_template`) in [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
  - Implemented `render_cite_sep_template` in [src/templates/citation.rs](file:///opt/src/templates/citation.rs) to format citations of the Stanford Encyclopedia of Philosophy.
  - Registered `"cite interview"` (alias of `render_citation_template`) and `"cite sep"` in [src/templates/citation.rs](file:///opt/src/templates/citation.rs).
- **Classified Skipped Templates**:
  - Appended 10 templates to [src/silent.csv](file:///opt/src/silent.csv): "cmn", "col div", "confused", "Confusing", "date missing", "EB1911", "efn-lr", "Essay", "Explanatory footnote", "Globalize".
  - Appended 9 templates to [src/navigations.csv](file:///opt/src/navigations.csv): "Artificial intelligence navbox", "Biases", "control theory", "Fallacies", "Incidence structures", "least squares and regression analysis", "Least_squares_and_regression_analysis", "Machine learning", "Machine learning bar".
  - Sorted both CSV files using `./tools/sort.sh`.
- **Added Unit Tests**:
  - Wrote 34 separate unit tests in [src/tests.rs](file:///opt/src/tests.rs) (one for each template) to verify exact behavior.
- **Updated Documentation**:
  - Documented new template conversion rules in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
- [src/templates/citation.rs](file:///opt/src/templates/citation.rs) [MODIFY]
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
- [src/navigations.csv](file:///opt/src/navigations.csv) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and formatting: `cargo check`, `cargo fmt`, and `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified unit and integration tests: `cargo test` (all passed successfully).
- Verified ignored tests: `cargo test --locked -- --ignored` (all passed successfully).

### Pending Follow-Ups
- None.


## 2026-06-23 Handle Additional Wikipedia Templates

### Summary
Added support for 20 new Wikipedia templates by registering them in template databases (silent/navigations) or adding rendering logic: "More footnotes", "technical", "erratum", "Merging from", "Merging to", "main cat", "Copy edit", "cols", "Wikiversity", "SpringerEOM", "nowrap begin", "nowrap end", "Commonscatinline", "Technical inline", "technical analysis", "Experimental design", "Six Sigma Tools", "NIST-PD", "lead rewrite", "AI-generated".

### Decisions Made
- **Created Renderers**:
  - Implemented `render_springereom_template` in [src/templates/citation.rs](file:///opt/src/templates/citation.rs) to format citations of the EMS Encyclopaedia of Mathematics.
  - Implemented `render_erratum_template` in [src/templates/citation.rs](file:///opt/src/templates/citation.rs) to format erratum metadata.
  - Registered both templates in the dispatch table in `src/templates/citation.rs`.
- **Classified Skipped Templates**:
  - Appended 15 templates to [src/silent.csv](file:///opt/src/silent.csv): "AI-generated", "cols", "Commonscatinline", "Copy edit", "lead rewrite", "main cat", "Merging from", "Merging to", "More footnotes", "NIST-PD", "nowrap begin", "nowrap end", "technical", "Technical inline", "Wikiversity".
  - Appended 3 templates to [src/navigations.csv](file:///opt/src/navigations.csv): "technical analysis", "Experimental design", "Six Sigma Tools".
  - Sorted both CSV files using `./tools/sort.sh`.
- **Added Unit Tests**:
  - Wrote 20 separate unit tests in [src/tests.rs](file:///opt/src/tests.rs) (one for each template) to verify exact behavior.
- **Updated Documentation**:
  - Documented `SpringerEOM` and `erratum` conversion rules in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/citation.rs](file:///opt/src/templates/citation.rs) [MODIFY]
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
- [src/navigations.csv](file:///opt/src/navigations.csv) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and formatting: `cargo check`, `cargo fmt`, and `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified unit and integration tests: `cargo test` (all passed successfully).
- Verified ignored tests: `cargo test --locked -- --ignored` (all passed successfully).

### Pending Follow-Ups
- None.


## 2026-06-23 Map -v Short Flag to Version

### Summary
Configured the command line interface to accept `-v` as a short flag alias for `--version`.

### Decisions Made
- **Overrode Default Version Flag in clap**:
  - Disabled the default auto-generated version flag by adding `disable_version_flag = true` to the `#[command(...)]` attribute of [CliArgs](file:///opt/src/config.rs#L91).
  - Explicitly added a `version` field of type `Option<bool>` to `CliArgs` with the `#[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]` attribute. Using `Option<bool>` prevents clap from throwing a required-argument-missing error when the flag is not provided.
- **Added Integration Test**:
  - Added [cli_version_flag_is_accepted_by_binary](file:///opt/tests/other.rs#L258) to [tests/other.rs](file:///opt/tests/other.rs) to verify that both `-v` and `--version` function correctly and output identical version info containing the program name and git SHA.

### Files Changed
- [src/config.rs](file:///opt/src/config.rs) [MODIFY]
- [tests/other.rs](file:///opt/tests/other.rs) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and formatting: `cargo check`, `cargo fmt`, and `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified unit and integration tests: `cargo test` (all passed successfully).
- Verified ignored tests: `cargo test --locked -- --ignored` (all passed successfully).

### Pending Follow-Ups
- None.


## 2026-06-21 Refactor Tables into a Separate Module

### Summary
Refactored wikitext table parsing, extraction, and rendering functions from [src/main.rs](file:///opt/src/main.rs) into a new dedicated module [src/tables.rs](file:///opt/src/tables.rs).

### Decisions Made
- **Created the Tables Module**:
  - Moved `extract_class_attr`, `table_marker_id`, `render_wikitext_tables`, `render_wikitext_tables_with_excluded_links`, `is_wikitable_attrs`, `render_wikitable`, and `extract_cell_content` to [src/tables.rs](file:///opt/src/tables.rs).
- **Exposed Re-exports in main.rs**:
  - Registered `mod tables;` and re-exported `render_wikitext_tables_with_excluded_links`, `table_marker_id`, and `render_wikitext_tables` inside [src/main.rs](file:///opt/src/main.rs) to avoid modifying import statements in tests or other modules.
- **Adjusted Visibility**:
  - Made [cleanup_inline_markup_with_excluded_links](file:///opt/src/main.rs#L1258) in `src/main.rs` package-private (`pub(crate)`) so that it can be invoked by the table renderer in `src/tables.rs`.

### Files Changed
- [src/main.rs](file:///opt/src/main.rs) [MODIFY]
- [src/tables.rs](file:///opt/src/tables.rs) [CREATE]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and formatting: `cargo check`, `cargo fmt`, and `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified unit and integration tests: `cargo test` (all passed successfully).
- Verified ignored tests: `cargo test --locked -- --ignored` (all passed successfully).

### Pending Follow-Ups
- None.


## 2026-06-21 Log Total Elapsed Time at End of Process

### Summary
Added duration tracking to measure and log the total elapsed time of the process run.

### Decisions Made
- **Recorded Start Instant and Logged Elapsed Duration**:
  - Initialized a start timer `std::time::Instant::now()` at the beginning of [try_main](file:///opt/src/main.rs#L88).
  - Logged `elapsed_seconds` as a float using `info!` structured logging at the end of `try_main`, capturing the total execution time of the entire main operation (including `run`).

### Files Changed
- [src/main.rs](file:///opt/src/main.rs) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and formatting: `cargo check`, `cargo fmt`, and `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified unit and integration tests: `cargo test` (all passed successfully).
- Verified ignored tests: `cargo test --locked -- --ignored` (all passed successfully).

### Pending Follow-Ups
- None.


## 2026-06-21 Move repo_root Helper to Shared Module

### Summary
Moved the [repo_root](file:///opt/tests/common/mod.rs#L3) helper function from [tests/books.rs](file:///opt/tests/books.rs) to a new shared module [tests/common/mod.rs](file:///opt/tests/common/mod.rs) to make it reusable across future integration test files.

### Decisions Made
- **Created a Shared Test Helper Module**:
  - Defined the new module [tests/common/mod.rs](file:///opt/tests/common/mod.rs) and moved the `repo_root` helper function there.
- **Imported `repo_root` in Existing Tests**:
  - Declared `mod common;` and explicitly imported `repo_root` in [tests/books.rs](file:///opt/tests/books.rs) without wildcard imports.

### Files Changed
- [tests/books.rs](file:///opt/tests/books.rs) [MODIFY]
- [tests/common/mod.rs](file:///opt/tests/common/mod.rs) [CREATE]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and formatting: `cargo check`, `cargo fmt`, and `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified unit and integration tests: `cargo test` (all passed successfully).
- Verified ignored tests: `cargo test --locked -- --ignored` (all passed successfully).

### Pending Follow-Ups
- None.


## 2026-06-20 Add Unit Tests to balanced_wiki_link_end Function

### Summary
Added unit-tests to the [balanced_wiki_link_end](file:///opt/src/tools.rs#L271) function in [src/tools.rs](file:///opt/src/tools.rs) to cover all scenarios, including simple links, nested links, unclosed links, unopened links, UTF-8 characters, and starting offset behavior.

### Decisions Made
- **Implemented Comprehensive Unit Tests**:
  - Added [balanced_wiki_link_end_finds_simple_link_end](file:///opt/src/tools.rs#L391)
  - Added [balanced_wiki_link_end_finds_outer_nested_link_end](file:///opt/src/tools.rs#L399)
  - Added [balanced_wiki_link_end_returns_none_for_unclosed_link](file:///opt/src/tools.rs#L407)
  - Added [balanced_wiki_link_end_returns_none_for_unopened_link](file:///opt/src/tools.rs#L415)
  - Added [balanced_wiki_link_end_handles_non_ascii](file:///opt/src/tools.rs#L423)
  - Added [balanced_wiki_link_end_uses_requested_start_offset](file:///opt/src/tools.rs#L431)
- **Updated Imports**:
  - Explicitly imported `balanced_wiki_link_end` in the `tests` module in [src/tools.rs](file:///opt/src/tools.rs) without using wildcard imports.

### Files Changed
- [src/tools.rs](file:///opt/src/tools.rs) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and formatting: `cargo check`, `cargo fmt`, and `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified unit and integration tests: `cargo test` (all passed successfully, including new test cases).
- Verified ignored tests: `cargo test --locked -- --ignored` (all passed successfully).

### Pending Follow-Ups
- None.


## 2026-06-18 Fix Ignored Real-API Book Generation Test

### Summary
Fixed the ignored real-API integration test (`generate_example_books_from_real_wikipedia_api`) that failed because of pretty-printed whitespace/newlines in the XHTML `<title>` and `<h1>` tags of the generated chapters.

### Decisions Made
- **Normalized Whitespace in Assertions**:
  - Modified the real-API book test assertion helper [`assert_real_api_generates_book`](file:///opt/tests/books.rs) to collapse and normalize all whitespace/newlines in the generated XHTML chapter contents before asserting the presence of the expected `<title>` and `<h1>` elements. This robustly accommodates formatting and pretty-printing of XHTML files.

### Files Changed
- [tests/books.rs](file:///opt/tests/books.rs) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and formatting: `cargo check`, `cargo fmt -- --check`, and `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Verified ignored tests: `cargo test --locked -- --ignored` (all ignored tests passed successfully).
- Verified standard tests: `cargo test` (all passed successfully).

### Pending Follow-Ups
- None.


## 2026-06-18 Fix XHTML Tag Nesting Violations and Mismatched Tag Warnings

### Summary
Fixed the XHTML generator for book compilation failures in `hangul`, `joseon`, and `Standard_deviation` caused by malformed tag nesting. Resolved the issues by implementing a robust linear quote state machine for bold/italic formatting, and updated all affected expected book integration test fixtures.

### Decisions Made
- **Fixed Formatting Parser**:
  - Replaced lookahead regex-based replacements in [`format_inline_text`](file:///opt/src/main.rs) with a stack-based linear quote state machine.
  - The new state machine tracks active bold/italic states and performs tag transitions to guarantee perfectly nested and valid XHTML (e.g., `<em><strong>...</strong></em>` instead of `<em><strong>...</em></strong>`).
  - Added a smart breakdown of quote counts (`count == 4` maps to two italic toggles; `count == 5` maps to one italic and one bold toggle; etc.) to cleanly support complex adjacent layouts like `''z''''\u03c3''` (rendering as `<em>z</em><em>\u03c3</em>`) and empty template boundaries like `''{{  }}''` (rendering as `<em></em>`).
- **Prevented Quote Merging Bug**:
  - Modified the possessive `'s` template mapping in [`src/templates/mod.rs`](file:///opt/src/templates/mod.rs) to replace literal apostrophes with a safe placeholder `__WIKIPEDIA_TO_EPUB_LITERAL_QUOTE__`, restoring them back to `'` after formatting. This prevents them from merging with surrounding formatting single quotes (e.g. `''Han''{{'s}}` becoming `''Han'''s` and breaking parsing).
- **Regenerated Expected Book Fixtures**:
  - Ran `./tools/regenerate.sh` for all five books (`hangul`, `joseon`, `Standard_deviation`, `han-dynasty`, `korean-language`) to update their expected XHTML fixtures under `expected/` to match the correct well-formed output.

### Files Changed
- [src/main.rs](file:///opt/src/main.rs) [MODIFY]
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
- [expected/Standard_deviation/OEBPS/Standard_deviation.xhtml](file:///opt/expected/Standard_deviation/OEBPS/Standard_deviation.xhtml) [MODIFY]
- [expected/han-dynasty/OEBPS/Han_dynasty.xhtml](file:///opt/expected/han-dynasty/OEBPS/Han_dynasty.xhtml) [MODIFY]
- [expected/hangul/OEBPS/Hangul.xhtml](file:///opt/expected/hangul/OEBPS/Hangul.xhtml) [MODIFY]
- [expected/joseon/OEBPS/Joseon.xhtml](file:///opt/expected/joseon/OEBPS/Joseon.xhtml) [MODIFY]
- [expected/korean-language/OEBPS/Korean_language.xhtml](file:///opt/expected/korean-language/OEBPS/Korean_language.xhtml) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and warnings: `cargo check` and `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Checked code formatting: `cargo fmt -- --check` (passed cleanly).
- Ran all tests: `cargo test` (all 424 unit/doc tests and 41 integration tests passed successfully).

### Pending Follow-Ups
- None.


## 2026-06-18 Handle lit., glossary, glossary end, fcn, dynamic list, SMS, SS, sronly, Expand language, Free-content attribution, pp, pp-sock, missing long citation, subscription, ambiguous, Bare URL PDF, better source, and More Templates

### Summary
Implemented, tested, and documented rendering handlers for 18 Wikipedia templates: `"lit."`, `"glossary"`, `"glossary end"`, `"fcn"`, `"dynamic list"`, `"SMS"`, `"SS"`, `"sronly"`, `"Expand language"`, `"Free-content attribution"`, `"pp"`, `"pp-sock"`, `"missing long citation"`, `"subscription"`, `"ambiguous"`, `"Bare URL PDF"`, `"better source"`, and `"More"`. Added them to the dispatch tables/CSVs, wrote separate unit tests, ran the sorting tool, and documented the rules in `DEVELOPMENT.md`.

### Decisions Made
- **Implemented / Registered Renderers**:
  - `lit.`: Registered case-insensitively to map to `render_literal_template` in `src/templates/lang.rs`.
  - `glossary` & `glossary end`: Handled as silent/omitted block structures, returning `String::new()` in `src/templates/formatting.rs` to allow child term/defn tags to compile cleanly.
  - `SMS` & `SS`: Dispatched to ship templates (HMS/USS equivalent) in `src/templates/mod.rs`.
  - `sronly`: Passes through the visible parameter text.
  - `Free-content attribution`: Parsed named parameters and formatted using CS1 free license attribution style.
  - `More`: Handled as an alias for the `Further` template.
- **Omitted Silent Templates**:
  - Registered `fcn`, `dynamic list`, `Expand language`, `pp`, `pp-sock`, `missing long citation`, `subscription`, `ambiguous`, `Bare URL PDF`, and `better source` into `src/silent.csv` to be skipped silently.
- **Added Unit Tests**:
  - Added 18 unit tests in `src/tests.rs` verifying each template case individually.
- **Sorted CSV Files**:
  - Sorted `src/silent.csv` using `./tools/sort.sh`.
- **Updated Documentation**:
  - Added new rules and updated list of omitted/formatting templates in `DEVELOPMENT.md`.

### Files Changed
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
- [src/templates/lang.rs](file:///opt/src/templates/lang.rs) [MODIFY]
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
- [src/templates/citation.rs](file:///opt/src/templates/citation.rs) [MODIFY]
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and formatting: `cargo check`, `cargo fmt -- --check`, and `cargo clippy --all-targets -- -D warnings` (all passed cleanly).
- Verified unit and integration tests: `cargo test` (all 424 unit/doc tests and 41 integration tests passed successfully).
- Ran `./tools/sort.sh` to sort CSV databases.

### Pending Follow-Ups
- None.


## 2026-06-18 Handle tyo, nag, tba, dagger, yen, stl, rcb, vertical header, sticky-header, jarc, JPYConvert, Inlang, and JRKSN Templates

### Summary
Implemented, tested, and documented rendering handlers for 13 Wikipedia templates: `"tyo"`, `"nag"`, `"tba"`, `"dagger"`, `"yen"`, `"stl"`, `"rcb"`, `"vertical header"`, `"sticky-header"`, `"jarc"`, `"JPYConvert"`, `"Inlang"`, and `"JRKSN"`. Added them to the dispatch tables, wrote separate unit tests, sorted the silent templates CSV file, and documented conversion rules in `DEVELOPMENT.md`.

### Decisions Made
- **Implemented / Registered Renderers**:
  - `tyo` (`render_tyo_template`) & `nag` (`render_nag_template`): Format stock exchange ticker codes for Tokyo and Nagoya.
  - `tba` & `dagger`: Mapped as fixed text replacements (`TBA` and `†`) in `get_fixed()`.
  - `yen` & `¥`: Mapped to the existing `render_jpy_template`.
  - `stl` (`render_stl_template`): Generates standard railway station links using parameter 2 (station).
  - `rcb` (`render_rcb_template`): Generates railway line links using parameter 2 (line).
  - `vertical header` (`render_vertical_header_template`): Wraps parameter and returns content to display horizontally.
  - `sticky-header`: Omitted from output by adding to `src/silent.csv`.
  - `jarc` & `Inlang`: Registered in the dispatch table, delegating to `render_ja_rail_color_template` and `render_in_lang_template` respectively.
  - `JPYConvert` (`render_jpy_convert_template`): Formats Yen values and converts them to US Dollars at a fixed 110.0 rate.
  - `JRKSN` (`render_jrksn_template`): Concatenates railway line code and station number.
- **Added Unit Tests**:
  - Added 13 separate test cases in `src/tests.rs` verifying each template separately.
- **Sorted CSV Files**:
  - Added `sticky-header` to `src/silent.csv` and ran `./tools/sort.sh` to keep CSVs sorted.
- **Updated Documentation**:
  - Appended conversion rules for the templates in `DEVELOPMENT.md`.

### Files Changed
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
- [src/templates/convert.rs](file:///opt/src/templates/convert.rs) [MODIFY]
- [src/templates/lang.rs](file:///opt/src/templates/lang.rs) [MODIFY]
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Verified formatting, compilation, lints, and tests: `cargo fmt`, `cargo check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` (All passed successfully).
- Ran `./tools/sort.sh` to sort CSV files.

### Pending Follow-Ups
- None.


## 2026-06-18 Handle sisterlinks, frac2, vanchor, block indent, dfni, radic, diagonal split header, pipe, legend-line, empty section, prime, isup, fv, family name hatnote, clear right, cjkv, udl, Bare URL inline, wikiquote, and Wikinews Templates

### Summary
Verified, refined, and updated handling for the requested 21 Wikipedia templates ("sisterlinks", "frac2", "vanchor", "block indent", "dfni", "radic", "diagonal split header", "pipe", "legend-line", "empty section", "prime", "isup", "fv", "family name hatnote", "clear right", "cjkv", "udl", "Bare URL inline", "wikiquote", "Wikinews") in the converter. Checked parameter fallback support for `vanchor`, `legend-line`, and `isup` to correctly parse named parameter variants. Extended unit tests to cover named parameter variants.

### Decisions Made
- **Refined Template Renderers**:
  - `vanchor` (`render_visible_anchor_template`): Correctly supported fallback to named parameter `1` if positional parameter is missing, and used only `text` or first parameter for visible text.
  - `legend-line` (`render_legend_line_template`): Supported named parameter `2` and fallback.
  - `isup` (`render_isup_template`): Supported named parameters `1`, `2` and fallbacks.
  - Silent/omitted templates: Verified they are all listed in `src/silent.csv`.
- **Wrote Unit Tests**:
  - Expanded test cases in [src/tests.rs](file:///opt/src/tests.rs) to test named parameters for `vanchor`, `legend-line`, and `isup`.
- **Documented conversion rules**:
  - Verified they are all documented in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and formatting: `cargo fmt`, `cargo check`, and `cargo clippy --all-targets -- -D warnings` (Passed cleanly, no warnings/errors).
- Verified unit and integration tests: `cargo test` (All passed successfully).

### Pending Follow-Ups
- None.


## 2026-06-18 Handle cite video, Cite tweet, cite constitution, cite bioRxiv, Harvard citation text, Cite MW, term, defn, cquote, London Gazette, and US$ Templates

### Summary
Implemented Wikipedia template renderers and aliases for `"cite video"`, `"Cite tweet"`, `"cite constitution"`, `"cite bioRxiv"`, `"Harvard citation text"`, `"Cite MW"`, `"term"`, `"defn"`, `"cquote"`, `"London Gazette"`, and `"US$"`. Wrote unit tests for each new template separately and documented the conversion rules.

### Decisions Made
- **Implemented template renderers**:
  - `cite video`: Mapped to the existing `render_cite_av_media_template` renderer.
  - `Cite tweet`: Formats tweets inline with author/holder name, user handle, text, tweet URL (constructed via user + status number or explicit link), date, and access date.
  - `cite constitution`: Formats citations of national constitutions.
  - `cite bioRxiv`: Formats bioRxiv preprints with authorship, title, year, and DOI-based link.
  - `Harvard citation text`: Mapped to the existing `render_harvtxt_template` renderer.
  - `Cite MW`: Mapped to the existing `render_cite_merriam_webster_template` renderer.
  - `term` and `defn`: Formats definition lists (term name bolded, and definition text passed through).
  - `cquote`: Formats centered blockquotes with quote text, author, and source.
  - `London Gazette`: Formats issue, page, date, and supplement status linked to the Gazette archives.
  - `US$`: Formats US dollar currency values.
- **Registered templates**:
  - Added case-insensitive matching for all 11 templates in `is_handled_template_name` within [src/templates/mod.rs](file:///opt/src/templates/mod.rs).
  - Mapped handlers in [src/templates/citation.rs](file:///opt/src/templates/citation.rs) and [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
- **Added unit tests**:
  - Added separate unit tests in [src/tests.rs](file:///opt/src/tests.rs) for each of the new templates.
- **Documented conversion rules**:
  - Appended documentation and example rules for each template in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
- [src/templates/citation.rs](file:///opt/src/templates/citation.rs) [MODIFY]
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and warnings: `cargo check --all-targets` and `cargo clippy --all-targets -- -D warnings` (Passed cleanly, no warnings/errors).
- Formatting check: `cargo fmt --check` (Passed cleanly).
- Verified unit and integration tests: `cargo test` (All 381 unit/doc tests and 42 integration tests passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-18 Validate Section Chapter Titles for Duplication

### Summary
Enhanced the YAML configuration validation to verify that section chapter titles are also unique and do not conflict with other article names or section titles, avoiding late-stage zip duplicate filename errors. Added 2 new unit tests to cover duplicate section titles and section title colliding with article names.

### Decisions Made
- **Modified Uniqueness Validation**:
  - Updated `collect_duplicate_articles` in [src/config.rs](file:///opt/src/config.rs) to unconditionally record the title of all detailed articles, including those of type `Section`.
- **Wrote Unit Tests**:
  - Added `read_config_rejects_duplicate_section_title_and_article` in [src/tests.rs](file:///opt/src/tests.rs) to verify that the config parser rejects configurations where a section title matches an article name.
  - Added `read_config_rejects_duplicate_section_titles` in [src/tests.rs](file:///opt/src/tests.rs) to verify that the config parser rejects configurations where two section titles are identical.

### Files Changed
- [src/config.rs](file:///opt/src/config.rs) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and warnings: `cargo check --all-targets` and `cargo clippy --all-targets -- -D warnings` (Passed cleanly, no warnings/errors).
- Formatting check: `cargo fmt --check` (Passed cleanly).
- Verified unit and integration tests: `cargo test` (All 371 unit/doc tests and 42 integration tests passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-17 Handle templates right, Cite peakbagger, wikibooks inline, refh, By whom, M

### Summary
Implemented Wikipedia template renderers and aliases for `"right"`, `"Cite peakbagger"`, `"wikibooks inline"`, `"refh"`, `"M"` templates, and added `"By whom"` as a silently skipped template. Wrote comprehensive unit tests and documented conversion rules.

### Decisions Made
- **Implemented template renderers**:
  - `render_right_template`: Renders `style="text-align:right"|` if no arguments, or wraps content in a right-aligned div.
  - `render_cite_peakbagger_template`: Renders a citation link to `peakbagger.com` supporting various ID params (`pid`, `lid`, `rid`, `kid`).
  - `render_wikibooks_inline_template`: Renders a sister project link inline to Wikibooks.
  - `render_refh_template`: Renders reference column headings for tables (standardizing to "Refs." or "Ref.").
  - `render_m_template`: Mapped to `"M"` (and aliases `"m"`, `"earthquake magnitude"`), formatting various earthquake magnitude scales with correct subscript abbreviation and anchor links.
- **Registered silent templates**:
  - Added `"By whom"` to [src/silent.csv](file:///opt/src/silent.csv).
- **Wrote unit tests**:
  - Added unit tests in [src/tests.rs](file:///opt/src/tests.rs) for each template individually.
- **Documented rules**:
  - Added conversion rules in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
- [src/templates/citation.rs](file:///opt/src/templates/citation.rs) [MODIFY]
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation, lints and formatting: `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` (Passed cleanly, no warnings/errors).
- Verified unit and integration tests: `cargo test` (All 369 unit/doc tests and 42 integration tests passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-17 Handle Matsumoto Airport Templates

### Summary
Implemented and refined the Wikipedia template rendering handlers for `"airport codes"`, `"Airport-dest-list"`, and `"NWS-current"` to compile the Matsumoto Airport book.

### Decisions Made
- **Implemented Template Renderers**:
  - `render_airport_codes_template`: Maps up to 6 airport code standards (IATA, ICAO, FAA, TC, GPS, CAAC), joining them with commas and wrapping in parentheses (unless parameter `p=n` is specified).
  - `render_airport_dest_list_template`: Formats positional airline-destination pairs as a wikitext table (`{| class="wikitable"`) so it is parsed and generated correctly as an XHTML table by the compiler instead of being stripped.
  - `render_nws_current_template`: Formats a weather link to NOAA/NWS current weather for a given ICAO code.
- **Added/Modified Tests**:
  - Added unit tests for each of the three templates in [src/tests.rs](file:///opt/src/tests.rs).
  - Regenerated integration test expected book fixtures under `expected/Matsumoto_Airport` using the `./tools/regenerate.sh` script.
- **Updated Documentation**:
  - Added documentation rules for the templates in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- [expected/Matsumoto_Airport/OEBPS/Matsumoto_Airport.xhtml](file:///opt/expected/Matsumoto_Airport/OEBPS/Matsumoto_Airport.xhtml) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and formatting: `cargo fmt`, `cargo check --all-targets`, and `cargo clippy --all-targets -- -D warnings` (Passed cleanly).
- Verified unit and integration tests: `cargo test` (All 367 unit/doc tests and 41 integration tests passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-17 Fast Fail on Missing Front Matter Files

### Summary
Implemented a check right at the beginning of the compilation process (`run` function) to verify that all front matter files defined in the book configuration exist. Stopped processing immediately by returning an error if any file is missing, avoiding unnecessary Wikipedia API calls or processing.

### Decisions Made
- **Implemented Fast Fail Check**:
  - Inserted a file check for each file in `config.front_mater` right at the start of `run()` inside [src/main.rs](file:///opt/src/main.rs).
- **Added Integration Test**:
  - Added `generate_book_fails_if_front_matter_file_is_missing` in [tests/books.rs](file:///opt/tests/books.rs) to verify that compilation fails immediately and returns the expected error message when a front matter file is missing.

### Files Changed
- [src/main.rs](file:///opt/src/main.rs) [MODIFY]
- [tests/books.rs](file:///opt/tests/books.rs) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and formatting: `cargo check`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` (Passed cleanly).
- Verified unit and integration tests: `cargo test` (All 360 unit tests and 40 integration tests passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-17 Skip Stub Templates Automatically

### Summary
Implemented automatic skipping of all wikitext templates ending with the word "stub" case-insensitively. Removed explicit stub entries from `src/navigations.csv`, and updated the skipped templates page with a note indicating this automatic behavior.

### Decisions Made
- **Implemented Auto-Skipping in Compiler**:
  - Added a case-insensitive check in `is_silent_template_name` inside [src/templates/mod.rs](file:///opt/src/templates/mod.rs) to skip any template ending with `"stub"`.
- **Wrote Unit Test**:
  - Added `render_wikitext_silently_skips_stub_templates` in [src/tests.rs](file:///opt/src/tests.rs) to verify correct skipping of stub templates under various casing and names.
- **Removed Stub Entries from CSV**:
  - Removed all explicit stub template entries from [src/navigations.csv](file:///opt/src/navigations.csv) and sorted the file using `./tools/sort.sh`.
- **Updated Website Template**:
  - Modified [templates/site/skipped-templates.html.j2](file:///opt/templates/site/skipped-templates.html.j2) to add a line explaining that stub templates are skipped automatically.

### Files Changed
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [src/navigations.csv](file:///opt/src/navigations.csv) [MODIFY]
- [templates/site/skipped-templates.html.j2](file:///opt/templates/site/skipped-templates.html.j2) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and formatting: `cargo check`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` (Passed cleanly).
- Verified unit and integration tests: `cargo test` (All 360 unit tests, 39 integration tests, and 4 doc-tests passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-17 Include Skipped Navigation Templates on Website

### Summary
Updated the website generator script and the "Skipped Templates" page to also include all skipped navigation templates listed in `src/navigations.csv` in a separate table, linking each navigation template directly to its corresponding Wikipedia page.

### Decisions Made
- **Updated `generate_site.py`**:
  - Implemented `get_navigation_templates()` in [tools/generate_site.py](file:///opt/tools/generate_site.py) to read [src/navigations.csv](file:///opt/src/navigations.csv), construct valid URL links to their corresponding Wikipedia template pages by substituting spaces with underscores and quoting special characters, and pass the list to the template context.
- **Updated Jinja2 template**:
  - Modified [templates/site/skipped-templates.html.j2](file:///opt/templates/site/skipped-templates.html.j2) to add a new "Navigation Templates" section containing a table of these templates. Updated the total count badge logic to aggregate both silent templates and navigation templates.

### Files Changed
- [tools/generate_site.py](file:///opt/tools/generate_site.py) [MODIFY]
- [templates/site/skipped-templates.html.j2](file:///opt/templates/site/skipped-templates.html.j2) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Checked compilation and formatting: `cargo check`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` (Passed cleanly).
- Verified unit and integration tests: `cargo test` (All passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-17 Implement Skipped Templates Website Page

### Summary
Modified the website generator script to parse `src/silent.csv` and generate a new page "Skipped Templates" listing all skipped templates with links (where available) and comments. Added a link to the new "Skipped Templates" page in the footer of the homepage.

### Decisions Made
- **Created a new Jinja2 Template**:
  - Added [templates/site/skipped-templates.html.j2](file:///opt/templates/site/skipped-templates.html.j2) with support for listing, linking, and dynamically filtering/searching skipped templates.
- **Updated `generate_site.py`**:
  - Modified [tools/generate_site.py](file:///opt/tools/generate_site.py) to read [src/silent.csv](file:///opt/src/silent.csv) using the Python `csv` module, parse it into structured dictionary list, and render both `index.html` and `skipped-templates.html`.
- **Linked homepage**:
  - Updated [templates/site/index.html.j2](file:///opt/templates/site/index.html.j2) to add a link to the "Skipped Templates" page in the site footer as requested by the user.

### Files Changed
- [tools/generate_site.py](file:///opt/tools/generate_site.py) [MODIFY]
- [templates/site/skipped-templates.html.j2](file:///opt/templates/site/skipped-templates.html.j2) [CREATE]
- [templates/site/index.html.j2](file:///opt/templates/site/index.html.j2) [MODIFY]
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run
- Verified formatting, compilation, and lints: `cargo fmt --check`, `cargo check --all-targets`, and `cargo clippy --all-targets -- -D warnings` (All passed cleanly).
- Ran cargo tests: `cargo test` (All 359 unit tests, 39 integration tests, and 4 doc-tests passed).

### Pending Follow-Ups
- None.

## 2026-06-16 Handle nobr, which?, Redirect-distinguish, Collapse top, Collapse bottom, var, gaps, and example needed Templates

### Summary
Implemented and registered template handlers for `nobr`, `which?`, `Redirect-distinguish`, `Collapse top`, `Collapse bottom`, `var`, `gaps`, and `example needed` templates for the `Standard deviation` page/book config. Updated conversion logic for inches normalization and composite `ftin` target unit inside the linear unit converter. Wrote unit tests, updated expected book fixtures, documented rules, and resolved clippy warnings.

### Decisions Made
- **Implemented template renderers**:
  - `Collapse bottom`: Mapped to `""` in `get_fixed` in [src/templates/mod.rs](file:///opt/src/templates/mod.rs).
  - `Collapse top`: Renders bold collapsible header blocks.
  - `var`: Formats variables inside `<var>` tags (utilizing temporary formatting placeholders that are restored back in [src/main.rs](file:///opt/src/main.rs)).
  - `gaps`: Formats numbers with digit groupings, optional scientific base/exponent notation, and optional unit/LHS values.
  - `nobr`: Handled as an alias of the existing `render_passthrough_template`.
  - `which?`, `Redirect-distinguish`, and `example needed`: Added to [src/silent.csv](file:///opt/src/silent.csv) as silent templates.
- **Improved convert unit support**:
  - Supported converting to composite `"ftin"` unit and normalizing source unit keys for `"inches"`/ `"inch"` to `"in"`.
- **Added unit tests**:
  - Wrote individual unit tests in [src/tests.rs](file:///opt/src/tests.rs) for all 8 templates.
- **Regenerated expected fixtures**:
  - Updated `Standard_deviation` integration book expected output using `./tools/regenerate.sh`.
- **Documented conversion rules**:
  - Added rules to [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/main.rs](file:///opt/src/main.rs) [MODIFY]
  - Restored `<var>` spans.
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
  - Registered templates case-insensitively in name filter.
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
  - Implemented `Collapse top`, `var`, and `gaps` renderers.
- [src/templates/convert.rs](file:///opt/src/templates/convert.rs) [MODIFY]
  - Added support for `"inches"` and composite `"ftin"`.
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
  - Added 8 separate unit tests.
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
  - Added silent templates and sorted.
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
  - Documented rules and examples.
- `expected/Standard_deviation/OEBPS/Standard_deviation.xhtml` [MODIFY]
  - Regenerated integration book expected output.
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]
  - Appended current session note.

### Tests Run
- Checked compilation and formatting: `cargo check`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` (Passed cleanly, no warnings/errors).
- Verified unit and integration tests: `cargo test` (All 359 unit tests, 39 integration tests, and 4 doc-tests passed).

### Pending Follow-Ups
- None.

## 2026-06-16 Handle Cite Templates: cite paper, cite court, Cite Dictionary.com, Cite speech, cite SSRN, cite tech report, and Cite CiteSeerX

### Summary
Implemented and registered handlers for `cite court`, `Cite Dictionary.com`, `Cite speech`, `cite SSRN`, and `Cite CiteSeerX` templates. Mapped `cite paper` to reuse the existing `cite journal` renderer, and mapped `cite tech report` to reuse the existing `cite report` renderer. Wrote separate unit tests for all seven templates and verified everything passes.

### Decisions Made
- **Implemented template renderers**:
  - `cite paper`: mapped to reuse `render_cite_journal_template` since it shares identical fields.
  - `cite court`: formats legal case citations in Bluebook style, using `official-url` wrapper for litigants when `url` is present.
  - `Cite Dictionary.com`: formats Dictionary.com definition lookups.
  - `Cite speech`: formats CS1-style speech and lecture citations.
  - `cite SSRN`: formats SSRN preprint citations with correct abstract query links.
  - `cite tech report`: mapped to reuse `render_cite_report_template` since it functions identically.
  - `Cite CiteSeerX`: formats CS1-style CiteSeerX digital library search citations.
- **Registered templates**:
  - Registered all new templates inside `is_handled_template_name` in [src/templates/mod.rs](file:///opt/src/templates/mod.rs).
  - Added mappings inside `get_dispatch_table` in [src/templates/citation.rs](file:///opt/src/templates/citation.rs).
- **Added unit tests**:
  - Wrote individual unit tests in [src/tests.rs](file:///opt/src/tests.rs) for all seven templates.
- **Documented conversion rules**:
  - Added conversion rules in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/citation.rs](file:///opt/src/templates/citation.rs) [MODIFY]
  - Appended the 5 new renderers and updated the dispatch table.
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
  - Registered the 7 templates in `is_handled_template_name`.
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
  - Added 7 separate unit tests.
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
  - Documented new template conversion rules.
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]
  - Updated with current session notes.

### Tests Run
- Checked compilation and formatting: `cargo check`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` (Passed cleanly, no warnings/errors).
- Verified unit and integration tests: `cargo test` (All 351 unit tests, 38 integration tests, and 4 doc-tests passed).

### Pending Follow-Ups
- None.

## 2026-06-16 Handle MathWorld, AS ref, Probability fundamentals, OEIS2C, thinsp, dfn, subsup, abs, mono, pi, divcol, Springer, divcol end, and ProbDistributions

### Summary
Implemented and registered handlers for `MathWorld`, `AS ref`, `OEIS2C`, `thinsp`, `dfn`, `subsup`, `abs`, `mono`, `pi`, and `Springer` templates. Mapped the navigation templates `Probability fundamentals`, `ProbDistributions` to `navigations.csv`, and formatting templates `divcol`, `divcol end` to `silent.csv`. Handled post-processing tag stripping issues for `<dfn>` and `<code>` by using custom placeholders, and handled absolute value wikitext table detection edge cases. Updated tests and documentation, and regenerated the `Normal_distribution` expected book output.

### Decisions Made
- **Implemented template renderers**:
  - `MathWorld`: formats MathWorld citations.
  - `AS ref`: formats reference citations to Abramowitz and Stegun.
  - `OEIS2C`: formats OEIS sequence link identifiers.
  - `thinsp`: joins parameters with a thin space (`\u{2009}`), using a template-specific placeholder `__WIKIPEDIA_TO_EPUB_THINSP_TEMPLATE__` to avoid side-effects on other files.
  - `dfn`: wraps definitions in custom placeholders (`__WIKIPEDIA_TO_EPUB_DFN_START__` / `__WIKIPEDIA_TO_EPUB_DFN_END__`) that map back to `<dfn>` tags at the end of formatting.
  - `subsup`: formats subscript and superscript alignments.
  - `abs`: wraps parameter inside vertical bars (`&#124;content&#124;`), avoiding line-start wikitext table marker check.
  - `mono`: wraps monospace code in custom placeholders (`__WIKIPEDIA_TO_EPUB_CODE_START__` / `__WIKIPEDIA_TO_EPUB_CODE_END__`) mapping to `<code>` tags.
  - `pi`: formats Greek letter pi (`π`).
  - `Springer`: formats Springer Encyclopedia of Mathematics citations.
- **Added silent and navigation templates**:
  - Added `Probability fundamentals` and `ProbDistributions` to `navigations.csv`.
  - Added `divcol` and `divcol end` to `silent.csv`.
  - Sorted the CSV files using `./tools/sort.sh`.
- **Added and updated tests**:
  - Wrote separate unit tests for all fourteen templates.
  - Fixed test assertions for `MathWorld`, `AS ref`, and `Springer` to handle link rendering correctly in test environments.
- **Regenerated expected fixtures**:
  - Updated the `Normal_distribution` book expected output using `./tools/regenerate.sh`.
- **Documented conversion rules**:
  - Added conversion rules in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
  - Implemented the ten new render functions and registered them in the dispatch table.
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
  - Registered all new handled template names in `is_handled_template_name`.
- [src/main.rs](file:///opt/src/main.rs) [MODIFY]
  - Handled the replacement and mapping of custom placeholders for `<dfn>`, `<code>`, and `\u{2009}`.
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
  - Added unit tests for each new template and adjusted assertions.
- [src/navigations.csv](file:///opt/src/navigations.csv) [MODIFY]
  - Added `Probability fundamentals` and `ProbDistributions`.
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
  - Added `divcol` and `divcol end`.
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
  - Documented rules and examples for the new templates.
- `expected/Normal_distribution/OEBPS/*` [MODIFY]
  - Regenerated Normal_distribution book output.
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]
  - Appended this session note.

### Tests Run
- Checked compilation and formatting: `cargo check`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` (Passed cleanly, no warnings/errors).
- Verified unit and integration tests: `cargo test` (All 344 unit tests, 38 integration tests, and 4 doc-tests passed).

### Pending Follow-Ups
- None.

## 2026-06-16 Handle JSTOR, wsPSM, StatsTopicTOC, Math topics TOC, em, Areas of mathematics, and Glossaries of science and engineering

### Summary
Implemented and registered handlers for `JSTOR`, `wsPSM`, and `em` templates. Mapped the navigation templates `StatsTopicTOC`, `Math topics TOC`, `Areas of mathematics`, and `Glossaries of science and engineering` to be silently skipped. Wrote unit tests, updated rules, regenerated book output, and sorted CSV files.

### Decisions Made
- **Implemented template renderers**:
  - `JSTOR`: formats JSTOR citation identifiers (e.g. `JSTOR 1400906`).
  - `wsPSM`: formats Wikisource Popular Science Monthly article citation links.
  - `em`: wraps text in emphasis (italic) tags `''text''` while safeguarding potential `=` inside the argument.
- **Added silent templates**:
  - Added `StatsTopicTOC`, `Math topics TOC`, `Areas of mathematics`, and `Glossaries of science and engineering` to [src/navigations.csv](file:///opt/src/navigations.csv) to silently omit these navigation sidebars and footer boxes from the EPUB output.
  - Sorted the CSV files using `./tools/sort.sh`.
- **Registered templates**:
  - Registered `jstor`, `wspsm`, and `em` in `get_dispatch_table` of [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
  - Added `JSTOR`, `wsPSM`, and `em` case-insensitively to `is_handled_template_name` in [src/templates/mod.rs](file:///opt/src/templates/mod.rs).
- **Added unit tests**:
  - Wrote separate unit tests for `JSTOR`, `wsPSM`, `em`, `StatsTopicTOC`, `Math topics TOC`, `Areas of mathematics`, and `Glossaries of science and engineering` in [src/tests.rs](file:///opt/src/tests.rs).
- **Regenerated expected fixtures**:
  - Updated the `Statistics` book expected output using `./tools/regenerate.sh`.
- **Documented conversion rules**:
  - Added conversion rules in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
  - Implemented `render_jstor_template`, `render_wspsm_template`, and `render_em_template` and registered them in the dispatch table.
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
  - Registered `JSTOR`, `wsPSM`, and `em` in `is_handled_template_name`.
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
  - Added separate unit tests for the seven templates.
- [src/navigations.csv](file:///opt/src/navigations.csv) [MODIFY]
  - Appended and sorted navigation templates.
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
  - Documented new template conversion rules.
- `expected/Statistics/OEBPS/*` [MODIFY]
  - Regenerated Statistics book expected output.
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]
  - Updated with current session notes.

### Tests Run
- Checked compilation: `cargo check` (passed).
- Checked formatting: `cargo fmt -- --check` (passed).
- Checked lints: `cargo clippy --all-targets -- -D warnings` (passed cleanly).
- Ran all tests: `cargo test` (all 331 unit tests, 37 integration tests, and 4 doctests passed).

### Pending Follow-Ups
- None.

## 2026-06-16 Handle tmath, closed-open, sqrt, Section link, mset, hidden begin, and hidden end

### Summary
Implemented and registered handlers for `tmath`, `closed-open`, `sqrt`, `Section link` / `section link` / `slink`, `mset`, and `hidden begin` / `hidden end` templates. Added case-insensitive registry check support, added unit tests, regenerated book outputs, and sorted CSV files.

### Decisions Made
- **Implemented template renderers**:
  - `tmath`: mathematical expression maps to raw math content in text.
  - `closed-open`: mathematical interval notation `[a, b)`.
  - `sqrt`: wraps content with square root symbol `√{content}`.
  - `Section link` / `section link`: mapped to standard wikitext links `[[Target#Section|Target § Section]]` (using the correct `§ ` section sign prefix).
  - `mset`: formats set values wrapped in curly braces `{val1, val2}`.
  - `hidden begin` / `hidden end`: collapses block by rendering bold header title and inline content, omitting the `hidden end` wrapper.
- **Registered templates**:
  - Registered all template renderers in `get_dispatch_table` within [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
  - Added all new templates case-insensitively to `is_handled_template_name` in [src/templates/mod.rs](file:///opt/src/templates/mod.rs).
  - Registered `"hidden end"` mapped to `""` in `get_fixed` in [src/templates/mod.rs](file:///opt/src/templates/mod.rs).
- **Added unit tests**:
  - Wrote individual unit tests for all new templates in [src/tests.rs](file:///opt/src/tests.rs).
  - Updated existing `slink` test cases to align with correct section sign (`§ `) rendering.
- **Regenerated expected fixtures**:
  - Updated integration books: `Variance` (incorporating all newly supported templates), `hangul`, and `planets` (reflecting improved `slink` section link formatting).
- **Documented conversion rules**:
  - Added new rules in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
  - Registered template handlers in the dispatch table, removed duplicate `render_section_link_template` function, and addressed clippy warnings.
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
  - Registered `hidden end` to `""` in `get_fixed` and added all new templates to `is_handled_template_name`.
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
  - Added unit tests for each template individually, and updated hangul/slink expectations.
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
  - Documented conversion rules for the new templates.
- `expected/Variance/OEBPS/Variance.xhtml` [MODIFY]
  - Regenerated Variance book output.
- `expected/hangul/OEBPS/*`, `expected/planets/OEBPS/*` [MODIFY]
  - Regenerated book outputs to match correct `slink` rendering.
- [src/navigations.csv](file:///opt/src/navigations.csv), [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
  - Sorted alphabetically.
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]
  - Updated with current session notes.

### Tests Run
- Checked compilation: `cargo check` (passed).
- Checked formatting: `cargo fmt -- --check` (passed).
- Checked lints: `cargo clippy --all-targets -- -D warnings` (passed cleanly, no warnings).
- Ran all tests: `cargo test` (all 324 unit tests, 36 integration tests, and 4 doctests passed).

### Pending Follow-Ups
- None.

## 2026-06-16 Handle sfrac, mvar, math, and nested wikitext '='

### Summary
Implemented handlers for `sfrac`, `mvar`, and `math` templates and fixed a critical parser issue to properly support nested `=` signs (such as the `{{=}}` template) inside other templates' parameters (e.g. `math`, `nowrap`, `blockquote`, `infobox`).

### Decisions Made
- **Implemented template renderers**:
  - `sfrac`: formats vulgar fractions using superscript/subscript spans (e.g. `<sup>1</sup>⁄<sub>6</sub>`).
  - `mvar`: wraps variables in italic text (`<em>variable</em>`) via wikitext `''` formatting.
  - `math`: evaluates and passes through mathematical expressions inline, processing nested templates inside it.
- **Fixed parameter parsing**:
  - Replaced the naive `param.split_once('=')` and `param.contains('=')` in [src/tools.rs](file:///opt/src/tools.rs) with a new helper `split_parameter_by_equals` that only splits on `=` signs at the top level of a parameter (ignoring `=` characters inside nested `{{...}}` templates or `[[...]]` links).
  - This solves the issue where templates like `{{=}}` or parameters containing `=` inside nested templates (like `name = ...` inside `{{efn|...}}`) were incorrectly treated as parameter keys, mangling the outer template parsing and dropping field values.
- **Added `=` template**:
  - Mapped `"="` to `"="` in `get_fixed` within [src/templates/mod.rs](file:///opt/src/templates/mod.rs) so that the template `{{=}}` resolves to a literal `=`.
- **Regenerated expected fixtures**:
  - Recompiled expected books where nested template parameters are now correctly parsed and rendered: `Statistical_model`, `goguryeo`, `korean-war`, `parhae`, `planets`, and `south-korea`.
- **Documented conversion rules**:
  - Added conversion rules for `sfrac`, `mvar`, and `math` in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/tools.rs](file:///opt/src/tools.rs) [MODIFY]
  - Implemented `split_parameter_by_equals` and updated `template_named_params` and `template_positional_params`.
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
  - Implemented and registered `sfrac`, `mvar`, and `math` templates in the dispatch table.
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
  - Registered `sfrac`, `mvar`, `math`, and `=` in `is_handled_template_name` and `get_fixed`.
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
  - Added unit tests for the three new templates.
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
  - Added template conversion documentation.
- `expected/Statistical_model/OEBPS/Statistical_model.xhtml` [MODIFY]
  - Updated expected book output to include the rendered math/sfrac/equals elements.
- `expected/goguryeo/OEBPS/*`, `expected/korean-war/OEBPS/*`, `expected/parhae/OEBPS/*`, `expected/planets/OEBPS/*`, `expected/south-korea/OEBPS/*` [MODIFY]
  - Updated book outputs due to corrected nested parameter parsing.
- [src/navigations.csv](file:///opt/src/navigations.csv), [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
  - Alphabetically sorted CSV files.

### Tests Run
- Checked compilation and formatting: `cargo check`, `cargo fmt -- --check`, and `cargo clippy --all-targets -- -D warnings` (All passed cleanly).
- Verified unit and integration tests: `cargo test` (All 316 unit tests, 35 integration tests, and 4 doc-tests passed).

### Pending Follow-Ups
- None.

## 2026-06-15 Render Plain Image Filenames in Infobox Military Conflict

### Summary
Updated the `Infobox military conflict` renderer to recognize and parse plain image filenames (e.g. `Sekigaharascreen.jpg`) by wrapping them into `[[File:filename|thumb]]` links. This ensures that the images are correctly registered, downloaded, and displayed in the generated pages.

### Decisions Made
- Updated `render_infobox_military_conflict_template` in [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) to check if the image parameter starts with `[[` or `{{`. If it does not, the plain filename is formatted into `[[File:filename|thumb]]`.
- Downloaded `Sekigaharascreen.jpg` using `./tools/add_images.pl examples/Battle_of_Sekigahara.yaml` and registered it in `pages/images/manifest.json`.
- Regenerated the expected integration book fixtures for `Battle_of_Sekigahara` using `./tools/regenerate.sh`.
- Ensured template calls inside image parameters (e.g. `{{multiple image ...}}` in the Korean War article) are not wrapped by only applying the formatting to values not starting with `[[` or `{{`.
- Added a unit test `render_wikitext_formats_infobox_military_conflict_template_with_plain_image` to [src/tests.rs](file:///opt/src/tests.rs) to verify that plain image filenames in the military conflict infobox are wrapped and correctly resolved by the image registry.

### Files Changed
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
  - Formatted plain image filenames as file links.
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
  - Added unit test case verifying plain image filename handling.
- [pages/images/manifest.json](file:///opt/pages/images/manifest.json) [MODIFY]
  - Added the mapping entry for `Sekigaharascreen.jpg`.
- `pages/images/Sekigaharascreen.jpg` [ADD]
  - Saved the downloaded screen image.
- `expected/Battle_of_Sekigahara/OEBPS/*` [MODIFY]
  - Updated expected integration book output.

### Tests Run
- Verified all unit and integration tests: `cargo test && cargo fmt --check && cargo check && cargo clippy --all-targets -- -D warnings` (Passed cleanly, no warnings/errors).
- Verified the new unit test `render_wikitext_formats_infobox_military_conflict_template_with_plain_image` runs and passes successfully.

### Pending Follow-Ups
- None.

## 2026-06-15 Add Perl Script to Download Book Images and Update Manifest

### Summary
Implemented a Perl script `tools/add_images.pl` to automatically download missing image files for a given book configuration, mapping them correctly to their actual downloaded file extension and media type. Fixed issues with non-ASCII image filenames (such as `大一大万大吉.svg`) resolving to `_.png` and restored missing manifest entries to resolve failing tests.

### Decisions Made
- Created the Perl utility [tools/add_images.pl](file:///opt/tools/add_images.pl) to parse missing images via `cargo run -- <config_file> --local pages`, fetch their thumbnail or original URL/mime-type from the Wikipedia API, download the files to `pages/images/` using `curl`, and update `pages/images/manifest.json`.
- Handled MIME-type discrepancies: For original SVG files, the Wikipedia API returns a PNG thumbnail URL when queried with a custom width (`iiurlwidth=800`). The script now extracts the actual file extension from the downloaded URL and dynamically maps it to the correct `media-type` in `manifest.json`.
- Fixed filename sanitization: Replaced the ASCII-only sanitization regex `[^\w\-\.\(\)]` in `tools/add_images.pl` with a Unicode property regex `[^\p{L}\p{N}\-\.\(\)_]` to ensure non-ASCII/Unicode characters (e.g. CJK character strings like `大一大万大吉`) are correctly preserved in downloaded filenames instead of being reduced to `_.png`.
- Renamed the existing `pages/images/_.png` file to its correct name `pages/images/大一大万大吉.png` and updated its mapping in `manifest.json`.
- Merged the manifests: Re-added the missing Japan and Busan book image entries to `manifest.json` by merging entries from HEAD with the working copy using a Perl script.
- Configured JSON::PP raw UTF-8 modes: Enabled raw byte reading and encoded UTF-8 JSON writing (`JSON::PP->new->utf8`) to prevent wide character write/read warnings.
- Regenerated the expected integration book fixtures for `administrative-divisions-of-south-korea` using `./tools/regenerate.sh`.

### Files Changed
- [tools/add_images.pl](file:///opt/tools/add_images.pl) [MODIFY]
  - Integrated Unicode sanitization regex, raw JSON encoding/decoding, and warning fixes.
- [pages/images/manifest.json](file:///opt/pages/images/manifest.json) [MODIFY]
  - Merged and registered all book images (South Korea, Sekigahara, Japan, Busan) with correct paths and media-types.
- `pages/images/*.png` [ADD]
  - Renamed `_.png` to `大一大万大吉.png` and added South Korea/Sekigahara images.
- `expected/administrative-divisions-of-south-korea/OEBPS/*` [MODIFY]
  - Updated integration expectations to include the newly downloaded images.

### Tests Run
- Ran `./tools/add_images.pl examples/administrative-divisions-of-south-korea.yaml`.
- Verified compilation, format, and checks: `cargo test && cargo fmt --check && cargo check && cargo clippy --all-targets -- -D warnings` (Passed cleanly, no warnings/errors).

### Pending Follow-Ups
- None.

## 2026-06-15 Embed Wikipedia Labelled Map Templates Generically

### Summary
Implemented generic support for Wikipedia map templates (such as `Template:South Korea Provincial level Labelled Map`) by mapping template names to their base map images in a new configuration file `src/maps.csv`. When encountered, these map templates are rendered as standard `[[File:ImageName|thumb|TemplateName]]` links.

### Decisions Made
- Added a new CSV file `src/maps.csv` containing template name to base image name mappings.
- Wrote a temporary ignored test `scrape_map_templates` in `src/tests.rs` to fetch all templates from `Category:Labelled_map_templates` category via the Wikipedia API and extract their main image filenames.
- Implemented `find_map_image` in `src/templates/mod.rs` to search `src/maps.csv` for matching template names.
- Updated `render_template` and `is_handled_template_name` in `src/templates/mod.rs` to render map templates as standard `[[File:...]]` links.
- Added a unit test in `src/tests.rs` to verify that `{{South Korea Provincial level Labelled Map}}` is formatted correctly.
- Documented the new template conversion rules in `DEVELOPMENT.md`.

### Files Changed
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
  - Integrated `find_map_image` in `render_template` and `is_handled_template_name`.
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
  - Added unit test and the ignored `scrape_map_templates` scraper test.
- [src/maps.csv](file:///opt/src/maps.csv) [ADD]
  - Added mappings for all templates in Category:Labelled_map_templates.
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
  - Documented labelled map conversion rules.
- [src/navigations.csv](file:///opt/src/navigations.csv) [MODIFY]
  - Sorted file using `./sort.sh`.
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
  - Sorted file using `./sort.sh`.

### Tests Run
- Checked compilation and warnings: `cargo check` and `cargo clippy --all-targets -- -D warnings` (Clean).
- Formatted codebase: `cargo fmt` (Clean).
- Executed unit and integration tests: `cargo test` (All passed).

### Pending Follow-Ups
- None.

## 2026-06-13 Handle Wikipedia File and Image Links in Tables and Inline Text

### Summary
Implemented robust rendering of Wikipedia `File:` and `Image:` links (e.g. `[[File:Regions and Prefectures of Japan 2.svg|...]]` in `pages/Japan.json`) located inside table cells and other formatted inline blocks rather than silently omitting them.

### Decisions Made
- Added `process_file_links_into_placeholders` in [src/main.rs](file:///opt/src/main.rs) to parse and register inline images into temporary placeholders (e.g. `__WIKIPEDIA_TO_EPUB_IMAGE_HTML_N__`) before processing normal wikitext/markup formatting, then replacing them back with the rendered `<img>` block to avoid escaping or stripping.
- Modified `cleanup_inline_markup_with_excluded_links` to accept `Option<&mut ImageRegistry>` and `source_page: &str` to process and resolve these image placeholders in place.
- Refactored `render_wikitable`, `render_wikitext_tables_with_excluded_links`, and their callers to propagate the optional `ImageRegistry` and `source_page` into table cell parsing and formatting.
- Added a custom `unzip_helper` binary in [src/bin/unzip_helper.rs](file:///opt/src/bin/unzip_helper.rs) to allow extracting EPUB contents for integration tests during `regenerate.sh` runs when the system does not have the native `unzip` package.
- Added `default-run = "wikipedia-to-epub"` in [Cargo.toml](file:///opt/Cargo.toml) to ensure standard `cargo run` continues to run the main binary since the new helper binary was added.
- Wrote two unit tests `render_wikitext_embeds_japan_file_link` and `test_render_table_with_image` to verify wikitext and table cell image handling.
- Regenerated the expected integration book fixtures for `japan` since it now correctly processes and renders its regions/prefectures SVG image.

### Files Changed
- [src/main.rs](file:///opt/src/main.rs) [MODIFY]
  - Implemented `process_file_links_into_placeholders` and integrated it with table and cell text formatting.
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
  - Added unit test cases for inline wikitext images and table cell image rendering.
- [src/bin/unzip_helper.rs](file:///opt/src/bin/unzip_helper.rs) [ADD]
  - Created a zip-extraction helper utility.
- [Cargo.toml](file:///opt/Cargo.toml) [MODIFY]
  - Added `default-run` key pointing to the main binary.
- [regenerate.sh](file:///opt/regenerate.sh) [MODIFY]
  - Replaced the dependency on `unzip` with a cargo run command using `unzip_helper`.
- `expected/japan/OEBPS/*` [MODIFY]
  - Regenerated Japan fixtures to match the newly added region map SVG image.

### Tests Run
- Checked formatting: `cargo fmt` (Clean)
- Checked compiler checks: `cargo check` (Clean)
- Checked clippy lints: `cargo clippy --all-targets -- -D warnings` (Clean)
- Checked all tests: `cargo test` (All 311 unit tests, 34 integration tests, and 4 doc-tests passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-13 Support Rendering Classless Tables

### Summary
Updated the table parser to render wikitables that do not declare any class attribute (previously skipped). This ensures layouts structured as classless tables (such as the Hangul syllable structure tables in `pages/Hangul.json` or regional lists/infobox containers in other pages) are correctly formatted and included in the output books.

### Decisions Made
- Modified the wikitext table scanner in [src/main.rs](file:///opt/src/main.rs) to trigger rendering when `is_wikitable_attrs` matches OR when `extract_class_attr` returns `None`.
- Cleaned up the unused `tracing::debug` import and simplified the table warning message logic in [src/main.rs](file:///opt/src/main.rs).
- Added a unit test in [src/tests.rs](file:///opt/src/tests.rs) checking that a table with no class attribute is rendered with the default `class="wikitable"` styles.
- Updated `regenerate.sh` to extract EPUB outputs dynamically using a temporary Perl script, resolving issues on systems lacking the native `unzip` package.
- Regenerated the expected integration book fixtures for `hangul`, `japan`, and `korea-in-hebrew` where classless tables are now correctly processed and included.

### Files Changed
- [src/main.rs](file:///opt/src/main.rs) [MODIFY]
  - Allowed parsing classless tables and simplified unused imports/warnings.
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
  - Added unit test covering classless table rendering.
- [regenerate.sh](file:///opt/regenerate.sh) [MODIFY]
  - Dynamic Perl-based zip extraction.
- `expected/hangul/OEBPS/*`, `expected/japan/OEBPS/*`, `expected/korea-in-hebrew/OEBPS/*` [MODIFY]
  - Updated expected integration book output.

### Tests Run
- Checked compilation and warnings: `cargo check` and `cargo clippy --all-targets -- -D warnings` (Clean).
- Formatted codebase: `cargo fmt` (Clean).
- Executed unit and integration tests: `cargo test` (All 309 unit tests and 34 integration tests passed successfully).

### Pending Follow-Ups
- None.

## 2026-06-12 Handle olist, webtrans, OSM, wiktionary-inline, cite opentopomap, EngvarB, Colorbull, how-to, portal-inline, end box, and mp Templates

### Summary
Added support for 11 additional Wikipedia templates:
1. `olist` / `ordered list`: Renders a numbered wikitext list.
2. `webtrans`: Renders a machine-translation external URL link `{title} (in {lang})`.
3. `OSM`: Renders an OpenStreetMap link pointing to nodes, ways, or relations.
4. `wiktionary-inline` / `wiktionary inline` / `wti`: Renders inline links referencing Wiktionary definitions.
5. `cite opentopomap`: Renders topographic map citation wrapper linking to `opentopomap.org`.
6. `EngvarB`: Silently skipped (added to `src/silent.csv`).
7. `Colorbull`: Renders a custom-colored bullet character, optionally wrapped in a wikilink.
8. `how-to`: Silently skipped (added to `src/silent.csv`).
9. `portal-inline` / `portal inline`: Renders an inline link referencing a Wikipedia portal.
10. `end box`: Silently skipped (added to `src/silent.csv`).
11. `mp` / `minor planet`: Formats minor planet designations, subscripting parts of the designation or joining parameters.

### Decisions Made
- Implemented rendering logic for all non-silent templates in [src/templates/citation.rs](file:///opt/src/templates/citation.rs) and [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
- Added separate unit tests for all 11 templates in [src/tests.rs](file:///opt/src/tests.rs).
- Added `EngvarB`, `how-to`, and `end box` to [src/silent.csv](file:///opt/src/silent.csv) and sorted it alphabetically using `./sort.sh`.
- Regenerated the expected integration book fixtures for `planets` because the `Solar System` article uses the `mp` template, which is now correctly parsed and rendered.
- Documented all new templates in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/citation.rs](file:///opt/src/templates/citation.rs) [MODIFY]
  - Implemented and registered `cite opentopomap`.
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
  - Implemented and registered `olist`/`ordered list`, `webtrans`, `osm`, `wiktionary-inline`/`wiktionary inline`/`wti`, `colorbull`, `portal-inline`/`portal inline`, and `mp`/`minor planet`.
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
  - Registered all new handled template names in `is_handled_template_name`.
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
  - Added `EngvarB`, `how-to`, and `end box` to silent list, sorted.
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
  - Added focused unit tests for each new template.
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
  - Documented rules and examples for all 11 new templates.
- `expected/planets/OEBPS/Solar_System.xhtml` [MODIFY]
  - Updated planets book fixture with parsed minor planet designation.

### Tests Run
- Checked compilation and warnings: `cargo check` and `cargo clippy --all-targets -- -D warnings` (Clean).
- Formatted codebase: `cargo fmt` (Clean).
- Executed full test suite: `cargo test` (All 309 unit tests, 34 integration tests, and 4 doc-tests passed).

### Pending Follow-Ups
- None.

## 2026-06-12 Handle ublist, end plainlist, multiref, hosking-jfood, parabr, Multiref2, Age in years..., est., e28, Britannica URL, Surname, and citation-attribution Templates

### Summary
Added support for 12 additional templates requested by the user:
1. `ublist`: Handled as an alias of the existing `unbulleted list` template.
2. `end plainlist`: Added to `src/silent.csv` to be silently ignored.
3. `multiref` & `Multiref2`: Joins positional reference values with a semicolon. Added a top-level split utility to handle digits/positional parameters while ignoring non-digit parameters (like `group=n`).
4. `hosking-jfood`: Renders Richard Hosking book citation format.
5. `parabr`: Outputs paragraph break placeholders.
6. `Age in years, months, weeks and days`: Calculates date duration difference in years, months, weeks, and days.
7. `est.`: Established/estimate abbreviation wrapper.
8. `e28`: Renders Ethnologue 28th edition citation format.
9. `Britannica URL`: Renders Encyclopædia Britannica citation format.
10. `Surname`: Added to `src/silent.csv` to be silently ignored.
11. `citation-attribution`: Renders public domain text attribution format.

### Decisions Made
- Implemented rendering logic for all 10 non-silent templates in [src/templates/citation.rs](file:///opt/src/templates/citation.rs) and [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs).
- Changed template keys to all-lowercase in `get_dispatch_table()` definitions to support case-insensitive lookups from `render_template`.
- Wrote separate unit tests for all 12 templates in [src/tests.rs](file:///opt/src/tests.rs).
- Added a core Perl unzip utility in the scratch directory to allow unpacking EPUB files for integration test regeneration since `unzip` was not installed on the system.
- Regenerated the expected integration book fixtures for `han-dynasty`, `japan`, `korean-language`, `korean-war`, and `north-korea`.
- Updated [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) and sorted the CSV files using `./sort.sh`.

### Files Changed
- [src/templates/citation.rs](file:///opt/src/templates/citation.rs) [MODIFY]
  - Implemented multiref/Multiref2, hosking-jfood, e28, and citation-attribution rendering.
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
  - Implemented parabr, age in years..., est., and Britannica URL rendering.
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
  - Registered the 10 handled templates in `is_handled_template_name`.
- [src/silent.csv](file:///opt/src/silent.csv) [MODIFY]
  - Added `end plainlist` and `surname`. Sorted using `./sort.sh`.
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
  - Added 12 separate unit tests for each template.
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
  - Updated conversion rules for all 12 templates.
- `expected/han-dynasty/OEBPS/*`, `expected/japan/OEBPS/*`, `expected/korean-language/OEBPS/*`, `expected/korean-war/OEBPS/*`, `expected/north-korea/OEBPS/*` [MODIFY]
  - Updated expected integration book output.

### Tests Run
- Checked compilation and warnings: `cargo check` and `cargo clippy --all-targets -- -D warnings` (Clean).
- Formatted codebase: `cargo fmt` (Clean).
- Executed unit, integration, and doc-tests: `cargo test` (All 298 unit tests, 34 integration tests, and 4 doc-tests passed).

### Pending Follow-Ups
- None.

## 2026-06-11 Remove pub(crate) from Unused Template Functions

### Summary
Removed `pub(crate)` visibility modifier from 150 functions across `src/templates/mod.rs`, `src/templates/formatting.rs`, `src/templates/citation.rs`, and `src/templates/convert.rs` that are not referenced outside of their defining module files.

### Decisions Made
- Wrote an automated analysis script to check function usages across files and safely change `pub(crate) fn` to private `fn`.
- Validated that the code continues to compile cleanly and all unit/integration tests pass.

### Files Changed
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
- [src/templates/citation.rs](file:///opt/src/templates/citation.rs) [MODIFY]
- [src/templates/convert.rs](file:///opt/src/templates/convert.rs) [MODIFY]
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]

### Tests Run
- Checked compilation: `cargo check` and `cargo clippy --all-targets -- -D warnings` (Passed cleanly, no warnings).
- Formatted codebase: `cargo fmt` (Passed cleanly).
- Executed unit and integration tests: `cargo test` (All 284 unit tests and 34 integration tests passed successfully).

## 2026-06-11 Refactor Template Dispatch Table

### Summary
Moved all template key-value pairs from the `get_dispatch_table` monolith in `src/templates/mod.rs` to the respective template modules: `src/templates/citation.rs`, `src/templates/formatting.rs`, `src/templates/lang.rs`, and `src/templates/convert.rs`.

### Decisions Made
- Reorganized `get_dispatch_table` functions in all template files so that each template file defines and returns its own local dispatch mappings.
- Cleaned up `src/templates/mod.rs` to import only the template functions actually used directly by it, removing unused template handlers and types like `TemplateHandler`.
- Kept the main template routing mechanism green and all existing tests passing.

### Files Changed
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
  - Replaced the large inline `HashMap::from` in `get_dispatch_table` with `HashMap::new()` and let it pull the submodule dispatch tables. Cleaned up unused template imports.
- [src/templates/citation.rs](file:///opt/src/templates/citation.rs) [MODIFY]
  - Registered citation-related template key-value pairs in its `get_dispatch_table`.
- [src/templates/convert.rs](file:///opt/src/templates/convert.rs) [MODIFY]
  - Registered convert-related template key-value pairs in its `get_dispatch_table`.
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
  - Registered formatting-related template key-value pairs in its `get_dispatch_table`.
- [src/templates/lang.rs](file:///opt/src/templates/lang.rs) [MODIFY]
  - Registered language-related template key-value pairs in its `get_dispatch_table`.

### Tests Run
- Checked compilation: `cargo check` and `cargo clippy --all-targets -- -D warnings` (Passed cleanly, no warnings).
- Formatted codebase: `cargo fmt` (Passed cleanly).
- Executed unit and integration tests: `cargo test` (All 284 unit tests and 34 integration tests passed successfully).

## 2026-06-11 Support Additional Formatting, Citation, and Language Templates

### Summary
Added support for 8 additional Wikipedia templates:
1. `flagdeco`: Silent placeholder template for decorative flags (renders empty string).
2. `pprime`: Renders a double prime symbol (″) after the given text.
3. `RA`: Formats Right Ascension coordinates with superscript hour, minute, and second tags using superscript markers (`__WIKIPEDIA_TO_EPUB_SUP_START__`, etc.).
4. `MW` / `Cite Merriam-Webster`: Formats online Merriam-Webster dictionary citations (supporting learners and medical dictionary types) with official-url formatting.
5. `indented plainlist`: Standard unbulleted list (wrapped identically to `plainlist`).
6. `bulleted list` / `blist`: Standard bulleted list yielding bulleted items.
7. `Hyphen`: Renders a standard hyphen `-` character.
8. `native phrase` / `native name`: Formats a term in its native language followed by its parenthesized language name.

### Decisions Made
- Implemented and registered the 8 new template handlers in [src/templates/mod.rs](file:///opt/src/templates/mod.rs), [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs), [src/templates/citation.rs](file:///opt/src/templates/citation.rs), and [src/templates/lang.rs](file:///opt/src/templates/lang.rs).
- Added comprehensive unit tests for each new template in [src/tests.rs](file:///opt/src/tests.rs).
- Regenerated the expected integration test fixtures for affected books (`parhae`, `north-korea`, `south-korea`, `planets`) because these books now correctly render their native names, page hyphens, and coordinates instead of leaving them blank/unrendered.
- Updated [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
- [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
- [src/templates/citation.rs](file:///opt/src/templates/citation.rs) [MODIFY]
- [src/templates/lang.rs](file:///opt/src/templates/lang.rs) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- `expected/parhae/OEBPS/*` [MODIFY]
- `expected/north-korea/OEBPS/*` [MODIFY]
- `expected/south-korea/OEBPS/*` [MODIFY]
- `expected/planets/OEBPS/*` [MODIFY]

### Tests Run
- `cargo test` (All 284 unit tests and 34 integration tests passed successfully)
- `cargo fmt` and `cargo clippy --all-targets -- -D warnings` (Clean)

## 2026-06-11 Support Longitem Template

### Summary
Added support for the `longitem` Wikipedia template. It acts as a passthrough wrapper for its content.

### Decisions Made
- Registered `longitem` in the template dispatch table mapping to `render_passthrough_template` in [src/templates/mod.rs](file:///opt/src/templates/mod.rs).
- Added `longitem` to `is_handled_template_name` in [src/templates/mod.rs](file:///opt/src/templates/mod.rs).
- Added `render_wikitext_formats_longitem_template` unit test in [src/tests.rs](file:///opt/src/tests.rs).
- Regenerated the expected integration test fixtures for the `planets` book because the `Sun` article uses `longitem` in its infobox and now correctly renders its labels rather than skipping them.
- Updated [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) and sorted the template CSV files using `./sort.sh`.

### Files Changed
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
- `src/navigations.csv`, `src/silent.csv` [MODIFY]
- `expected/planets/OEBPS/Sun.xhtml` [MODIFY]

### Tests Run
- `cargo test` (All 276 unit tests and 34 integration tests passed successfully)
- `cargo fmt` and `cargo clippy --all-targets -- -D warnings` (Clean)

## 2026-06-11 Support New Citation Templates

### Summary
Added support for 11 new Wikipedia citation templates and one template alias:
1. `cite dictionary`: Formats dictionary entries with dictionary name, edition, publisher, and page numbers.
2. `cite press release`: Formats press releases with title annotations and publisher information.
3. `Cite APOD`: Parses dates into standard YYMMDD format to construct NASA's Astronomy Picture of the Day links.
4. `Cite OED` / `OED`: Formats Oxford English Dictionary online entries with DOI or view URLs and term queries.
5. `Cite AV media`: Formats media publications with custom format tags, publishers, and platforms.
6. `Cite American Heritage Dictionary`: Formats dictionary query entries with links to `ahdictionary.com`.
7. `Cite wikisource`: Formats book citations linking to Wikisource pages via interlanguage target structures.
8. `Cite CIA World Factbook`: Formats country profiles with lower-cased kebab URL links.
9. `Cite letter`: Formats historical letters with sender, recipient, and subject metadata.
10. `Cite arXiv`: Formats scientific preprint citations with eprint numbers and arXiv tags.
11. `Cite Q`: Formats Wikidata entries linking to the Wikidata item profiles.

### Decisions Made
- Implemented rendering logic for all 11 templates in [src/templates/citation.rs](file:///opt/src/templates/citation.rs).
- Registered all 11 new renderers and the `OED` template alias in [src/templates/mod.rs](file:///opt/src/templates/mod.rs)'s dispatch table and handled name filter.
- Added comprehensive separate unit tests for each new template and alias in [src/tests.rs](file:///opt/src/tests.rs).
- Updated the expected fixtures for affected books (`goguryeo`, `japan`, `kyoto`, `north-korea`, `osaka`, `planets`, `south-korea`) using a custom Perl unzip helper.
- Documented all new conversion rules in [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md).

### Files Changed
- [src/templates/citation.rs](file:///opt/src/templates/citation.rs) [MODIFY]
  - Implemented the 11 new render functions.
- [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
  - Registered renderers and `OED` alias in the dispatch table and handles list.
- [src/tests.rs](file:///opt/src/tests.rs) [MODIFY]
  - Added individual test functions for each template and the `OED` alias.
- [DEVELOPMENT.md](file:///opt/DEVELOPMENT.md) [MODIFY]
  - Documented rules and examples for all templates and aliases.
- [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]
  - Appended this session note.
- `src/navigations.csv`, `src/silent.csv` [MODIFY]
  - Sorted CSV files.
- `expected/*/OEBPS/*` [MODIFY]
  - Updated expected book integration fixtures.

### Tests Run
- `cargo test` (All 275 unit tests and 34 integration tests passed successfully)
- `cargo fmt` and `cargo clippy --all-targets -- -D warnings` (Clean)

## 2026-06-09 Support Translation, Station, and ja-rail-linem templates

### Summary
Added support for three Wikipedia templates:
1. `Translation`: formats translation values with optional literal annotations and abbreviation tooltips.
2. `Station`: formats railway station links with customizable capitalization, location suffix, and label parameters.
3. `ja-rail-linem`: renders Japanese railway line table rows with line colors, station indicators, line names, and directions.

### Decisions Made
- Added implementation of `Translation` and `ja-rail-linem` in `src/templates/lang.rs`.
- Added implementation of `Station` in `src/templates/formatting.rs`.
- Registered the three templates in `src/templates/mod.rs` (in `render_template` and `is_handled_template_name`).
- Added corresponding unit tests in `src/tests.rs` verifying rendering correctness against spec/examples.
- Sorted the CSV files in the workspace (causing sorting updates to `src/navigations.csv` and `src/silent.csv`).
- Updated `DEVELOPMENT.md` to document the new conversion rules.

### Files Changed
- `src/templates/lang.rs` [MODIFY]
  - Implemented `render_translation_template` and `render_ja_rail_linem_template`.
- `src/templates/formatting.rs` [MODIFY]
  - Implemented `render_station_template`.
- `src/templates/mod.rs` [MODIFY]
  - Registered templates in dispatching router.
- `src/tests.rs` [MODIFY]
  - Added unit test cases for the three templates.
- `src/navigations.csv` [MODIFY]
  - Sorted file alphabetically.
- `src/silent.csv` [MODIFY]
  - Sorted file alphabetically.
- `DEVELOPMENT.md` [MODIFY]
  - Documented conversion rules for the new templates.
- `docs/codex-notes.md` [MODIFY]
  - Appended session notes.

### Tests Run
- `cargo test render_wikitext_formats_translation_template`
- `cargo test render_wikitext_formats_station_template`
- `cargo test render_wikitext_formats_ja_rail_linem_template`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`

## 2026-06-09 Add `harvtxt` and `NDLDC` template support

### Summary

Added support for the `harvtxt` (Harvard citation text format) and `NDLDC` (National Diet Library Digital Collection link identifier) templates. Verified that both render correctly in the "Battle of Sekigahara" integration book test and regenerated the expected book output fixtures.

### Decisions Made

* Implemented `harvtxt` in `src/templates/citation.rs` and `NDLDC` in `src/templates/formatting.rs`.
* Supported `format=url`, `format=pid`, `format=digimeta`, `format=ndljp`, `format=doi`, `format=hdl`, and `format=external` parameters for `NDLDC`.
* Evaluated positional and named parameters of both templates.
* Wrote unit tests for both templates, verifying they output correct XHTML content in the parsed EPUB environment.
* Regenerated the expected fixture files of `examples/Battle_of_Sekigahara.yaml` to include the new template representations.

### Files Changed

* `src/templates/citation.rs` [MODIFY]
  * Implemented `render_harvtxt_template` parsing logic.
* `src/templates/formatting.rs` [MODIFY]
  * Implemented `render_ndldc_template` parsing logic.
* `src/templates/mod.rs` [MODIFY]
  * Registered `harvtxt` and `NDLDC` templates in routing maps.
* `src/tests.rs` [MODIFY]
  * Added unit tests verifying `harvtxt` and `NDLDC` formatting outputs.
* `DEVELOPMENT.md` [MODIFY]
  * Added documentation of the new template conversion rules.
* `expected/Battle_of_Sekigahara/OEBPS/Battle_of_Sekigahara.xhtml` [MODIFY]
  * Updated book fixture with parsed citation entries.
* `src/silent.csv` [MODIFY]
  * Automatically sorted CSV entries alphabetically.

## 2026-06-09 Add Military status/abbreviation templates

### Summary

Added support for all 14 military-related status and abbreviation templates (`AWOL`, `Assassinated`, `DOW`, `Died of wounds`, `Executed`, `KIA`, `KIA2`, `MIA`, `Natural Causes`, `PKIA`, `POW`, `Suicide`, `Surrendered`, `Turncoat`, `WIA`) commonly used within `Infobox military conflict` and battle description tables on Wikipedia. Added comprehensive unit test coverage for each template, and documented their conversion rules in `DEVELOPMENT.md`.

### Decisions Made

* Implemented the templates in `src/templates/formatting.rs` to match the style of other inline formatting and annotation templates.
* Handled named parameters such as `alt` and `bold` for templates like `KIA`, `Assassinated`, `Natural Causes`, and `Suicide` to output the correct labels, tooltips, and styles.
* Parsed and rendered inner templates like `{{tooltip|...}}` and `{{abbr|...}}` correctly by routing them through the main `render_templates` engine.
* Added separate focused unit tests for each of the 14 templates.

### Files Changed

* `src/templates/formatting.rs` [MODIFY]
  * Appended the 14 military template render functions.
* `src/templates/mod.rs` [MODIFY]
  * Registered the 14 new templates in the main dispatcher and handles-list.
* `src/tests.rs` [MODIFY]
  * Added unit tests verifying output representation and configuration handling for each template.
* `DEVELOPMENT.md` [MODIFY]
  * Added documentation of the new military template conversion rules.

## 2026-06-09 Add `official` Template Alias

### Summary

Added support for the `official` template name by routing it through the existing `official website` renderer. This keeps external official-site links working for both template names without changing output format.

### Decisions Made

* Treated `official` as an alias of `official website` instead of creating a separate renderer, since the expected parameters and output format are the same.
* Added focused unit coverage for the alias form rather than changing any fixtures, because the cached pages currently use `official website` already.

### Files Changed

* `src/templates/mod.rs` [MODIFY]
  * Wired `official` into template dispatch and recognized-template tracking.
* `src/tests.rs` [MODIFY]
  * Added unit coverage for the `official` alias.
* `DEVELOPMENT.md` [MODIFY]
  * Documented `official` as an alias of `official website`.
* `docs/codex-notes.md` [MODIFY]
  * Added this session note.

### Tests Run

* `cargo test render_wikitext_formats_official_website_templates`
* `./sort.sh`
* `cargo fmt`
* `cargo test`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`

### Pending Follow-Ups

* None.


## Session Note: 2026-06-26 - Handle H-series templates from h.txt

### Decisions Made

* Used the `handle-template` skill for Wikipedia template support work.
* Checked Wikipedia Template pages for H-series definitions, including renderable cases such as `H2G2`, `hbf`, `hdl`, `HDS`, `HESA student population`, `Hidden`, `Hiero`, `highlight`, `historical population`, `HKG`, `HKG-CHN`, `Hl-Lex`, `Hounshell1984`, `hr`, `Hungarian county link`, `Hungarian county name`, and `Hungarian name`.
* Added renderer support for H-series country/territory code templates, Harvard citation aliases, station/identifier/reference helpers, hidden/highlight/hieroglyph wrappers, historical population aliasing, horizontal rules, and Hungarian county helpers.
* Added H-series topic, history, Hungary, humanities, Holocaust, hydrography, cleanup, data, and map/sidebar templates to recognized skip lists where EPUB output should omit them.
* Added one focused unit test per unique template name from `h.txt`.

### Files Changed

* `DEVELOPMENT.md` [MODIFY]
* `src/navigations.csv` [MODIFY]
* `src/silent.csv` [MODIFY]
* `src/templates/citation.rs` [MODIFY]
* `src/templates/formatting.rs` [MODIFY]
* `src/templates/mod.rs` [MODIFY]
* `src/tests.rs` [MODIFY]
* `docs/codex-notes.md` [MODIFY]

### Tests Run

* `./tools/sort.sh`
* `cargo fmt`
* `cargo test test_template_h -- --nocapture`
* `cargo check`
* `cargo test`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test --locked -- --ignored`

### Pending Follow-Ups

* None.


## Session Note: 2026-06-26 - Handle G-series templates from g.txt

### Decisions Made

* Used the `handle-template` skill for Wikipedia template support work.
* Checked Wikipedia Template pages for the listed G-series templates, including renderable cases such as `GamesName`, `GamesSport`, `GEOnet2`, `Glottolog`, `Google Scholar ID`, `Guardian topic`, `Gutenberg author`, `Goal`, medal templates, and country-code redirects.
* Added renderer support for G-series country codes, Olympic games/sport helpers, geography/source/profile links, goal/medal markers, glossary/bracket/transliteration helpers, and the Greenwood/Earnshaw chemistry citation.
* Added G-series topic, sidebar, map, subscription, cleanup, and navigation templates to recognized skip lists where EPUB output should omit them.
* Added one focused unit test per unique template name from `g.txt`.

### Files Changed

* `DEVELOPMENT.md` [MODIFY]
* `src/navigations.csv` [MODIFY]
* `src/silent.csv` [MODIFY]
* `src/templates/formatting.rs` [MODIFY]
* `src/tests.rs` [MODIFY]
* `docs/codex-notes.md` [MODIFY]

### Tests Run

* `./tools/sort.sh`
* `cargo fmt`
* `cargo test test_template_g -- --nocapture`
* `cargo check`
* `cargo test`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test --locked -- --ignored`

### Pending Follow-Ups

* None.


## Session Note: 2026-06-19 - Add cleanup parser unit tests

### Decisions Made

* Added focused unit tests for `normalize_reference_attr` in `src/cleanup.rs`.
* Covered whitespace trimming, double-quoted values, single-quoted values, and whitespace inside surrounding quotes.
* Added focused unit tests for `parse_reference_tags` covering content refs, self-closing refs, unquoted attributes, empty value filtering, and reference order.
* Added focused unit tests for `matching_template_end` covering simple, nested, unclosed, and nonzero-offset templates.

### Files Changed

* `src/cleanup.rs` [MODIFY]
* `docs/codex-notes.md` [MODIFY]

### Tests Run

* `cargo test` (passed)
* `cargo fmt` (passed)
* `cargo check` (passed)
* `cargo clippy --all-targets -- -D warnings` (passed)
* `cargo test --locked -- --ignored` (passed)

### Pending Follow-Ups

* None.

## 2026-06-08 Add Google Books Template Support

### Summary

Added support for the `Google books` template so the Joseon fixture now renders its bibliography entry as a clickable Google Books external link instead of dropping the template output.

### Decisions Made

* Reused the existing Google Books URL-building rules from `GBurl` so `id`, `page`, `pg`, and query parameters stay consistent across both templates.
* Rendered `Google books` as an external link with the supplied label text, defaulting to `Google Books` when the template has no explicit label.
* Regenerated only the affected Joseon expected fixture whose XHTML changed because the reference entry now includes a live Google Books link.

### Files Changed

* `src/templates/formatting.rs` [MODIFY]
  * Added `Google books` rendering and shared Google Books URL construction logic.
* `src/templates/mod.rs` [MODIFY]
  * Wired `Google books` into template dispatch and recognized-template tracking.
* `src/tests.rs` [MODIFY]
  * Added focused unit coverage for the Joseon `Google books` form.
* `expected/japan/` [MODIFY]
* `expected/joseon/` [MODIFY]
  * Updated expected XHTML output for rendered Google Books output in references/citations.
* `DEVELOPMENT.md` [MODIFY]
  * Documented `Google books` conversion behavior.
* `docs/codex-notes.md` [MODIFY]
  * Added this session note.

### Tests Run

* `cargo test render_wikitext_formats_google_books_template`
* `cargo test --test books generate_joseon_book_from_local_page_dumps`
* `./sort.sh`
* `cargo fmt`
* `cargo test`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`

### Pending Follow-Ups

* None.

## 2026-06-08 Reject Duplicate Pages in YAML Config

### Summary

Configuration loading now fails before processing if the same page is listed more than once anywhere in the article tree. Added a focused regression test covering a duplicate page repeated under a nested section.

### Decisions Made

* Enforced duplicate detection during config parsing so invalid configs fail before page loading or EPUB generation starts.
* Reused the existing page lookup normalization rules for duplicate detection, so equivalent page spellings are treated as the same page.
* Ignored section container titles when checking duplicates, since only actual page entries should count as included pages.

### Files Changed

* `src/config.rs` [MODIFY]
  * Added recursive duplicate-page validation for parsed article trees.
* `src/tests.rs` [MODIFY]
  * Added coverage for duplicate page rejection in YAML configs.
* `docs/codex-notes.md` [MODIFY]
  * Added this session note.

### Tests Run

* `cargo test read_config_rejects_duplicate_pages_with_clear_error`
* `./sort.sh`
* `cargo fmt`
* `cargo test`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`

### Pending Follow-Ups

* None.

## 2026-06-08 Reject Invalid YAML Configurations

### Summary

Tightened YAML configuration parsing so unknown fields are rejected and missing or invalid fields surface as clear file-scoped configuration errors before any book processing starts. Added focused tests for unknown-field, invalid-value, and missing-field failures.

### Decisions Made

* Added `serde(deny_unknown_fields)` to the main config structs so stray keys are rejected instead of silently ignored.
* Wrapped YAML deserialization in a config-specific parser helper that prefixes failures with the config path and line/column when available.
* Kept the existing field requirements intact and extended tests around the clearer error surface instead of changing the config schema itself.

### Files Changed

* `src/config.rs` [MODIFY]
  * Rejected unknown fields on config structs and added clearer file-scoped YAML error reporting.
* `src/tests.rs` [MODIFY]
  * Added focused tests covering unknown fields, invalid enum values, and missing required fields.
* `docs/codex-notes.md` [MODIFY]
  * Added this session note.

### Tests Run

* `cargo test read_config_rejects_unknown_fields_with_clear_error`
* `cargo test read_config_rejects_invalid_values_with_clear_error`
* `cargo test read_config_rejects_missing_fields_with_clear_error`
* `cargo test book_config_requires_links_to_excluded_pages`
* `./sort.sh`
* `cargo fmt`
* `cargo test`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`

### Pending Follow-Ups

* None.

## 2026-06-08 Show Both `convert` Values

### Summary

Expanded `convert`/`cvt` rendering so supported page cases now keep the original value and add the converted value in parentheses. Updated the affected expected book fixtures and documented the revised conversion behavior.

### Decisions Made

* Replaced the old meter-only parenthetical fallback with broader `convert` handling that supports explicit alternate units and common default counterparts used by the cached page fixtures.
* Added coverage for compound convert forms from `pages/`, including ranges, paired values, multi-unit outputs such as `K -> C/F`, and page-backed regression checks that scan fixture content.
* Regenerated only the expected books whose chapter XHTML changed because of the new parenthetical convert output.

### Files Changed

* `src/templates/convert.rs` [MODIFY]
  * Added broader convert-unit parsing, conversion, formatting, default target inference, and parenthetical output generation.
* `src/tests.rs` [MODIFY]
  * Updated convert expectations and added fixture-backed coverage for supported `convert` cases found in `pages/`.
* `expected/busan-images/` [MODIFY]
* `expected/busan/` [MODIFY]
* `expected/han-dynasty/` [MODIFY]
* `expected/japan/` [MODIFY]
* `expected/korea/` [MODIFY]
* `expected/korean-war/` [MODIFY]
* `expected/kyoto/` [MODIFY]
* `expected/north-korea/` [MODIFY]
* `expected/osaka/` [MODIFY]
* `expected/planets/` [MODIFY]
* `expected/sejong-the-great/` [MODIFY]
* `expected/seoul/` [MODIFY]
* `expected/south-korea/` [MODIFY]
  * Updated expected XHTML output to match the new parenthetical convert rendering.
* `DEVELOPMENT.md` [MODIFY]
  * Documented that convert output now shows secondary values in parentheses.
* `docs/codex-notes.md` [MODIFY]
  * Added this session note.

### Tests Run

* `cargo test render_wikitext_formats_convert_templates`
* `cargo test render_wikitext_shows_secondary_convert_values_for_supported_page_cases`
* `cargo test --test books`
* `./sort.sh`
* `cargo fmt`
* `cargo fmt --check`
* `cargo test`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`

### Pending Follow-Ups

* None.

## 2026-06-08 Add Infobox military conflict Support

### Summary

Added support for rendering `Infobox military conflict` templates as two-column wikitables so military-conflict pages now include their infobox content instead of silently dropping it. Updated the affected expected book fixture and documented the new conversion rule.

### Decisions Made

* Added a dedicated `render_infobox_military_conflict_template` renderer in `src/templates/formatting.rs` rather than routing through the generic infobox path, because the cached conflict page uses a stable set of battle-specific fields.
* Rendered the key fields used by the cached fixture: conflict name, image/caption, part of, date, place, result, territorial changes, combatants, commanders, strength, casualties, and notes.
* Kept the output in the existing wikitable format used by other supported infoboxes so it flows through the current table-rendering pipeline unchanged.
* Updated only the affected expected book fixture whose source page contains `Infobox military conflict`.

### Files Changed

* `src/templates/formatting.rs` [MODIFY]
  * Added `render_infobox_military_conflict_template`.
* `src/templates/mod.rs` [MODIFY]
  * Wired `Infobox military conflict` into template dispatch and infobox allowlists.
* `src/tests.rs` [MODIFY]
  * Added unit coverage for `Infobox military conflict`.
* `expected/korean-war/` [MODIFY]
  * Updated expected XHTML output to include the rendered military conflict infobox.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the new `Infobox military conflict` conversion rule.
* `docs/codex-notes.md` [MODIFY]
  * Added this session note.

### Tests Run

* `./sort.sh`
* `cargo test render_wikitext_formats_infobox_military_conflict_template`
* `cargo test --test books`
* `cargo test`
* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`

### Pending Follow-Ups

* None.

## 2026-06-08 Add Infobox country Support

### Summary

Added support for rendering `Infobox country` templates as two-column wikitables so country/state pages now include their infobox content instead of silently dropping it. Updated the affected expected book fixtures and documented the new conversion rule.

### Decisions Made

* Added a dedicated `render_infobox_country_template` renderer in `src/templates/formatting.rs` rather than routing through the generic infobox path, because the cached country pages use a stable set of domain-specific fields.
* Rendered the core fields actually used by the cached fixtures: long/common/native names, flags/symbols, anthem, motto/status/government, capital/language/demonym/religion/currency, establishment years, and historical event/predecessor/successor rows.
* Kept media fields consistent with the project’s existing infobox handling by rendering image filenames/symbol content as textual table cell content instead of introducing a separate image pipeline.
* Updated only the six expected books that actually contain `Infobox country` in their source pages.

### Files Changed

* `src/templates/formatting.rs` [MODIFY]
  * Added `render_infobox_country_template`.
* `src/templates/mod.rs` [MODIFY]
  * Wired `Infobox country` into template dispatch and infobox allowlists.
* `src/tests.rs` [MODIFY]
  * Added unit coverage for `Infobox country`.
* `expected/goguryeo/` [MODIFY]
* `expected/japan/` [MODIFY]
* `expected/joseon/` [MODIFY]
* `expected/north-korea/` [MODIFY]
* `expected/parhae/` [MODIFY]
* `expected/south-korea/` [MODIFY]
  * Updated expected XHTML/metadata files to match rendered country infoboxes.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the new `Infobox country` conversion rule.
* `docs/codex-notes.md` [MODIFY]
  * Added this session note.

### Tests Run

* `./sort.sh`
* `cargo test render_wikitext_formats_infobox_country_template`
* `cargo test --test books`
* `cargo test`
* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`

### Pending Follow-Ups

* None.

## 2026-06-08 Add Reflist Rendering

### Summary

Changed `Reflist` from an ignored template into rendered output. The renderer now collects page `<ref>` tags before the normal cleanup strips them, and inserts ordered reference lists at `{{Reflist}}` locations, including grouped reflists such as `{{Reflist|group=n}}`.

### Decisions Made

* Implemented page-level reference collection in `src/main.rs` rather than a simple template function, because `Reflist` depends on `<ref>` tags that were previously removed before template rendering.
* Added support for named references, self-closing named references, grouped references, and definitions supplied through `{{Reflist|refs=...}}`.
* Kept empty `{{Reflist}}` blocks silent so pages without collected references do not gain blank placeholder output.
* Left `notelist` and related wrappers unchanged; only `Reflist` now renders collected references.
* Regenerated all affected expected books whose chapter output changed because references are now visible.

### Files Changed

* `src/main.rs` [MODIFY]
  * Added reference parsing, grouped reference collection, reflist placeholder replacement, and ordered reference list rendering.
* `src/tests.rs` [MODIFY]
  * Added reflist unit coverage and updated the metadata-template skip-count expectation.
* `DEVELOPMENT.md` [MODIFY]
  * Documented rendered `Reflist` behavior and narrowed the omission note to the still-ignored wrappers.
* `expected/administrative-divisions-of-south-korea/` [MODIFY]
* `expected/buddhist-temples-in-japan/` [MODIFY]
* `expected/busan-images/` [MODIFY]
* `expected/busan/` [MODIFY]
* `expected/goguryeo/` [MODIFY]
* `expected/han-dynasty/` [MODIFY]
* `expected/hangul/` [MODIFY]
* `expected/history-of-korea/` [MODIFY]
* `expected/japan/` [MODIFY]
* `expected/joseon/` [MODIFY]
* `expected/Kiso_Mountains/` [MODIFY]
* `expected/korea/` [MODIFY]
* `expected/korean-language/` [MODIFY]
* `expected/korean-war/` [MODIFY]
* `expected/kyoto/` [MODIFY]
* `expected/macchini-deep/` [MODIFY]
* `expected/macchini/` [MODIFY]
* `expected/north-korea/` [MODIFY]
* `expected/osaka/` [MODIFY]
* `expected/parhae/` [MODIFY]
* `expected/planets/` [MODIFY]
* `expected/sejong-the-great/` [MODIFY]
* `expected/seoul/` [MODIFY]
* `expected/south-korea/` [MODIFY]
  * Updated expected XHTML/metadata files to match rendered reflists.
* `docs/codex-notes.md` [MODIFY]
  * Added this session note.

### Tests Run

* `cargo test render_wikitext_formats_reflist`
* `cargo test render_wikitext_silently_skips_metadata_templates`
* `./sort.sh`
* `cargo test --test books`
* `cargo test`
* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`

### Pending Follow-Ups

* None.

## 2026-06-08 Add Comma Grouping to convert Values

### Summary

Updated `convert` template rendering so large numeric values are grouped with commas every three digits from the right, e.g. `384400 km` now renders as `384,400 km`. Refreshed the affected expected book fixtures whose content changed because of the new formatting.

### Decisions Made

* Reused the existing shared `format_number_with_commas()` helper instead of adding a second number-grouping implementation.
* Applied grouping in `format_convert_value()` so the behavior covers standard `convert` rendering consistently, while preserving the original minus-sign style (`-` vs `−`) from the input.
* Added a focused test case for `{{convert|384400|km}}`.
* Regenerated only the affected expected books whose rendered chapter XHTML changed due to comma insertion.

### Files Changed

* `src/templates/convert.rs` [MODIFY]
  * Applied comma grouping in `format_convert_value()` while preserving sign style.
* `src/tests.rs` [MODIFY]
  * Updated convert expectations and added coverage for `384,400 km`.
* `expected/Kiso_Mountains/` [MODIFY]
* `expected/han-dynasty/` [MODIFY]
* `expected/japan/` [MODIFY]
* `expected/korea/` [MODIFY]
* `expected/kyoto/` [MODIFY]
* `expected/north-korea/` [MODIFY]
* `expected/osaka/` [MODIFY]
* `expected/planets/` [MODIFY]
* `expected/south-korea/` [MODIFY]
  * Updated expected XHTML/EPUB metadata to match the grouped convert output.
* `DEVELOPMENT.md` [MODIFY]
  * Documented grouped numeric output for `convert`.
* `docs/codex-notes.md` [MODIFY]
  * Added this session note.

### Tests Run

* `./sort.sh`
* `cargo test render_wikitext_formats_convert_templates`
* `cargo test --test books`
* `cargo test`
* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`

### Pending Follow-Ups

* None.

## 2026-06-08 Add Infobox planet Support

### Summary

Added support for rendering `Infobox planet` templates as two-column wikitables so planetary article pages now include their infobox content instead of dropping it. Updated the affected `planets` expected EPUB fixtures and documented the new conversion rule.

### Decisions Made

* Added a dedicated `render_infobox_planet_template` renderer in `src/templates/formatting.rs` rather than routing through the generic infobox path, because the planet template has stable domain-specific fields worth labeling explicitly.
* Rendered key sections including symbol, image/caption, orbital parameters, physical properties, temperature rows, and atmosphere-related fields.
* Added special handling for file-link-based planet symbols so glyphs like `♂` survive the wikitext cleanup path.
* Kept the existing table-rendering pipeline by emitting a `wikitable`, including the project’s internal line-break placeholder where multi-line cell content is needed.
* Regenerated the affected `expected/planets/OEBPS/*.xhtml` fixtures from a fresh local build because the new infobox output intentionally changes chapter XHTML.

### Files Changed

* `src/templates/formatting.rs` [MODIFY]
  * Added `render_infobox_planet_template`.
* `src/templates/mod.rs` [MODIFY]
  * Wired `Infobox planet` into template dispatch and infobox allowlists.
* `src/tests.rs` [MODIFY]
  * Added unit coverage for `Infobox planet`.
* `expected/planets/OEBPS/Earth.xhtml` [MODIFY]
* `expected/planets/OEBPS/Mars.xhtml` [MODIFY]
* `expected/planets/OEBPS/Venus.xhtml` [MODIFY]
  * Updated expected XHTML output to include rendered planetary infobox tables.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the new `Infobox planet` conversion rule.
* `docs/codex-notes.md` [MODIFY]
  * Added this session note.

### Tests Run

* `./sort.sh`
* `cargo test render_wikitext_formats_infobox_planet_template`
* `cargo test --test books`
* `cargo test`
* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`

### Pending Follow-Ups

* None.

## 2026-06-08 Add HTML Book Report

### Summary

Added generation of a companion HTML report file for each book build. The report shows the included book hierarchy and lists same-language Wikipedia pages that were linked from included pages but were not themselves included, to help authors decide what to add next.

### Decisions Made

* Reused the already-loaded article graph instead of doing extra fetches, so report generation reflects exactly what went into the book.
* Wrote the report next to the EPUB output, using the same basename and an `.html` extension.
* Built the included hierarchy from `TocNode` data and filtered out non-article leaf nodes such as auxiliary front-matter/report-only entries.
* Listed excluded Wikipedia pages as links and included the source pages they were linked from.
* Added integration coverage that verifies the report file is created and contains included hierarchy entries plus at least one excluded page link.

### Files Changed

* `src/main.rs` [MODIFY]
  * Added report path derivation, excluded-link collection, report HTML rendering, and report writing.
* `tests/books.rs` [MODIFY]
  * Added assertions for report file creation and content.
* `README.md` [MODIFY]
  * Documented the generated HTML report.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the companion report behavior.
* `docs/codex-notes.md` [MODIFY]
  * Added this session note.

### Tests Run

* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test`

### Pending Follow-Ups

* None.

## 2026-06-08 Add links_to_excluded_pages Configuration

### Summary

Added a new required config field, `links_to_excluded_pages`, to control how same-language Wikipedia links are rendered when the linked page is not included in the generated book. The new modes are `display`, `emphasize`, and `disregard`, with all existing example/test configs set to `emphasize` to preserve current output.

### Decisions Made

* Added `LinksToExcludedPages` to `src/config.rs` and made `BookConfig.links_to_excluded_pages` required.
* Threaded the excluded-page link policy through chapter rendering so normal paragraphs, headings, table cells, and image captions all honor the same behavior.
* Kept `emphasize` as the explicit value in all existing YAML configs to preserve the current external-link-arrow behavior.
* Added unit tests covering `display`, `disregard`, config parsing, and missing-field rejection.
* Updated `DEVELOPMENT.md`, `skeleton.yaml`, and the commented `examples/Kiso_Mountains.yaml` template docs to describe the new field.

### Files Changed

* `src/config.rs` [MODIFY]
  * Added the required `LinksToExcludedPages` enum and config field.
* `src/main.rs` [MODIFY]
  * Added excluded-page link policy-aware rendering helpers and link handling.
* `src/epub.rs` [MODIFY]
  * Passed the config field into chapter rendering.
* `src/tests.rs` [MODIFY]
  * Added policy/config tests and updated inline YAML fixtures.
* `tests/books.rs` [MODIFY]
  * Updated inline YAML fixtures.
* `examples/*.yaml` [MODIFY]
  * Added `links_to_excluded_pages: emphasize` to all existing example configs.
* `skeleton.yaml` [MODIFY]
  * Documented the new required field.
* `examples/Kiso_Mountains.yaml` [MODIFY]
  * Documented the new required field in the commented example.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the new config field and excluded-link behavior.
* `docs/codex-notes.md` [MODIFY]
  * Added this session note.

### Tests Run

* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test`

### Pending Follow-Ups

* None.

## 2026-06-08 Remove Config Metadata Date Field

### Summary

Removed the optional `metadata.date` field from the Rust config schema and switched EPUB generation to always stamp the current UTC date at render time. Also removed the leftover date-field comments from the commented YAML templates.

### Decisions Made

* Deleted `date` from `src/config.rs::Metadata` so the config schema no longer models a user-supplied publication date.
* Simplified `read_config()` so it only deserializes the YAML instead of mutating metadata after load.
* Added `current_utc_date_string()` and computed the generated book date once in `write_epub()` so front matter and `content.opf` always use the same value.
* Kept the generated date in the output book, but no longer sourced it from YAML.
* Removed the obsolete commented publication-date block from `skeleton.yaml` and `examples/Kiso_Mountains.yaml`.

### Files Changed

* `src/config.rs` [MODIFY]
  * Removed `Metadata.date`, stopped mutating config dates after deserialization, and added `current_utc_date_string()`.
* `src/epub.rs` [MODIFY]
  * Switched front matter and OPF date generation to use the runtime-generated current date.
* `skeleton.yaml` [MODIFY]
  * Removed commented guidance for the old `metadata.date` field.
* `examples/Kiso_Mountains.yaml` [MODIFY]
  * Removed commented guidance for the old `metadata.date` field.
* `docs/codex-notes.md` [MODIFY]
  * Added this session note.

### Tests Run

* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test`

### Pending Follow-Ups

* None.

## 2026-06-08 Modularize src/main.rs Refactoring Proposal

### Summary

Proposed a comprehensive refactoring and modularization strategy for `src/main.rs` to break down the 9,700+ line monolith into clean, single-responsibility files and modules. Analyzed the code's structural dependencies, mapped functions to targets, and created a formal refactoring proposal artifact.

### Decisions Made

* Created a structured refactoring proposal in `refactoring_proposal.md` detailing target files, module responsibilities, a Mermaid diagram of dependencies, component-to-file mapping, and an incremental execution plan.
* Defined the target module architecture:
  * `src/error.rs` (AppError & Conversions)
  * `src/config.rs` (CLI & Book Config)
  * `src/cache.rs` (Caching & Page Sources)
  * `src/epub.rs` (EPUB writing & chapter loading)
  * `src/image.rs` (Scraped image registry & download logic)
  * `src/parser.rs` (Wikitext-to-HTML parser core)
  * `src/templates/` (Mod-level routing & themed template submodules: citation, lang, formatting, convert)
* Outlined a step-by-step migration plan starting from leaf dependencies (`error.rs`) up to `parser.rs` to keep the build green and tests passing at every increment.
* Verified that the current branch is clean and passes all 225 unit tests before proposing architectural changes.

### Files Changed

* `docs/codex-notes.md` [MODIFY]
  * Appended session summary.
* `refactoring_proposal.md` [NEW] (located in the artifacts directory)
  * Created proposal.

### Tests Run

* `git status` (verified working tree clean)
* `cargo check` (passed cleanly)
* `cargo test` (all 225 unit tests and 33 integration tests passed successfully)

### Pending Follow-Ups

* Solicit user feedback on the proposed modularization architecture.

## 2026-06-08 Wikipedia Template Documentation Links for render_* Functions

### Summary

Added Wikipedia template documentation links as doc-comments (`///`) to all template-rendering helper functions starting with `render_*` in `src/main.rs`. Extracted mappings from the central `render_template` routing table to correctly associate each renderer with its corresponding Wikipedia templates. Verified all quality checks and test suites pass successfully.

### Decisions Made

* Extracted template mappings dynamically from `render_template` in `src/main.rs` to map each function to its templates.
* Formatted Wikipedia template doc links as: `/// [TemplateName](https://en.wikipedia.org/wiki/Template:TemplateName)`
* Replaced spaces with underscores and capitalized the first letter of titles for correct Wikipedia URL mapping (e.g., `Template:Historical_populations`).
* Checked if functions already have doc-comments (such as the user's manual comment on `render_convert_template`) and skipped them to prevent duplicate comments.
* Checked that the code compiles (`cargo check`, `cargo clippy`) and passes all tests.

### Files Changed

* `src/main.rs` [MODIFY]
  * Prepended doc comments for all template-rendering helper functions.
* `docs/codex-notes.md` [MODIFY]
  * Added session notes at the beginning of the file.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 225 unit tests and 33 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-08 Infobox and Infobox settlement Support

### Summary

Added support for rendering `Infobox settlement` and generic `Infobox` templates as clean two-column `wikitable` structures containing their respective titles, images, captions, and key/value properties. Updated all unit tests, regenerated the expected EPUB XHTML fixtures for affected books (Osaka, Planets, Busan, Busan Images, Kyoto, Seoul), and verified all check scripts and test suites pass successfully.

### Decisions Made

* Excluded `Infobox settlement` and `Infobox` (generic infobox) from being skipped silently in `is_silent_template_name`.
* Implemented `render_infobox_settlement_template` to output a 2-column wikitext table (`{| class="wikitable"` / `|}`) containing populated fields like Name, Official name, Native name, Country, Region, Prefecture, Governing body, Area, Population, Density, Time zone, Coordinates, blank symbol/address sections, and Website.
* Implemented `render_infobox_generic_template` to dynamically output a 2-column wikitext table for the standard `Template:Infobox` structure containing `title`, `image`, `caption`, and sequential `headerX`, `labelX`, and `dataX` parameters.
* Added standalone unit tests to `src/tests.rs` for both `Infobox settlement` and generic `Infobox`.
* Updated `DEVELOPMENT.md` to document the new template rendering conversion rules.
* Updated `regenerate.sh` to extract EPUB books using Python's standard `zipfile` module since `unzip` is missing.
* Regenerated expected EPUB book fixtures for `osaka`, `planets`, `busan`, `busan-images`, `kyoto`, and `seoul`.
* Ran `./sort.sh` to sort the template databases.
* Resolved collapsible `if` clippy lints in the new renderers.

### Files Changed

* `DEVELOPMENT.md` [MODIFY]
  * Documented `Infobox settlement` and generic `infobox` rendering rules.
* `regenerate.sh` [MODIFY]
  * Changed `unzip` invocation to use `python3 -m zipfile`.
* `src/main.rs` [MODIFY]
  * Added dispatching, classification, and implementation for `render_infobox_settlement_template` and `render_infobox_generic_template`.
* `src/tests.rs` [MODIFY]
  * Added unit tests.
  * Replaced `Infobox` / `Infobox settlement` with `Infobox road` in silent skip tests.
* `expected/busan-images/` [MODIFY]
* `expected/busan/` [MODIFY]
* `expected/kyoto/` [MODIFY]
* `expected/osaka/` [MODIFY]
* `expected/planets/` [MODIFY]
* `expected/seoul/` [MODIFY]
  * Regenerated expected XHTML, OPF, and nav fixtures reflecting the newly rendered infobox tables.
* `docs/codex-notes.md` [MODIFY]
  * Prepended session notes.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 224 unit tests and 33 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-07 Infobox Mountain Support for Kiso Mountains

### Summary

Added support for rendering the `Infobox mountain` template as a clean two-column `wikitable` structure containing key properties (Name, Native name, Country, Highest point, Coordinates, etc.). Added support for list and name templates `hlist`/`flatlist` and `native name list` which are nested within the mountain infobox. Updated all unit tests, regenerated the expected `Kiso_Mountains` EPUB XHTML fixture, sorted templates using `./sort.sh`, and verified all compilation checks and tests pass successfully.

### Decisions Made

* Excluded `Infobox mountain` from being skipped silently in `is_silent_template_name`.
* Implemented `render_infobox_mountain_template` to output a 2-column wikitext table (`{| class="wikitable"` / `|}`) containing populated fields. This feeds perfectly into the existing table rendering pipeline, allowing nested links to be parsed and formatted cleanly.
* Implemented `render_hlist_template` to render `hlist` and `flatlist` templates, joining items with commas.
* Implemented `render_native_name_list_template` to render the names and their corresponding language tags recursively (up to 10 entries).
* Added three new unit tests to `src/tests.rs` for `hlist`, `native name list`, and `Infobox mountain` templates.
* Updated `DEVELOPMENT.md` to document the new template rendering conversion rules.
* Regenerated expected `Kiso_Mountains` EPUB book fixtures by unzipping with python3's `zipfile` module.
* Ran `./sort.sh` to sort the template databases alphabetically.

### Files Changed

* `DEVELOPMENT.md` [MODIFY]
  * Documented new conversion rules.
* `src/main.rs` [MODIFY]
  * Added routing and template renderers for `Infobox mountain`, `native name list`, and `hlist`/`flatlist`.
* `src/tests.rs` [MODIFY]
  * Added unit tests.
* `src/silent.csv` [MODIFY]
  * Alphabetically sorted templates.
* `expected/Kiso_Mountains/OEBPS/Kiso_Mountains.xhtml` [MODIFY]
  * Updated with the newly rendered `Infobox mountain` table content.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 222 unit tests and 33 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-07 Extract Mock Date from Expected EPUB Metadata

### Summary

Replaced `extract_yaml_date` with `extract_opf_date` in the integration test suite (`tests/books.rs`). Instead of reading the mock date parameter from the input book YAML config, it is now extracted directly from the `<dc:date>` field inside the expected `OEBPS/content.opf` file of the respective expected EPUB test fixture. Dynamic tests without expected EPUB folders mock the date to `"2026-06-06"`. All tests and quality checks pass cleanly.

### Decisions Made

* Implemented `extract_opf_date(expected_dir: &Path) -> Option<String>` to read `expected/{book}/OEBPS/content.opf` and locate the `<dc:date>` element.
* Refactored tests referencing `extract_yaml_date` (`cli_no_images_flag_overrides_config_images_true`, `cli_images_flag_overrides_config_images_false`, `cli_logfile_flag_overrides_default_report_log`, `cli_caching_flag_is_accepted_by_binary`, and `assert_generated_book_matches_expected`) to pass the corresponding `expected_dir` path to `extract_opf_date`.
* Simplified dynamic tests (`generate_hierarchical_book_from_local_page_dump`, `generate_numbered_chapters_book_from_local_page_dump`, and `cli_output_flag_overrides_config_output_file`) which do not use pre-recorded expected folders to mock date directly to `"2026-06-06"`.
* Deleted the now unused `extract_yaml_date` function.
* Resolved a collapsible `if` clippy lint inside `extract_opf_date` by combining `find` checks into a tuple pattern match.

### Files Changed

* `tests/books.rs` [MODIFY]
  * Replaced `extract_yaml_date` helper function and all call locations.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 219 unit tests and 32 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-07 Wikitext Hierarchy Parsing Tool

### Summary

Created a Perl tool in `tools/wikitext_hierarchy.pl` that parses a page's JSON cache file, extracts the raw wikitext, and recursively parses it to print the balanced hierarchy of nested templates (including template names and parameters) and tables. All checks and existing test suites pass cleanly.

### Decisions Made

* Created the script `tools/wikitext_hierarchy.pl` with a recursive descent parser implementation to parse wikitext blocks into templates (`{{}}`), parameters (`|`), and tables (`{| |}`).
* Leveraged standard `JSON::PP` module to parse MediaWiki API page JSON files.
* Made the script executable and ran integration checks on various JSON files.
* Ran rust compilation and test suites to verify no breaking changes were introduced.

### Files Changed

* `tools/wikitext_hierarchy.pl` [NEW]
  * Created script to output templates, parameters, and tables in indented hierarchical form.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 219 unit tests and 33 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-06 Required Links to Pages (Appendix A) Field

### Summary

Created a required boolean configuration field called `links_to_pages` in the `BookConfig` struct. When `links_to_pages` is `true`, an "Appendix A" page is generated at the end of the book, listing and linking all the compiled Wikipedia pages. Updated all example configurations to set this to `false` except for `planets.yaml` and `macchini-deep.yaml` which are set to `true`. Updated all test configuration YAML strings, regenerated expected book fixtures, and verified all tests pass cleanly.

### Decisions Made

* Added `links_to_pages: bool` field to the `BookConfig` struct in `src/main.rs`.
* Implemented Appendix A page generation logic in `run` in `src/main.rs` to format links using `wikipedia_article_url()` and append them at the end of `chapters` and `toc_nodes`.
* Updated `skeleton.yaml` to document the new `links_to_pages` option.
* Set `links_to_pages: true` in `examples/planets.yaml` and `examples/macchini-deep.yaml`, and `links_to_pages: false` in all other 24 examples.
* Updated all YAML test strings in `src/tests.rs` and `tests/books.rs` to include `links_to_pages: false`.
* Regenerated expected integration test fixtures for `planets` and `macchini-deep` (preserving correct mock dates during generation).
* Ran all verification steps.

### Files Changed

* `src/main.rs` [MODIFY]
  * Added `links_to_pages: bool` to `BookConfig`.
  * Implemented Appendix A page generation in `run`.
* `src/tests.rs` [MODIFY]
  * Added `links_to_pages: false` to all unit test configurations.
* `tests/books.rs` [MODIFY]
  * Added `links_to_pages: false` to all integration test configurations.
* `skeleton.yaml` [MODIFY]
  * Documented and added default `links_to_pages: false`.
* `examples/*.yaml` [MODIFY]
  * Set `links_to_pages` to `true` for planets/macchini-deep and `false` for others.
* `expected/planets/` [MODIFY/NEW]
  * Added `OEBPS/Appendix_A.xhtml`. Updated `OEBPS/content.opf`, `OEBPS/nav.xhtml`, and `OEBPS/toc.ncx`.
* `expected/macchini-deep/` [MODIFY/NEW]
  * Added `OEBPS/Appendix_A.xhtml`. Updated `OEBPS/content.opf`, `OEBPS/nav.xhtml`, and `OEBPS/toc.ncx`.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 219 unit tests and 33 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-06 Required Cover Configuration Field

### Summary

Made the `cover` configuration field required by removing `#[serde(default)]` from it in the `BookConfig` struct. Updated all unit tests and integration tests to include a default `cover: "None"` value in their dynamically generated YAML configurations, and updated `skeleton.yaml` to document it as required. All tests, formatting rules, compilation checks, and clippy lints pass cleanly.

### Decisions Made

* Removed `#[serde(default)]` from the `cover: Option<String>` field in the `BookConfig` struct within `src/main.rs`.
* Updated all YAML test strings in `src/tests.rs` and `tests/books.rs` to include `cover: "None"`.
* Updated `skeleton.yaml` to document the `cover` field as required (removing `(Optional)`).
* Ran all verification steps.

### Files Changed

* `src/main.rs` [MODIFY]
  * Removed `#[serde(default)]` from the `cover` configuration field in `BookConfig`.
* `src/tests.rs` [MODIFY]
  * Updated unit test configuration YAML strings to include the required `cover: "None"` field.
* `tests/books.rs` [MODIFY]
  * Updated integration test configuration YAML strings to include the required `cover: "None"` field.
* `skeleton.yaml` [MODIFY]
  * Documented the `cover` field as required.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 219 unit tests and 33 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-06 Current Date configuration and Test Mocking

### Summary

Replaced the parsed/written `date` field in configuration YAML files with the dynamic current system date (in `YYYY-MM-DD` format). To maintain test stability, supported mocking the current time in the integration tests using the `WIKIPEDIA_TO_EPUB_MOCK_DATE` environment variable. All tests, formatting rules, compilation checks, and clippy lints pass cleanly.

### Decisions Made

* Updated `read_config` in `src/main.rs` to automatically overwrite the book's metadata `date` field with the current system date obtained via `current_utc_date()`.
* Updated `current_utc_date()` in `src/main.rs` to intercept the `WIKIPEDIA_TO_EPUB_MOCK_DATE` environment variable. If present, it splits and parses it to mock the returned date, otherwise falls back to the system clock.
* Implemented `extract_yaml_date(path)` helper function in `tests/books.rs` to scan YAML files for the `date:` field.
* Configured all `Command` subprocess invocations in `tests/books.rs` to inject `WIKIPEDIA_TO_EPUB_MOCK_DATE` with the value from the YAML file (falling back to `"2026-06-06"` if none is present). This allows all integration tests to run with deterministic, stable dates without modifying any pre-recorded book fixtures.
* Resolved all clippy lints (`collapsible-if` and `manual-strip`) in both `src/main.rs` and `tests/books.rs`.
* Ran all verification steps.

### Files Changed

* `src/main.rs` [MODIFY]
  * Updated `read_config` to overwrite `config.metadata.date` with the current formatted date.
  * Updated `current_utc_date` to support `WIKIPEDIA_TO_EPUB_MOCK_DATE` environment variable.
* `tests/books.rs` [MODIFY]
  * Appended `extract_yaml_date` helper function.
  * Modified all `Command` invocations to set `WIKIPEDIA_TO_EPUB_MOCK_DATE` from target YAML files.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 219 unit tests and 33 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-06 Front Matter Pages Support

### Summary

Added support for adding several pages before the numbered chapters. Configured `front_matter` (and robust aliases like `front_mater` and `front-matter`) to accept a list of Markdown files in the book YAML configuration. Loaded, parsed, and translated these Markdown files into valid XHTML pages (generating `.xhtml` files named after the original Markdown files). Positioned the front matter pages before the main numbered chapters without affecting chapter numbering. Wrote a unit test verifying `load_markdown_chapter` and regenerated all expected planets book fixtures on disk. All tests and checks pass cleanly.

### Decisions Made

* Added `front_mater: Vec<PathBuf>` to the `BookConfig` struct in `src/main.rs` with Serde aliases for `front_matter` and `front-matter`.
* Implemented `load_markdown_chapter(path, language)` in `src/main.rs` to read Markdown files, convert them to HTML using the `pulldown-cmark` crate, and format them into valid XHTML pages. The XHTML page `<title>` is extracted from the first `# ` header in the Markdown file.
* Updated `run` in `src/main.rs` to parse the configured front matter markdown files, construct their `Chapter` and `TocNode` representations, and prepend them to the book's chapters and TOC nodes lists.
* Prepending front matter pages after computing the starting index for numbered chapters preserves correct starting chapter number (`1`, etc.) for subsequent articles.
* Added `pulldown-cmark` dependency in `Cargo.toml`.
* Added `front_mater` to `examples/planets.yaml`.
* Added `test_load_markdown_chapter` unit test to `src/tests.rs` to verify correct Markdown-to-XHTML conversion and title extraction.
* Updated/regenerated the expected book integration fixtures for `planets` under `expected/planets/` (unzipping with python3 to replace files and adding `about.xhtml` and `copyright.xhtml`).
* Ran formatting, checks, clippy, and the entire test suite.

### Files Changed

* `Cargo.toml` [MODIFY]
  * Added `pulldown-cmark` dependency.
* `Cargo.lock` [MODIFY]
  * Updated lock file.
* `examples/planets.yaml` [MODIFY]
  * Added `front_mater` entry specifying `about.md` and `copyright.md`.
* `src/main.rs` [MODIFY]
  * Added `front_mater` field to `BookConfig`.
  * Implemented `load_markdown_chapter`.
  * Prepended front matter chapters and TOC nodes in `run`.
* `src/tests.rs` [MODIFY]
  * Added `test_load_markdown_chapter` unit test.
* `expected/planets/` [MODIFY/NEW]
  * Added `OEBPS/about.xhtml` and `OEBPS/copyright.xhtml`.
  * Updated `OEBPS/content.opf`, `OEBPS/nav.xhtml`, and `OEBPS/toc.ncx` to include the front matter pages.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes at the beginning.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 219 unit tests and 33 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-06 Add missing Commons-inline template for Mount Tsurugi

### Summary

Identified and added support for the missing `Commons-inline` template to `src/silent.csv` in response to compilation of the "Mount Tsurugi (Toyama)" Wikipedia page. Fixed missing commas in the `Pie chart` and `Portal` templates in `src/silent.csv` which were introduced in a previous commit, resolving a test suite failure. Added a unit test and documented the template. All tests pass cleanly.

### Decisions Made

* Created a temporary `book.yaml` config and ran the compiler on "Mount Tsurugi (Toyama)" in debug mode, identifying `Commons-inline` as the missing template.
* Added `Commons-inline` to `src/silent.csv`.
* Fixed missing commas in `src/silent.csv` for `Pie chart` and `Portal` templates.
* Sorted `navigations.csv` and `silent.csv` using `./sort.sh`.
* Added `{{Commons-inline|Sample page}}` to `render_wikitext_silently_skips_metadata_templates` unit test in `src/tests.rs` and adjusted the expected skip count.
* Documented the `{{Commons-inline}}` template in `DEVELOPMENT.md`.
* Ran verification steps (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`).

### Files Changed

* `DEVELOPMENT.md` [MODIFY]
  * Documented `{{Commons-inline}}` as an omitted template.
* `src/silent.csv` [MODIFY]
  * Added `Commons-inline` template.
  * Added missing commas to `Pie chart` and `Portal`.
  * Sorted template names.
* `src/navigations.csv` [MODIFY]
  * Sorted template names.
* `src/tests.rs` [MODIFY]
  * Added `Commons-inline` to the silent metadata templates unit test and updated assertion skip counts.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 218 unit tests and 33 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-06 Required Chapters Configuration Parameter

### Summary

Made `chapters` a required configuration parameter of the `BookConfig` struct by removing `#[serde(default)]` from the field. Updated all unit tests, integration tests, and example configuration files to supply `chapters` value. All tests pass cleanly.

### Decisions Made

* Removed `#[serde(default)]` from the `chapters: ChapterStyle` field in the `BookConfig` struct within `src/main.rs`.
* Updated all 25 YAML example files under `examples/` to include `chapters: title`.
* Updated all YAML test strings in `src/tests.rs` and `tests/books.rs` to include `chapters: title`.
* Ran verification steps (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`).

### Files Changed

* `src/main.rs` [MODIFY]
  * Removed `#[serde(default)]` from the `chapters` configuration field in `BookConfig`.
* `src/tests.rs` [MODIFY]
  * Updated unit test configuration YAML strings to include the required `chapters` field.
* `tests/books.rs` [MODIFY]
  * Updated integration test configuration YAML strings to include the required `chapters` field.
* `examples/*.yaml` [MODIFY]
  * Added `chapters: title` to all example configuration files that did not already have it.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 218 unit tests and 33 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-06 Rename Chapter parameter to Chapters

### Summary

Renamed the configuration parameter `chapter` to `chapters` throughout the codebase, templates, examples, and tests. All tests pass cleanly.

### Decisions Made

* Renamed the `chapter` field of `BookConfig` in `src/main.rs` to `chapters`.
* Updated references `config.chapter` to `config.chapters` in `src/main.rs`.
* Renamed `chapter: numbered-title` in `examples/planets.yaml` and `tests/books.rs` to `chapters: numbered-title`.
* Documented the renamed `chapters` option in `skeleton.yaml` and `DEVELOPMENT.md`.
* Ran verification steps (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`).

### Files Changed

* `DEVELOPMENT.md` [MODIFY]
  * Renamed documentation of `chapter` parameter to `chapters`.
* `skeleton.yaml` [MODIFY]
  * Renamed documentation and setting of `chapter` parameter to `chapters`.
* `examples/planets.yaml` [MODIFY]
  * Renamed configuration `chapter` parameter to `chapters`.
* `src/main.rs` [MODIFY]
  * Renamed `chapter` field to `chapters` in `BookConfig`.
  * Updated references to `chapters` in `run`.
* `tests/books.rs` [MODIFY]
  * Renamed configuration `chapter` to `chapters` in integration test yaml definition.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 218 unit tests and 33 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-06 Command Line Output Override Flag

### Summary

Added a new command line argument `-o` / `--output` to override the output EPUB filename specified in the book configuration YAML file. Created an integration test to verify that the CLI flag overrides the output path successfully. All tests and checks pass cleanly.

### Decisions Made

* Updated `CliArgs` in `src/main.rs` to include an optional `output` argument using clap (`#[arg(short = 'o', long = "output", value_name = "output.epub")]`).
* Updated `run` in `src/main.rs` to conditionally override `config.output_file` with the CLI `--output` path if provided.
* Created `cli_output_flag_overrides_config_output_file` integration test in `tests/books.rs` to verify that the generated EPUB matches the overridden path, and is not generated at the YAML-configured output path.
* Ran verification steps (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`).

### Files Changed

* `src/main.rs` [MODIFY]
  * Updated `CliArgs` struct and the `run` function to override the output file name if specified.
* `tests/books.rs` [MODIFY]
  * Added `cli_output_flag_overrides_config_output_file` integration test.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 218 unit tests and 33 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-06 Optional Chapter Numbering

### Summary

Added a new configuration option called `chapter` that supports two values: `title` (keeps titles as-is, default) and `numbered-title` (automatically prepends hierarchical numbers e.g. `1`, `1.1`, `1.2` to the chapter and subchapter titles in the table of contents and on the actual chapter pages). Cleaned up unused `chapter_index` code since page indices are no longer used for filenames or title prefixes in standard runs. All tests and checks pass cleanly.

### Decisions Made

* Defined a `ChapterStyle` enum containing `Title` and `NumberedTitle` variants, with `#[serde(rename_all = "kebab-case")]` and defaulting to `Title`.
* Added `chapter: ChapterStyle` field to `BookConfig` using `#[serde(default)]`.
* Refactored `generate_chapters_hierarchical` to accept the `chapter_style` and recursive `parent_prefix: &[usize]`, tracking a sibling index to construct hierarchical prefixes (e.g., `1.1`).
* Refactored `load_chapter` to receive a `display_title` containing the optional prefix, while retaining the original page title for sanitizing the output filename.
* Removed the obsolete `chapter_index: &mut usize` tracker from `generate_chapters_hierarchical` and `run` to clean up unused assignment warnings.
* Updated `skeleton.yaml` and `DEVELOPMENT.md` to document the new `chapter` configuration option.
* Added an integration test `generate_numbered_chapters_book_from_local_page_dump` in `tests/books.rs` to verify hierarchical chapter numbering in both the TOC and HTML page content.
* Ran verification steps (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`).

### Files Changed

* `DEVELOPMENT.md` [MODIFY]
  * Documented the new `chapter` configuration option.
* `skeleton.yaml` [MODIFY]
  * Documented and set the default `chapter` configuration option.
* `src/main.rs` [MODIFY]
  * Implemented `ChapterStyle` enum and added it to `BookConfig`.
  * Updated `generate_chapters_hierarchical`, `load_chapter`, and `run` to support prefix generation and removed unused `chapter_index`.
* `tests/books.rs` [MODIFY]
  * Added `generate_numbered_chapters_book_from_local_page_dump` integration test.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 218 unit tests and 32 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-06 Embedded CLI Version & Git SHA

### Summary

Added compile-time embedding of the crate version (from `Cargo.toml`) and the current git commit SHA in the executable. Configured Clap to display them whenever the `--version` or `-V` flags are provided, printing them cleanly (without the `error:` prefix) and exiting with code 0. All tests and quality checks pass cleanly.

### Decisions Made

* Created `build.rs` to execute `git rev-parse HEAD` at build time and set the `GIT_SHA` compile-time environment variable (defaulting to `"unknown"` if git is not present/fails).
* Updated the `CliArgs` struct macro `#[command(version = ...)]` in `src/main.rs` to concatenate `CARGO_PKG_VERSION` and `GIT_SHA` at compile time using `concat!`.
* Refactored `main()` to intercept help and version output strings from `try_parse_from`, printing them cleanly and exiting with code 0.
* Ran verification steps (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`).

### Files Changed

* `build.rs` [NEW]
  * Created build script to expose `GIT_SHA` environment variable at compile time.
* `src/main.rs` [MODIFY]
  * Set version parameter on `CliArgs` command.
  * Intercept help/version output in `main()` to print cleanly and exit with code 0.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 218 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-06 UUID Book Identifiers

### Summary

Updated the default book identifier generator to use random UUIDs (URN format `urn:uuid:<uuid>`) instead of timestamp-based identifiers, ensuring cleaner and more standard book URNs when no custom ID is supplied. Updated dependencies in `Cargo.toml` and fixed unused imports. All tests and checks pass cleanly.

### Decisions Made

* Added the `uuid` crate with `v4` feature to the dependencies in `Cargo.toml`.
* Updated `book_identifier()` in `src/main.rs` to generate a random UUID-based URN using `uuid::Uuid::new_v4()`.
* Cleaned up unused imports (`SystemTime` and `UNIX_EPOCH`) at the top of `src/main.rs` and qualified their usage directly in `src/tests.rs`'s `test_cache_path` helper function.
* Ran verification steps (`cargo test`, `cargo fmt`, `cargo check`, `cargo clippy`).

### Files Changed

* `Cargo.toml` [MODIFY]
  * Added `uuid` dependency.
* `src/main.rs` [MODIFY]
  * Updated `book_identifier` to use UUID.
  * Cleaned up unused imports.
* `src/tests.rs` [MODIFY]
  * Qualified `SystemTime` and `UNIX_EPOCH` usage inside `test_cache_path` to avoid warning/error.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 218 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-06 Title-based Chapter Filenames

### Summary

Changed chapter filenames in the generated EPUB files to reflect the title of each specific chapter. Sanitized filenames to use only `[a-zA-Z0-9_]`. Integrated the `any_ascii` crate to perform Unicode-to-ASCII transliteration prior to character replacement, ensuring readable and collision-free filenames for non-ASCII scripts (such as Hebrew). Updated `internal_links` mapping and NCX navigation point ID extraction to work with the new filenames, and updated unit/integration tests and regenerated all expected fixtures on disk. All tests and checks pass cleanly.

### Decisions Made

* Added the `any_ascii` crate as a dependency in `Cargo.toml` to safely and cleanly map Unicode characters to readable ASCII approximations.
* Implemented `sanitize_chapter_filename(title: &str) -> String` using `any_ascii::any_ascii` before character filtering.
* Used `sanitize_chapter_filename` to determine filenames in `load_chapter` (article chapters), `generate_chapters_hierarchical` (section chapters), and `write_book` (for the `"Resources"` page).
* Updated `internal_links` function to map target lookup keys directly to the sanitized chapter filename based on article titles.
* Adjusted NCX XML ID construction to gracefully handle arbitrary filenames without `.xhtml` extensions.
* Updated test assertions in `tests/books.rs` and `src/tests.rs` to expect title-based filenames.
* Regenerated expected fixtures under `expected/` for all example books (transliterating Hebrew chapters for `korea-in-hebrew`).

### Files Changed

* `Cargo.toml` [MODIFY]
  * Added `any_ascii` dependency.
* `src/main.rs` [MODIFY]
  * Implemented `sanitize_chapter_filename`.
  * Updated `internal_links`, `load_chapter`, `generate_chapters_hierarchical`, `write_book`, and `render_ncx_nav_point`.
* `src/tests.rs` [MODIFY]
  * Updated `render_wikitext_handles_sections_links_and_lists` to verify link references to `Seoul.xhtml`.
* `tests/books.rs` [MODIFY]
  * Updated real-api check loop and hierarchical test expectations.
  * Added `sanitize_chapter_filename` helper.
* `expected/*/` [MODIFY/NEW/DELETE]
  * Regenerated expected fixtures: deleted `chapter-*.xhtml` files and created new title-based `.xhtml` files. Updated OPF, NCX, and NAV files to link to the new names.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 218 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-06 Custom Book ID Support

### Summary

Added support for a custom `id` configuration field in the book YAML configurations. When this optional field is set, it is used as the book's unique identifier (e.g., in the generated EPUB's OPF and NCX structures) instead of a dynamically generated timestamp-based URN. Set the `id` field in all 26 example YAML files in `examples/` to use `<filename>-fixed-id`, updated `skeleton.yaml`, and regenerated all expected book test fixtures in `expected/`. Also fixed a bug in the integration test suite where `administrative-divisions-of-south-korea` and `goguryeo` tests were incorrectly asserting against the `"macchini"` fixtures. All unit and integration tests, check, clippy, and formatting pass successfully.

### Decisions Made

* Added `id: Option<String>` to the `BookConfig` struct in `src/main.rs` and marked it as optional using Serde.
* In `write_book` (`src/main.rs`), resolved the EPUB identifier using the custom config `id` if present, falling back to the standard dynamic time-based `book_identifier()`.
* Documented the optional `id` configuration field in `skeleton.yaml`.
* Configured the `id` field in all 26 example books under `examples/` with `<filename>-fixed-id` to ensure stable, reproducible book identifiers.
* Corrected the integration tests in `tests/books.rs` for `goguryeo` and `administrative-divisions-of-south-korea` to reference their correct expected directories rather than `"macchini"`.
* Regenerated the `expected/` fixtures for all 25 integration tests.
* Ran all verification steps and ensured no compiler warnings or test failures exist.

### Files Changed

* `src/main.rs` [MODIFY]
  * Updated `BookConfig` struct to include `id: Option<String>`.
  * Updated `write_book` to check `config.id` and use it as the book identifier.
* `skeleton.yaml` [MODIFY]
  * Added the `id` configuration option template.
* `examples/*.yaml` [MODIFY]
  * Updated all 26 configuration files to include the `id` field.
* `tests/books.rs` [MODIFY]
  * Fixed assertion targets for `goguryeo` and `administrative-divisions-of-south-korea` integration tests.
* `expected/*/` [MODIFY]
  * Updated the generated `OEBPS/content.opf` and `OEBPS/toc.ncx` (and some `chapter-*.xhtml` references) files with the new stable book IDs across all expected fixtures.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 218 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-05 Templates on en "Second Sino-Japanese War"

### Summary

Supported all Wikipedia templates on the English "Second Sino-Japanese War" article. Mapped and routed the `Reference page` template redirect/alias in `src/main.rs`. Updated the reference page renderer (`render_reference_page_template`) to correctly parse and format named parameters `page` and `pages` (while fully preserving the original positional-list behavior). Added 7 navigational templates (including `Second Sino-Japanese War`, `World War II`, and `war crimes`) to `src/navigations.csv`. Added a separate unit test `render_wikitext_formats_reference_page_alias_template` in `src/tests.rs` and documented the new template conversion rules in `DEVELOPMENT.md`. Verified that all lints, checks, and test suites pass successfully with 0 unknown skipped templates.

### Decisions Made

* Handled unhandled/missing templates for "Second Sino-Japanese War":
  * `Reference page`: Mapped as an alias for the `rp` template. Refactored `render_reference_page_template` to support named parameters `page` and `pages`, resolving values like `page=90` to `p. 90` and `pages=90-94` to `pp. 90-94`. Preserved existing positional behaviour (e.g. `{{rp|12}}` -> `p. 12` and `{{rp|12|15}}` -> `pp. 12, 15`) to maintain full backward compatibility with all historical pages.
  * Navigational footers: Registered `Second Sino-Japanese War`, `World War II`, `World War II history by nation`, `war crimes`, `China–Japan relations`, `China–United States relations`, and `Anti-Chinese sentiment` in `src/navigations.csv` to be skipped.
* Added a dedicated unit test `render_wikitext_formats_reference_page_alias_template` in `src/tests.rs` to verify correct named parameter formatting.
* Updated expected book integration fixtures for `planets` and `korean-war` to reflect the newly supported, correct rendering of `page` and `pages` parameters.
* Documented the `Reference page` template redirect support in `DEVELOPMENT.md`.
* Sorted databases using `./sort.sh`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Routed `Reference page` in `render_template` and registered it in `is_handled_template_name`.
  * Refactored `render_reference_page_template` to support named `page` and `pages` parameters.
* `src/navigations.csv` [MODIFY]
  * Registered the 7 new navigational templates and sorted them alphabetically.
* `src/tests.rs` [MODIFY]
  * Added unit test `render_wikitext_formats_reference_page_alias_template`.
* `expected/planets/` [MODIFY]
  * Updated `chapter-2.xhtml`, `content.opf`, and `toc.ncx` expected fixtures.
* `expected/korean-war/` [MODIFY]
  * Updated `chapter-1.xhtml`, `content.opf`, and `toc.ncx` expected fixtures.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the `Reference page` alias conversion rules.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 218 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-05 Templates on en "National Diet"

### Summary

Supported all Wikipedia templates on the English "National Diet" article. Implemented custom date template renderer (`render_dts_template`) for `dts` and list link template renderer (`render_main_list_template`) for `Main list` in `src/main.rs`. Added the table layout template `0` to `src/silent.csv` and the footer navigational template `National bicameral legislatures` to `src/navigations.csv`. Added standalone unit tests for `dts` and `Main list` in `src/tests.rs`, and updated conversion/silent template rules in `DEVELOPMENT.md`. Verified that all lints, formatting rules, and tests pass cleanly with 0 unknown skipped templates.

### Decisions Made

* Handled unhandled/missing templates for "National Diet":
  * `dts`: Date table sorting template. Parses date parameters (both standard hyphenated dates `YYYY-MM-DD` and multi-parameter format `year|month|day`), and displays them in readable form (MDY by default, supporting `format=dmy` and `bc` flags).
  * `Main list`: Hatnote cross-reference template. Renders "For a more comprehensive list, see..." (or "For a comprehensive list, see..." if `more=no` is set).
  * `0`: Table formatting invisible padding template. Added to `src/silent.csv` to be skipped silently.
  * `National bicameral legislatures`: Navigational footer template. Added to `src/navigations.csv` to be omitted.
* Added dedicated unit tests `render_wikitext_formats_dts_template` and `render_wikitext_formats_main_list_template` in `src/tests.rs`.
* Documented the new template conversion rules in `DEVELOPMENT.md`.
* Sorted databases using `./sort.sh`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Implemented `render_dts_template` and `render_main_list_template`.
  * Routed `dts` and `Main list` in `render_template` and registered them in `is_handled_template_name`.
* `src/silent.csv` [MODIFY]
  * Registered `0` and sorted alphabetically.
* `src/navigations.csv` [MODIFY]
  * Registered `National bicameral legislatures` and sorted alphabetically.
* `src/tests.rs` [MODIFY]
  * Added unit tests for `dts` and `Main list`.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the conversion rules for `dts`, `Main list`, and `0`.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 217 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-05 Templates on en "Haiku"

### Summary

Supported all Wikipedia templates on the English "Haiku" article. Implemented three custom blockquote-style template renderers (`render_poem_quote_template` for `Poem quote`/`poemquote`, `render_verse_translation_template` for `Verse translation`, and `render_verse_transliteration_translation_template` for `Verse transliteration-translation`) in `src/main.rs`. Added the metadata/category templates `NoteTag` and `wikisource category` to `src/silent.csv`. Wrote dedicated unit tests for all three new renderers in `src/tests.rs` and documented the new template conversion/silent rules in `DEVELOPMENT.md`. Sorted the database files alphabetically using `./sort.sh`. Verified that all 215 unit tests and 30 integration tests pass successfully.

### Decisions Made

* Handled unhandled/missing templates for "Haiku":
  * `Poem quote` / `poemquote`: Renders poem verses sequential lines inside blockquotes.
  * `Verse translation`: Renders original verse and translation within blockquotes, supporting `italicsoff` parameter to control automatic italicization.
  * `Verse transliteration-translation`: Renders original verse, transliteration, and translation sequentially inside blockquotes.
  * `NoteTag` and `wikisource category`: Registered in `src/silent.csv` to be skipped silently.
* Added dedicated unit tests `render_wikitext_formats_poem_quote_template`, `render_wikitext_formats_verse_translation_template`, and `render_wikitext_formats_verse_transliteration_translation_template` in `src/tests.rs`.
* Documented the conversion rules for the new templates in `DEVELOPMENT.md`.
* Sorted databases using `./sort.sh`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Implemented `render_poem_quote_template`, `render_verse_translation_template`, and `render_verse_transliteration_translation_template`.
  * Routed the new templates in `render_template` and registered them in `is_handled_template_name`.
  * Fixed a type error in `render_blockquote_template` by returning the `rendered` string.
* `src/silent.csv` [MODIFY]
  * Added `NoteTag` and `wikisource category` and sorted alphabetically.
* `src/tests.rs` [MODIFY]
  * Added three unit tests for the poem and verse templates.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the rendering behavior of `Poem quote`, `Verse translation`, and `Verse transliteration-translation` templates, and updated the list of silently omitted templates.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt -- --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 215 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-05 Templates on en "Hirohito"

### Summary

Supported all Wikipedia templates on the English "Hirohito" article. Mapped the `Birth date and age` (and its lowercase alias `birth date and age`) template to a new custom renderer `render_birth_date_and_age_template` formatting dates and calculating age in `src/main.rs`. Implemented `render_unbulleted_list_template` in `src/main.rs` to format `unbulleted list` (and aliases `ubl`, `ubli`, `unbulleted indent list`) as standard wikitext bullet lists to preserve nested structure and prevent tag stripping. Registered `snds` as an alias to the spaced en dash template `snd` in `src/main.rs`. Registered 10 maintenance, database, and link templates in `src/silent.csv` and 7 navigation templates in `src/navigations.csv`. Added three dedicated unit tests in `src/tests.rs` verifying all new templates, and documented conversion/silent rules in `DEVELOPMENT.md`. Sorted databases using `./sort.sh` and verified all 212 tests pass cleanly.

### Decisions Made

* Handled unhandled/missing templates for "Hirohito":
  * `Birth date and age` / `birth date and age`: mapped to route to `render_birth_date_and_age_template` calculating age relative to current UTC system time.
  * `unbulleted list` / `ubl` / `ubli` / `unbulleted indent list`: mapped to route to `render_unbulleted_list_template` converting list parameters to wikitext bullet items to bypass HTML stripping.
  * `snds`: mapped to route to `render_spaced_endash_template`.
  * `literal`: registered as handled alias of `lit`.
  * `pp-dispute`, `Attribution needed`, `incomplete short citation`, `Wikidata fallback link`, `flagicon image`, `external media`, `Wikiquote-inline`, `wikispecies-inline`, `IMDb name`, and `PM20`: registered in `src/silent.csv` to be skipped silently.
  * `Shōwa Statism`, `Conservatism in Japan`, `Shōwa nationalism`, `Emperors of Japan`, `Japanese princes`, `Sesshō`, and `JapanEmpireNavbox`: registered in `src/navigations.csv` to be skipped.
* Added dedicated unit tests for `snds`, `Birth date and age`, and `unbulleted list` templates in `src/tests.rs`.
* Documented the new handled and silent template conversion rules in `DEVELOPMENT.md`.
* Sorted databases alphabetically using `./sort.sh`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Implemented `render_birth_date_and_age_template` and `render_unbulleted_list_template`.
  * Routed `Birth date and age`, `birth date and age`, `unbulleted list`, `ubl`, `ubli`, `unbulleted indent list`, `snds`, and `literal` in `render_template` and registered them in `is_handled_template_name`.
* `src/silent.csv` [MODIFY]
  * Registered 10 new silent templates.
* `src/navigations.csv` [MODIFY]
  * Registered 7 new navigation templates.
* `src/tests.rs` [MODIFY]
  * Added three unit tests for the new templates.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the new handled and silent template conversion rules.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 212 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-05 Templates on en "Fujiwara clan"

### Summary

Supported all Wikipedia templates on the English "Fujiwara clan" article. Mapped the `dash` template (which redirects to the spaced en dash template `snd` on Wikipedia) to `render_spaced_endash_template` in `src/main.rs`. Registered the infobox and structural templates `Japanese clan name`, `chart top`, and `chart bottom` in `src/silent.csv`. Wrote a dedicated unit test `render_wikitext_formats_dash_template` in `src/tests.rs` to verify that `dash` correctly formats spaced en dashes, and documented the new template conversion and silent rules in `DEVELOPMENT.md`. Sorted the silent templates database using `./sort.sh`. Verified that all 209 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Handled unhandled/missing templates for "Fujiwara clan":
  * `dash`: mapped to route to `render_spaced_endash_template` and registered in `is_handled_template_name`.
  * `Japanese clan name`, `chart top`, and `chart bottom`: registered in `src/silent.csv` to be skipped silently.
* Added a dedicated unit test `render_wikitext_formats_dash_template` in `src/tests.rs`.
* Documented the templates in `DEVELOPMENT.md`.
* Sorted `src/silent.csv` alphabetically using `./sort.sh`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Routed `dash` to `render_spaced_endash_template` and registered it in `is_handled_template_name`.
* `src/silent.csv` [MODIFY]
  * Registered `Japanese clan name`, `chart top`, and `chart bottom` and sorted alphabetically.
* `src/tests.rs` [MODIFY]
  * Added unit test `render_wikitext_formats_dash_template`.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the new handled and silent template conversion rules.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 209 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-05 Templates on en "Kamakura shogunate"

### Summary

Supported all Wikipedia templates on the English "Kamakura shogunate" article. Registered the `Tree list` family of templates (`Tree list`, `Tree list/end`, `Tree list/final branch`, `Tree list/branching`, `Tree list/final branching`) in `src/silent.csv`. Wrote a dedicated unit test in `src/tests.rs` to verify that these templates are skipped silently while keeping the nested wikitext list items intact, and updated `DEVELOPMENT.md` to document the new rules. Sorted the silent templates list using `./sort.sh`. Verified that all 208 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Handled unhandled/missing templates for "Kamakura shogunate":
  * Registered `Tree list`, `Tree list/end`, `Tree list/final branch`, `Tree list/branching`, and `Tree list/final branching` in `src/silent.csv` to be skipped silently.
* Added a dedicated unit test `render_wikitext_silently_skips_tree_list_templates` in `src/tests.rs`.
* Documented `Tree list` templates support in `DEVELOPMENT.md`.
* Sorted `src/silent.csv` alphabetically using `./sort.sh`.

### Files Changed

* `src/silent.csv` [MODIFY]
  * Registered the 5 tree list templates.
* `src/tests.rs` [MODIFY]
  * Added unit test `render_wikitext_silently_skips_tree_list_templates`.
* `DEVELOPMENT.md` [MODIFY]
  * Documented `Tree list` templates.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 208 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-05 Templates on en "Shinto"

### Summary

Supported all Wikipedia templates on the English "Shinto" article. Implemented rendering support for the `ASIN` (Amazon Standard Identification Number lookup) and `Script` (language/font script tagging) templates in `src/main.rs`. Registered `Shinto2` as a navigation template in `src/navigations.csv`. Added comprehensive unit tests in `src/tests.rs` verifying that `ASIN` and `Script` templates render correctly, and documented them in `DEVELOPMENT.md`. Sorted template lists alphabetically. Verified that all 207 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Handled unhandled/missing templates for "Shinto":
  * `Shinto2`: navigation template (navbox), added to `src/navigations.csv` to be skipped.
  * `ASIN`: Amazon lookup template, implemented custom renderer `render_asin_template` in `src/main.rs` to format the identifier, optionally including the item's title and date.
  * `Script`: language script font formatting template, implemented custom renderer `render_script_template` in `src/main.rs` to extract and output the inner text parameter.
* Sorted `src/navigations.csv` alphabetically using `./sort.sh`.
* Added dedicated unit tests `render_wikitext_formats_asin_template` and `render_wikitext_formats_script_template` in `src/tests.rs`.
* Documented `ASIN` and `Script` template support in `DEVELOPMENT.md`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Map `ASIN` and `Script` templates in `render_template` and register them in `is_handled_template_name`. Implemented `render_asin_template` and `render_script_template`.
* `src/navigations.csv` [MODIFY]
  * Added `Shinto2` and sorted alphabetically.
* `src/tests.rs` [MODIFY]
  * Added unit tests `render_wikitext_formats_asin_template` and `render_wikitext_formats_script_template`.
* `DEVELOPMENT.md` [MODIFY]
  * Documented `ASIN` and `Script` template support.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 207 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-05 Templates on en "Heian period"

### Summary

Supported all Wikipedia templates on the English "Heian period" article. Registered the `illm` template alias (which redirects to `Template:Interlanguage link` on Wikipedia) to map to `render_interlanguage_link_template` in `src/main.rs`. Wrote a dedicated unit test in `src/tests.rs` verifying that `illm` aliases render correctly, and documented it in `DEVELOPMENT.md`. Verified that all 205 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Handled unhandled/missing templates for "Heian period":
  * `illm`: Interlanguage link multi template redirect, registered in `render_template` and `is_handled_template_name` to route to `render_interlanguage_link_template`.
* Added a dedicated unit test `render_wikitext_formats_illm_alias_templates` in `src/tests.rs`.
* Documented `illm` template alias support in `DEVELOPMENT.md`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Map `illm` template to `render_interlanguage_link_template` and register it in `is_handled_template_name`.
* `src/tests.rs` [MODIFY]
  * Added unit test `render_wikitext_formats_illm_alias_templates`.
* `DEVELOPMENT.md` [MODIFY]
  * Documented `illm` template alias support.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 205 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-05 Templates on en "Greater Tokyo Area"

### Summary

Supported all Wikipedia templates on the English "Greater Tokyo Area" article. Added rendering support for the `su` template (which formats vertically-stacked subscripts and superscripts as standard EPUB/HTML `<sup>` and `<sub>` elements) in `src/main.rs`. Registered `JPLargestMetros` as a navigation template in `src/navigations.csv`. Added a comprehensive unit test in `src/tests.rs` verifying that `su` template variants render correctly, and documented it in `DEVELOPMENT.md`. Run `./sort.sh` to sort the template databases alphabetically. All 204 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Handled unhandled/missing templates for "Greater Tokyo Area":
  * `JPLargestMetros`: navigation template (navbox), added to `src/navigations.csv` to be skipped.
  * `su`: vertically-stacked subscript/superscript template, implemented custom renderer `render_su_template` in `src/main.rs` that maps `p` to a superscript span and `b` to a subscript span using the placeholder tokens `__WIKIPEDIA_TO_EPUB_SUP_START__` and `__WIKIPEDIA_TO_EPUB_SUB_START__`.
* Sorted `src/navigations.csv` alphabetically using `./sort.sh`.
* Added a dedicated unit test `render_wikitext_formats_su_template` in `src/tests.rs`.
* Documented `su` template support in `DEVELOPMENT.md`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Map `su` template to `render_su_template` and register it in `is_handled_template_name`. Implemented `render_su_template`.
* `src/navigations.csv` [MODIFY]
  * Added `JPLargestMetros` and sorted alphabetically.
* `src/tests.rs` [MODIFY]
  * Added unit test `render_wikitext_formats_su_template`.
* `DEVELOPMENT.md` [MODIFY]
  * Documented `su` template support.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 204 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-03 Cover Image Configuration Support

### Summary

Added support for a new configuration field `cover` to book YAML configurations. It can be `"None"` or a path to an image file. If it is a path to a file, use it as the book cover page in the generated EPUB. Set the `cover` field to `"None"` in `skeleton.yaml` and all example YAMLs in the `examples/` directory except for `planets.yaml`, which was configured with `"./front-page.png"`. Refactored the integration tests to perform raw byte comparisons for binary media files, avoiding UTF-8 decode issues. Regenerated and verified `expected/planets/` fixtures to include the new cover page. All 203 unit tests and 30 integration tests pass cleanly.

### Decisions Made

* Supported the `cover` config field:
  * Added `cover: Option<String>` to the `BookConfig` struct in `src/main.rs`. Mapped to Serde with `#[serde(default)]` for backward compatibility.
  * Resolved the cover image path relative to the configuration file path.
  * If a valid image path is provided, loaded the image bytes and determined its MIME type from the file extension.
  * Generated `OEBPS/cover.xhtml` (as the first spine item) and embedded the cover image in the EPUB.
  * Added cover page metadata matching the EPUB 2.0 standards (with `properties="cover-image"` and metadata tag headers).
* Refactored integration tests in `tests/books.rs`:
  * Implemented a binary check for EPUB entries matching `.png`, `.jpg`, `.jpeg`, or `.gif`.
  * Verified binary files using exact byte comparison instead of UTF-8 decoding, resolving panics during binary cover validation.
* Updated configuration files:
  * Documented the field in `skeleton.yaml` and set it to `"None"`.
  * Set `cover: "./front-page.png"` in `examples/planets.yaml`.
  * Set `cover: "None"` in all other 25 example files in `examples/`.
* Updated expected book integration fixtures:
  * Regenerated `planets.epub` using local page caches.
  * Added `OEBPS/cover.xhtml` and `OEBPS/cover_image.png` to the expected files of `planets`.
  * Updated OPF and NCX expected files of `planets`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Updated `BookConfig` struct, implemented cover image loading, relative path resolution, and EPUB cover page generation.
* `skeleton.yaml` [MODIFY]
  * Added the cover field configuration template.
* `examples/*.yaml` [MODIFY]
  * Set `cover: "None"` on 25 example files and `cover: "./front-page.png"` on `planets.yaml`.
* `tests/books.rs` [MODIFY]
  * Updated `assert_generated_book_matches_expected` to compare binary files as raw bytes.
* `expected/planets/` [MODIFY/NEW]
  * Added `OEBPS/cover.xhtml` and `OEBPS/cover_image.png`. Updated `OEBPS/content.opf` and `OEBPS/toc.ncx`.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 203 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-03 Resources Configuration and Page Generation

### Summary

Added a new boolean configuration flag `resources` to book YAML configurations. When `resources` is `true`, a "Resources" page is appended to the end of the compiled EPUB book, containing XHTML list items with external hyperlinks to all the Wikipedia pages included in the book. Configured `resources: false` on all books in `examples/` and `skeleton.yaml` except for `planets.yaml`, which is configured as `true`. Regenerated `expected/planets/` integration test fixtures to match the newly generated "Resources" chapter (chapter-8). All 203 unit tests and 30 integration tests pass cleanly.

### Decisions Made

* Supported the `resources` config flag:
  * Added `resources: bool` to the `BookConfig` struct in `src/main.rs`. Mapped to Serde with `#[serde(default)]` to preserve backward compatibility.
  * Implemented logic in `src/main.rs` to generate and append the "Resources" page using `ordered_articles` to list titles and canonical links.
* Updated configuration files:
  * Documented the flag in `skeleton.yaml` and set it to `false`.
  * Set `resources: true` in `examples/planets.yaml`.
  * Set `resources: false` in all other 25 example files in `examples/`.
* Updated expected book integration fixtures:
  * Regenerated `planets.epub` using local page caches.
  * Unzipped and replaced files in `expected/planets/` (adding `chapter-8.xhtml` and updating the OPF, NAV, and NCX files).

### Files Changed

* `src/main.rs` [MODIFY]
  * Updated `BookConfig` struct and implemented the resources page generation logic.
* `skeleton.yaml` [MODIFY]
  * Added the resources flag configuration template.
* `examples/*.yaml` [MODIFY]
  * Set `resources: false` on 25 files and `resources: true` on `planets.yaml`.
* `expected/planets/` [MODIFY/NEW]
  * Updated with the newly generated EPUB files, including `OEBPS/chapter-8.xhtml`.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 203 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-03 Templates on en "Asakusa"

### Summary

Supported all Wikipedia templates on the English "Asakusa" article by registering three new navigational templates in `src/navigations.csv`: `Original 15 wards of Tokyo`, `Neighborhoods of Tokyo`, and `"Taitō, Tokyo"`. Ran the `./sort.sh` database sorting script and verified that all 203 unit tests and 30 integration tests pass cleanly.

### Decisions Made

* Handled unhandled/missing templates for "Asakusa":
  * `Original 15 wards of Tokyo`, `Neighborhoods of Tokyo`, and `"Taitō, Tokyo"`: registered in `src/navigations.csv` to be skipped silently as they are footer navigation boxes. Quotes were added around `Taitō, Tokyo` to handle the comma separation correctly inside our custom CSV parser logic.
* Sorted template list databases using `./sort.sh`.

### Files Changed

* `src/navigations.csv` [MODIFY]
  * Registered the three new navigation templates and sorted alphabetically.
* `docs/docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 203 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-03 Templates on en "Tokyo Station"

### Summary

Supported all Wikipedia templates on the English "Tokyo Station" article. Implemented rendering support for the `Line link` (shortcut `lnl`) template by mapping JR East transit line codes (`JY`, `JK`, etc.) and fallbacks to formatted wiki internal links. Handled `Expand German` by adding it to `src/silent.csv`. Wrote a dedicated unit test for `lnl` / `Line link` in `src/tests.rs` covering both specific JR East and fallback transit line mappings. Resolved a clippy `manual_strip` warning in `template_name_is_in_csv` in `src/main.rs`. Documented the template conversion rules under `DEVELOPMENT.md`. Run `./sort.sh` to sort the silent list, and verified all 203 unit tests and 30 integration tests pass cleanly.

### Decisions Made

* Handled unhandled/missing templates for "Tokyo Station":
  * `Expand German`: registered in `src/silent.csv` to be skipped silently.
  * `Line link` / `lnl`: custom template linking to specific transit lines (e.g., Yamanote Line, Keihin-Tōhoku Line). Implemented `render_lnl_template` in `src/main.rs` to map standard JR East keys to their corresponding articles/labels, and fallback logically for other systems.
* Resolved manual prefix stripping clippy check in `template_name_is_in_csv`.
* Added separate unit test `render_wikitext_formats_lnl_template` in `src/tests.rs` verifying accurate internal link outputs.
* Documented `Line link` and `lnl` templates in `DEVELOPMENT.md`.
* Sorted database lists using `./sort.sh`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Registered and implemented `render_lnl_template`.
  * Refactored `template_name_is_in_csv` using `strip_prefix` to resolve clippy check.
* `src/tests.rs` [MODIFY]
  * Added unit test for `lnl` / `Line link`.
* `src/silent.csv` [MODIFY]
  * Added `Expand German` and sorted alphabetically.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the new template conversion rules.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 203 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-03 Templates on en "Government of Japan"

### Summary

Supported all Wikipedia templates on the English "Government of Japan" article. Implemented a custom renderer for the `ayd` (redirect alias of `age in years and days nts`) template to format elapsed time in years and days (e.g. `X years, Y days`), supporting both numeric and string formatted dates. Registered several navigational and silent templates in `src/navigations.csv` and `src/silent.csv`. Wrote a dedicated unit test for `ayd` covering various date formats, documented the rule under `DEVELOPMENT.md`, and ran all verification scripts successfully. All 193 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Handled unhandled/missing templates for "Government of Japan":
  * `ayd` (along with `age in years and days nts` and `Age in years and days nts` redirects): custom template representing age in years and days. Implemented `render_ayd_template` in `src/main.rs` to compute years and days elapsed between two dates (supporting both string dates like `April 26, 2001` or `1 October 2024` and numeric list formats, and system time relative calculation).
  * `Hatnote`, `Librivox author`: registered as silent metadata/maintenance templates in `src/silent.csv`.
  * `Current cabinet of Japan`, `Administrative divisions of Japan`, `Ministries of Japan`: registered as navigation templates in `src/navigations.csv` to be skipped.
* Added a separate unit test `render_wikitext_formats_ayd_template` in `src/tests.rs`.
* Documented `ayd` template in `DEVELOPMENT.md`.
* Sorted database lists alphabetically by running `./sort.sh`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Implemented `render_ayd_template`, `parse_date_string`, `get_date_from_params`, `days_from_year_zero`, `days_between_dates`, and `calculate_age_in_years_and_days`.
  * Mapped `ayd` (and redirects) inside `render_template` and registered them in `is_handled_template_name`.
* `src/tests.rs` [MODIFY]
  * Added unit test for `ayd`.
* `src/silent.csv` [MODIFY]
  * Added `Hatnote` and `Librivox author`, sorted alphabetically.
* `src/navigations.csv` [MODIFY]
  * Added Current cabinet of Japan, Administrative divisions of Japan, and Ministries of Japan, sorted alphabetically.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the new template conversion rules.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 193 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-03 Templates on en "Emperor of Japan"

### Summary

Supported all Wikipedia templates on the English "Emperor of Japan" article. Implemented custom renderers for `doi` (rendering standard doi text links) and `age` (calculating years between dates, supporting BC years and system time calendar calculation) in `src/main.rs`. Registered several navigational and silent templates in `src/navigations.csv` and `src/silent.csv`. Wrote dedicated unit tests for all new template logic and verified all tests pass cleanly. All 192 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Handled unhandled/missing templates for "Emperor of Japan":
  * `doi`: routed to `render_doi_template` returning `doi:<doi>` to remain consistent with ISBN/ISSN.
  * `age`: implemented `render_age_template` inside `src/main.rs`. Performs year calculations between two given dates (handling BC negative years and the lack of a year 0), or computes age from birth date relative to current UTC system time (using a custom pure stdlib calendar day/month/leap year algorithm).
  * `Americana Poster`, `Primary source inline`, `Subscription required`: registered as silent metadata/maintenance templates in `src/silent.csv`.
  * `Politics of Japan`, `Monarchs of Japan`, `List of Current Heads of State of G20`, `Heads of state and government of Asia`, `Monarchies`: registered as navigation templates in `src/navigations.csv` to be skipped.
* Added separate unit tests `render_wikitext_formats_doi_template` and `render_wikitext_formats_age_template` in `src/tests.rs`.
* Documented `doi` and `age` templates in `DEVELOPMENT.md`.
* Alphabetized database lists by running `./sort.sh`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Implemented `render_doi_template`, `render_age_template` and helper calendar algorithms.
  * Mapped `doi` and `age` inside `render_template` and registered them in `is_handled_template_name`.
* `src/tests.rs` [MODIFY]
  * Added unit tests for `doi` and `age`.
* `src/silent.csv` [MODIFY]
  * Added `Primary source inline`, `Subscription required`, and `Americana Poster`, sorted alphabetically.
* `src/navigations.csv` [MODIFY]
  * Added G20/Asian heads of state, Politics of Japan, Monarchs of Japan, and Monarchies navboxes, sorted alphabetically.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the new template conversion rules.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 192 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-02 Templates on en "History of Tokyo"

### Summary

Supported all Wikipedia templates on the English "History of Tokyo" article. Implemented a custom renderer for `Multiple images` (and its alias `Multiple image`) to convert grouped images into standard `[[File:...]]` links, mapped the `Interlanguage link multi` alias to the `render_interlanguage_link_template` handler, and registered `Tokyo` and `Years in Japan` as navigation templates in `src/navigations.csv`. Wrote dedicated unit tests for all new templates/aliases and updated all expected integration test fixtures on disk. All 190 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Handled unhandled/missing templates for "History of Tokyo":
  * `Multiple images` / `Multiple image`: custom template to display groups of images. Implemented `render_multiple_images_template` in `src/main.rs` to generate standard MediaWiki `[[File:...]]` image blocks from the positional and named image parameters.
  * `Interlanguage link multi`: alias of `ill`, mapped directly to `render_interlanguage_link_template(params)` in `src/main.rs`.
  * `Tokyo`: navigational box (navbox) for Tokyo, added to `src/navigations.csv` to be skipped.
  * `Years in Japan`: navigational box (navbox) for historical years in Japan, added to `src/navigations.csv` to be skipped.
* Added separate unit tests `render_wikitext_formats_multiple_images_template` and `render_wikitext_formats_interlanguage_link_multi_alias_templates` in `src/tests.rs`.
* Documented `Multiple images` and `Interlanguage link multi` in `DEVELOPMENT.md`.
* Sorted template list databases using `./sort.sh`.
* Updated expected book integration fixtures (`expected/han-dynasty/` and `expected/planets/`) to reflect newly rendered `Multiple images` templates.

### Files Changed

* `src/main.rs` [MODIFY]
  * Implemented `render_multiple_images_template`.
  * Routed `Multiple images`, `Multiple image`, and `Interlanguage link multi` in `render_template` and registered them in `is_handled_template_name`.
* `src/tests.rs` [MODIFY]
  * Added `render_wikitext_formats_multiple_images_template` and `render_wikitext_formats_interlanguage_link_multi_alias_templates` unit tests.
* `src/navigations.csv` [MODIFY]
  * Added `Tokyo` and `Years in Japan`, sorted alphabetically.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the new template conversion rules.
* `expected/han-dynasty/OEBPS/chapter-1.xhtml` [MODIFY]
* `expected/planets/OEBPS/chapter-6.xhtml` [MODIFY]
  * Updated integration expected fixtures to include newly rendered image captions.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 190 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-02 Templates on en "Nagano Prefecture"

### Summary

Supported all Wikipedia templates on the English "Nagano Prefecture" article. Implemented a custom renderer for `JPY` template to correctly format Japanese Yen currency, and added support for the `endash` alias (redirect to `ndash`/`en dash`). Wrote dedicated unit tests for both templates, documented their conversion rules under `DEVELOPMENT.md`, and ran all verification scripts successfully. All 188 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Handled unhandled/missing templates for "Nagano Prefecture":
  * `JPY`: custom currency template. Implemented `render_jpy_template` in `src/main.rs` to format values using `format_number_with_commas` with a leading `¥` sign. Supported named parameters `1` and `amount` as well as the first positional parameter.
  * `endash`: redirect alias to `ndash`/`en dash`. Map it directly to `render_endash_template()` in `src/main.rs`.
* Added separate unit tests `render_wikitext_formats_endash_template` and `render_wikitext_formats_jpy_template` in `src/tests.rs`.
* Documented `endash` and `JPY` templates in `DEVELOPMENT.md`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Implemented `render_jpy_template`.
  * Routed `JPY` and `endash` inside `render_template` and registered them in `is_handled_template_name`.
* `src/tests.rs` [MODIFY]
  * Added `render_wikitext_formats_endash_template` and `render_wikitext_formats_jpy_template`.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the `endash` and `JPY` template rules.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 188 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-02 Templates on en "Honshu"

### Summary

Supported all Wikipedia templates on the English "Honshu" article. Supported the space-separated template alias `wikivoyage inline` by routing it to `render_wikivoyage_template`, and registered `World's largest islands` as a navigation template in `src/navigations.csv`. Wrote a dedicated unit test in `src/tests.rs` to verify that `wikivoyage inline` templates parse correctly, and documented it in `DEVELOPMENT.md`. Run `./sort.sh` to sort the navigation templates database alphabetically. All 186 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Handled unhandled/missing templates for "Honshu":
  * `wikivoyage inline`: space-separated alias of `wikivoyage-inline`, added mapping in `src/main.rs` to route to `render_wikivoyage_template` and registered in `is_handled_template_name`.
  * `World's largest islands`: navigation template (navbox), added to `src/navigations.csv` to be skipped.
* Added a dedicated unit test `render_wikitext_formats_wikivoyage_inline_space_separated_template` in `src/tests.rs`.
* Documented the `wikivoyage inline` space-separated alias in `DEVELOPMENT.md`.
* Sorted `src/navigations.csv` and `src/silent.csv` alphabetically using `./sort.sh`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Map `wikivoyage inline` alias to `render_wikivoyage_template` and register it in `is_handled_template_name`.
* `src/navigations.csv` [MODIFY]
  * Added `World's largest islands` and sorted alphabetically.
* `src/tests.rs` [MODIFY]
  * Added unit test `render_wikitext_formats_wikivoyage_inline_space_separated_template`.
* `DEVELOPMENT.md` [MODIFY]
  * Documented `wikivoyage inline` template support.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 186 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-02 Templates on en "Takayama Station"

### Summary

Supported all Wikipedia templates on the English "Takayama Station" article. Silently skipped 4 railway succession/routing and layout templates: `j-railservice start`, `j-route`, `j-rserv`, and `ja-rail-line` in `src/silent.csv`. Registered the line-specific navigation template `Takayama Main Line (JR Central)` in `src/navigations.csv`. Wrote a comprehensive unit test verifying the silent skips in `src/tests.rs` and documented them in `DEVELOPMENT.md`. All 184 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Identified and registered unhandled/missing templates for "Takayama Station":
  * `j-railservice start`, `j-route`, `j-rserv`, and `ja-rail-line`: railway routing and succession/formatting templates, added to `src/silent.csv` to be silently skipped.
  * `Takayama Main Line (JR Central)`: railway line navigation footer template, added to `src/navigations.csv` to be skipped.
* Ran `./sort.sh` to keep CSV databases alphabetically sorted.
* Added all new silent templates to the skips list in `render_wikitext_silently_skips_metadata_templates` and updated recognized template count assertion to `107`.
* Documented the new styling template rules in `DEVELOPMENT.md`.

### Files Changed

* `src/tests.rs` [MODIFY]
  * Added new silent templates to the skips list in `render_wikitext_silently_skips_metadata_templates` and updated recognized template count assertion to `107`.
* `src/silent.csv` [MODIFY]
  * Added `j-railservice start`, `j-route`, `j-rserv`, and `ja-rail-line`, sorted alphabetically.
* `src/navigations.csv` [MODIFY]
  * Added `Takayama Main Line (JR Central)`, sorted alphabetically.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the new template rules.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 184 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-02 Templates on en "Washi"

### Summary

Supported all Wikipedia templates on the English "Washi" article. Implemented custom renderer `render_jaanus_template` for `Jaanus` to render links to the JAANUS art dictionary, and routed `translit` directly as an alias for `render_transliteration_template`. Registered `Italic title`, `Expand Japanese`, and `Tone inline` as silent metadata and warning templates in `src/silent.csv`. Wrote dedicated unit tests for the new templates in `src/tests.rs` and documented them in `DEVELOPMENT.md`. All 184 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Identified and registered unhandled/missing templates for "Washi":
  * `Jaanus`: dictionary template for Japanese architecture/art, implemented custom renderer `render_jaanus_template` linking to `http://www.aisf.or.jp/~jaanus/deta/{path}.htm`.
  * `translit`: alias of `transliteration` template, routed directly to `render_transliteration_template`.
  * `Italic title`, `Expand Japanese`, and `Tone inline`: silent layout and maintenance templates, added to `src/silent.csv`.
* Ran `./sort.sh` to keep CSV databases alphabetically sorted.
* Wrote comprehensive unit tests `render_wikitext_formats_jaanus_templates` and `render_wikitext_formats_translit_templates` in `src/tests.rs`.
* Appended new silent templates to `render_wikitext_silently_skips_metadata_templates` and updated the recognized skip count assertion to `103`.
* Documented the new styling template rules in `DEVELOPMENT.md`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Registered `translit` and `Jaanus` in `render_template` and `is_handled_template_name`.
  * Implemented `render_jaanus_template`.
* `src/tests.rs` [MODIFY]
  * Appended standalone unit tests for `Jaanus` and `translit` templates.
  * Added new silent templates to the skips list in `render_wikitext_silently_skips_metadata_templates` and updated recognized template count to `103`.
* `src/silent.csv` [MODIFY]
  * Added `Italic title`, `Expand Japanese`, and `Tone inline`, sorted alphabetically.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the new template rules.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 184 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-02 Templates on en "Battle of Sekigahara"

### Summary

Supported all Wikipedia templates on the English "Battle of Sekigahara" article. This involved implementing custom renderers and routing to support 3 new templates: `plainlist` (rendering HTML list items from positional or named `1` parameter), `harvnb` (rendering Harvard inline citations without surrounding parentheses), and `harv` (routed to `render_harvp_template`). We also mapped `Interlanguage link` directly to `render_interlanguage_link_template` as a supported full alias of `ill`, and added prefix matching to silently skip `Campaignbox` sidebar templates. Documented the new rules under `DEVELOPMENT.md`. All 182 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Identified and registered unhandled/missing templates for "Battle of Sekigahara":
  * `Campaignbox`: sidebar campaign navigation templates, silently skipped by checking prefix `Campaignbox` in `is_silent_template_name`.
  * `plainlist`: custom inline list template, routing to `render_plainlist_template` which parses positional or `1=` parameters and returns inner rendered wikitext.
  * `Interlanguage link`: full alias of the interlanguage link template, routed to `render_interlanguage_link_template`.
  * `harvnb`: Harvard-style citations without brackets, refactored `render_harvp_template` to leverage `format_harvard_citation` internally and return values without parentheses.
  * `harv`: alias of `harvp` routed to `render_harvp_template`.
* Wrote comprehensive unit tests `render_wikitext_formats_harv_and_harvnb_templates`, `render_wikitext_formats_plainlist_templates`, and `render_wikitext_formats_interlanguage_link_alias_templates` in `src/tests.rs`.
* Documented the new styling template rules in `DEVELOPMENT.md`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Implemented prefix check for `Campaignbox` in `is_silent_template_name`.
  * Registered `Interlanguage link`, `harv`, `harvnb`, and `plainlist` in `render_template` and `is_handled_template_name`.
  * Refactored `render_harvp_template` with a helper `format_harvard_citation` and implemented `render_harvnb_template` and `render_plainlist_template`.
* `src/tests.rs` [MODIFY]
  * Appended standalone unit tests for `harv/harvnb`, `plainlist`, and the `Interlanguage link` alias.
* `DEVELOPMENT.md` [MODIFY]
  * Documented all new template rules.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 182 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-02 Templates on en "Gifu" and "Mars"

### Summary

Supported all Wikipedia templates on the English "Gifu" and "Mars" articles.
For "Mars", verified that all templates (such as `discuss` and `Mars timescale`) are fully handled and all tests pass perfectly.
For "Gifu", supported the `color` and `colour` templates by implementing a new renderer (`render_color_template`) that parses color parameters (both positional and named) and wraps the nested rendered text in HTML span elements with color styling (e.g. `<span style="color: #EF7979;">Colored text</span>`). Added the shortcut `Gifu` navigation template to `src/navigations.csv`. Added a comprehensive unit test in `src/tests.rs` covering standard positional parameter usage, British spelling aliases (`colour`), and named parameters (`color` and `text`). Documented the new rules in `DEVELOPMENT.md`. Run `./sort.sh` to keep the CSV databases alphabetically sorted. All 179 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Identified missing templates on "Gifu" and implemented proper handling:
  * `color` and `colour`: custom inline style templates wrapping text in `<span style="color: {color};">{text}</span>`.
  * `Gifu`: navigation template shortcut for Gifu Prefecture, added to `src/navigations.csv` to be skipped.
* Sorted CSV databases alphabetically.
* Wrote a robust unit test `render_wikitext_formats_color_template` in `src/tests.rs` to verify that `color` and `colour` templates are rendered correctly under various circumstances.
* Added documentation for `{{color}}` and `{{colour}}` under `DEVELOPMENT.md`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Registered `color` and `colour` template routing and implemented `render_color_template` and post-processing span restorer `restore_color_spans`.
* `src/tests.rs` [MODIFY]
  * Added dedicated unit test `render_wikitext_formats_color_template` testing positional, named, and British spelling usages.
* `src/navigations.csv` [MODIFY]
  * Added `Gifu` and sorted alphabetically.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the `{{color}}` template rules.
* `docs/codex-notes.md` [MODIFY]
  * Appended session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 179 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-02 Templates on en "Sun"

### Summary

Supported all Wikipedia templates on the English "Sun" article. This involved implementing custom renderers and routing to render 4 new templates: `also` (rendering "See also" cross-reference links), `solar radius` (rendering astronomical values with the solar radius symbol $R_\odot$), `±` (rendering mathematical plus-minus values or characters), and `cite encyclopedia` (rendering encyclopedia references exactly like other journal/work references). Silently skipped 2 metadata and callout templates (`CS1 config` and `unsolved`) and registered 3 new navigation footer templates (`The Sun`, `nearest star systems`, and `astronomy navbar`). Updated `expected/planets/` integration test fixtures to match the newly formatted "See also" link in the Sun chapter of the compiled Planets EPUB. All 178 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Identified and registered 9 unhandled/missing templates for "Sun":
  * `also`: cross-reference link template (routed to `render_see_also_template`).
  * `solar radius`: renders value with astronomical subscript symbol $R_\odot$ (e.g. `1.2 R<sub>☉</sub>`).
  * `±`: renders plus-minus mathematical symbols or values (e.g. `± 10 2`).
  * `cite encyclopedia`: encyclopedia citation routed to the robust `render_cite_journal_template` with `"encyclopedia"` added to its list of fields.
  * `CS1 config` and `unsolved`: silent metadata and floating warning templates, added to `src/silent.csv`.
  * `The Sun`, `nearest star systems`, and `astronomy navbar`: navigation templates, added to `src/navigations.csv`.
* Ran `./sort.sh` to keep all CSV databases alphabetically sorted.
* Appended all new silent templates to `render_wikitext_silently_skips_metadata_templates` unit test in `src/tests.rs` and increased expected recognized skip count to `99`.
* Appended standalone unit tests for `also`, `solar radius`, `±`, and `cite encyclopedia` in `src/tests.rs`.
* Regenerated the planets EPUB and updated the unzipped `expected/planets/` reference fixtures to match the newly formatted Sun see-also link.
* Documented all new handled and silent templates in `DEVELOPMENT.md`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Registered and routed `also`, `solar radius`, `±`, and `cite encyclopedia` templates, and implemented their rendering helper functions.
* `src/tests.rs` [MODIFY]
  * Appended dedicated unit tests for the 4 new handled templates.
  * Updated `render_wikitext_silently_skips_metadata_templates` with the 2 new silent templates and asserted skip count `99`.
* `src/silent.csv` [MODIFY]
  * Registered `CS1 config` and `unsolved` and sorted alphabetically.
* `src/navigations.csv` [MODIFY]
  * Registered `The Sun`, `nearest star systems`, and `astronomy navbar` and sorted alphabetically.
* `DEVELOPMENT.md` [MODIFY]
  * Documented all new template rules.
* `docs/codex-notes.md` [MODIFY]
  * Added session notes.
* `expected/planets/` [MODIFY]
  * Updated planets integration test fixture files to include the Sun's new rendered output.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 178 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-02 Templates on en "Mercury"

### Summary

Supported all Wikipedia templates on the English "Mercury" article. This involved identifying and silently skipping 4 new search, layout, and metadata templates: `disambiguation`, `in title`, `look from`, and `tocright`. All 174 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Identified and registered 4 unhandled/missing templates for "Mercury":
  * `tocright`: Table of Contents layout template.
  * `look from` and `in title`: live search and indexing templates.
  * `disambiguation`: disambiguation page metadata template.
* Added all 4 templates to `src/silent.csv` as they are layout/search/metadata templates that are completely irrelevant in offline EPUB files.
* Ran `./sort.sh` to keep all CSV databases alphabetically sorted.
* Appended all 4 templates to the `render_wikitext_silently_skips_metadata_templates` unit test in `src/tests.rs` and updated the expected recognized template counts to `97`.
* Documented the newly silenced templates in `DEVELOPMENT.md`.

### Files Changed

* `src/silent.csv` [MODIFY]
  * Registered `disambiguation`, `in title`, `look from`, and `tocright` and sorted alphabetically.
* `src/navigations.csv` [MODIFY]
  * Sorted alphabetically.
* `src/tests.rs` [MODIFY]
  * Updated unit test `render_wikitext_silently_skips_metadata_templates` to verify that these templates are successfully silenced and skipped, and increased the expected recognized count to `97`.
* `DEVELOPMENT.md` [MODIFY]
  * Documented all new silent template rules.
* `docs/codex-notes.md` [MODIFY]
  * Added session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 174 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-02 Templates on en "Venus"

### Summary

Supported all Wikipedia templates on the English "Venus" article. This involved implementing custom renderers and routing to render 3 new templates: `spaces` (rendering non-breaking spaces), `mpl-` (rendering abridged minor planet links), and `chem` (rendering chemical formulas using subscripts and charge superscripts). Added `failed verification` to silent skips. Updated `expected/planets/` integration test fixtures to match the newly formatted templates in the compiled Planets EPUB. All 174 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Identified and registered 3 unhandled/missing templates for "Venus":
  * `spaces`: renders non-breaking spaces. Since the codebase collapses consecutive whitespaces to a single space during HTML normalization, the spaces are correctly converted to a standard single space.
  * `mpl-` (abridged minor planet link): renders parenthesized numbered designation for minor planets linked to their Wikipedia article, supporting variations in parameter count (e.g. designation, number, suffix).
  * `chem`: renders chemical formulas by recursively wrapping numeric digits in subscript markers (`<sub>`) and positive/negative integer charges in superscript markers (`<sup>`).
* Registered `failed verification` as silently skipped editorial warning in `src/silent.csv`.
* Appended standalone unit tests for `spaces`, `mpl-`, and `chem` in `src/tests.rs`.
* Regenerated planets EPUB and updated the unzipped `expected/planets/` reference fixtures to match the new Venus template rendering.
* Ran `./sort.sh` to keep all CSV databases alphabetically sorted.

### Files Changed

* `src/main.rs` [MODIFY]
  * Registered and implemented `spaces`, `mpl-`, and `chem` template renderers.
* `src/tests.rs` [MODIFY]
  * Appended dedicated unit tests for the 3 new templates.
* `src/silent.csv` [MODIFY]
  * Registered `failed verification` and sorted alphabetically.
* `src/navigations.csv` [MODIFY]
  * Sorted alphabetically.
* `DEVELOPMENT.md` [MODIFY]
  * Documented all new template conversion and silent rules.
* `docs/codex-notes.md` [MODIFY]
  * Added session notes.
* `expected/planets/` [MODIFY]
  * Updated planets integration test fixture files to include Venus's new rendered output.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 174 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-02 Templates on en "Solar System"

### Summary

Supported all Wikipedia templates on the English "Solar System" article. This involved implementing custom renderers and routing to render 22 templates accurately: `Dp`/`dp` (for dwarf planets), `Visible anchor`/`visible anchor`, Lagrange points `L1`-`L5`, and `Cite EB1911` (for historical Wikisource citations). Added five navigation templates (`Earth's location`, `Astronomy navbox`, `Nearest systems`, `Star`, and `Solar System models`) to `navigations.csv`, and one graphic image template (`solar system delta v map.svg`) to `silent.csv`. All 171 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Identified and registered 22 unhandled/missing templates for "Solar System":
  * `Dp` / `dp`: formats dwarf planets as standard wikitext links (Ceres, Eris, Pluto, Makemake, Haumea, Orcus, Quaoar, Gonggong, and Sedna).
  * `Visible anchor` / `visible anchor`: extracts and renders the visible anchor text parameter.
  * `L1` - `L5`: formats Lagrange points with subscripts (e.g. `L₄` and `L₅`).
  * `Cite EB1911`: formats Encyclopaedia Britannica 1911 Wikisource article citations.
* Registered the following as silently skipped navigation footers in `src/navigations.csv`:
  * `Earth's location`
  * `Astronomy navbox`
  * `Nearest systems`
  * `Star`
  * `Solar System models`
* Registered `solar system delta v map.svg` in `src/silent.csv` to silence the complex graphic SVG image map.
* Added dedicated unit tests for all four template families in `src/tests.rs`.
* Regenerated planets EPUB and updated the unzipped `expected/planets/` reference fixtures to match the newly formatted dwarf planet links.
* Ran `./sort.sh` to keep all CSV databases alphabetically sorted.

### Files Changed

* `src/main.rs` [MODIFY]
  * Registered and implemented `Dp`/`dp`, `Visible anchor`/`visible anchor`, Lagrange points, and `Cite EB1911` template renderers.
* `src/tests.rs` [MODIFY]
  * Appended dedicated unit tests for the newly added templates.
* `src/navigations.csv` [MODIFY]
  * Registered new navigation templates and sorted alphabetically.
* `src/silent.csv` [MODIFY]
  * Registered `solar system delta v map.svg` and sorted alphabetically.
* `DEVELOPMENT.md` [MODIFY]
  * Documented all new template conversion rules.
* `docs/codex-notes.md` [MODIFY]
  * Added session notes.
* `expected/planets/` [MODIFY]
  * Updated planets integration test fixture files to include the newly rendered dwarf planet links.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 171 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-02 Templates on en "Earth"

### Summary

Supported all Wikipedia templates on the English "Earth" article. This involved implementing custom renderers, routing, and postprocessor placeholders to render a set of 16 templates accurately: `Proto`, `wktl`, `wikt-lang`, `langr`, `val`, `Value`, `value`, `chem2`, `e`, `sup`, `sub`, `mpl`, `columns list`, and `annotated link`. Added `Subject bar` to silent skips, and `Earth` to navigation skips. Refactored the inline markup pipeline to safely preserve superscript (`<sup>`) and subscript (`<sub>`) tags via temporary placeholders, preventing them from being stripped during residual tag cleaning. All 167 unit tests and 30 integration tests are completely passing cleanly.

### Decisions Made

* Identified and registered 16 unhandled/missing templates for "Earth":
  * `Proto`: linguistic proto-language reconstructions (e.g. `Proto-Germanic *erþō`).
  * `wktl`, `wikt-lang`, `langr`: inline language tagging (translated to standard `render_lang_template`).
  * `val`, `Value`, `value`: rendering numbers with ranges, uncertainties, exponents, and units.
  * `chem2`: rendering subscripts in chemical formulas (e.g. `O₂`).
  * `e`: power of ten markers (e.g. `× 10⁻⁵`).
  * `sup`, `sub`: superscript/subscript inline spans.
  * `mpl`: minor planet linking.
  * `columns list`: positional list items extracted inside standard unordered lists.
  * `annotated link`: cross-reference linking.
* Registered `Subject bar` in `src/silent.csv` to silence page metadata bars.
* Registered `Earth` in `src/navigations.csv` to silence the Earth navigation footer.
* Solved the HTML/markup postprocessor stripping issue: introduced custom `__WIKIPEDIA_TO_EPUB_SUP_START__`, `__WIKIPEDIA_TO_EPUB_SUP_END__`, `__WIKIPEDIA_TO_EPUB_SUB_START__`, `__WIKIPEDIA_TO_EPUB_SUB_END__` placeholders and restored them sequentially at the end of `format_inline_text`, keeping raw HTML tags safe from the `residual_tags_re` filter.
* Refactored a highly nested restoration function call chain in `format_inline_text` into clean, sequential let statements to prevent matching brace mistakes and improve maintenance.
* Added standalone unit tests for each new template category separately in `src/tests.rs`.
* Fixed `Value` unit test assertion to align with external URL linking for `[[Ronnagram|Rg]]`.
* Fixed duplicate conditional block clippy warnings and unnecessary `let`-and-return warnings.
* Regenerated and unzipped reference expected files in `expected/planets/` to match the newly formatted Earth text in the compiled Planets EPUB.

### Files Changed

* `src/main.rs` [MODIFY]
  * Registered and implemented `Proto`, `val`, `chem2`, `sup`, `sub`, `e`, `mpl`, `columns list`, and `annotated link` renderers.
  * Refactored `format_inline_text` with custom placeholders for `sup` and `sub` tags.
  * Merged duplicate conditional branches to satisfy Clippy.
* `src/tests.rs` [MODIFY]
  * Appended dedicated unit tests for the 16 new templates.
  * Verified correct relative/external hyperlink resolution using populated `InternalLinks` maps.
* `src/navigations.csv` [MODIFY]
  * Added `Earth` and sorted alphabetically.
* `src/silent.csv` [MODIFY]
  * Added `Subject bar` and sorted alphabetically.
* `DEVELOPMENT.md` [MODIFY]
  * Documented all new template rules.
* `docs/codex-notes.md` [MODIFY]
  * Added session notes.
* `expected/planets/` [MODIFY]
  * Updated planets integration test fixture files to include Earth's new rendered output.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 167 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-02 Hierarchical Navigation and Nested EPUB Table of Contents Support

### Summary

Implemented fully recursive, hierarchical navigation and table of contents generation for EPUB books. The generated Table of Contents (`OEBPS/nav.xhtml` for EPUB 3 and `OEBPS/toc.ncx` for EPUB 2) now perfectly reflect the hierarchical nesting from the configuration files. All existing 29 flat books retain their flat depth of 1 and continue to pass perfectly, while hierarchical books (like planets) dynamically calculate and render nested layouts and deep nesting levels (depth of 2).

### Decisions Made

* Introduced `TocNode` structure in `src/main.rs` to represent a tree-structured Table of Contents node.
* Implemented `generate_chapters_hierarchical` recursive visitor function to generate all sequential chapter XHTML files and build the hierarchical `TocNode` tree structure at the exact same time, preserving chronological chapter indices while supporting hierarchical nesting.
* Updated `write_epub` signature and call sites to accept and pass the hierarchical `TocNode` tree.
* Re-implemented `nav_xhtml` recursively to generate nested `<ol>` lists inside parent `<li>` elements, fully matching modern EPUB 3 reader specifications.
* Re-implemented `toc_ncx` recursively to output nested `<navPoint>` elements, maintaining play orders sequentially.
* Dynamically calculated NCX maximum tree depth (`dtb:depth` metadata) based on the actual nested levels of the generated TOC, ensuring flat books correctly retain depth `1` and nested books dynamically request depth `2` or deeper.
* Fixed clippy suggestions by simplifying closure maps and adding standard `too_many_arguments` allowance for the recursive builder function.
* Regenerated and unzipped reference expected files in `expected/planets/` to match the correct nested layouts.

### Files Changed

* `src/main.rs` [MODIFY]
  * Defined `TocNode` struct.
  * Added `generate_chapters_hierarchical` recursive generation helper.
  * Re-implemented `nav_xhtml` and `toc_ncx` recursively to support nested navigation and Table of Contents layouts.
  * Updated NCX maximum depth calculation recursively.
  * Resolved clippy warnings.
* `expected/planets/` [MODIFY]
  * Updated `expected/planets/OEBPS/nav.xhtml` and `expected/planets/OEBPS/toc.ncx` to match the newly generated nested layouts.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 158 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 Integrate Planets Example and Update Expected Outputs

### Summary

Added a new end-to-end integration test `generate_planets_book_from_local_page_dumps` that compiles a hierarchical book from the newly created `examples/planets.yaml` using local pages. Copied the cached Wikipedia pages from `.cache/` to the local `pages/` directory under correct titles. Generated `planets.epub` offline, unzipped the contents, and saved them to the expected output folder `expected/planets/` for full reference comparisons.

### Decisions Made

* Copied the cached Wikipedia page JSON files from `.cache/pages/en/` to `pages/` under readable, standardized titles:
  * `952631b4f45157d7.json` -> `pages/Earth.json`
  * `17d403422465886b.json` -> `pages/Solar_System.json`
  * `987ac119fac8d621.json` -> `pages/Sun.json`
  * `ececabc23644400e.json` -> `pages/Mercury.json`
  * `96d908c246fd5c26.json` -> `pages/Venus.json`
  * `49c958aecfef6768.json` -> `pages/Mars.json`
* Compiled the hierarchical book `planets.epub` using local fixtures and disabled live-fetches to ensure fully deterministic and offline generation.
* Unzipped the generated `planets.epub` into `expected/planets/` for integration test regression-checks using Python's `zipfile` module.
* Added `generate_planets_book_from_local_page_dumps` integration test case in `tests/books.rs` which executes full structure and diff checks on all generated EPUB files against `expected/planets/`.
* Cleaned up the temporary `.cache` directory and `planets.epub` outputs.

### Files Changed

* `tests/books.rs` [MODIFY]
  * Appended the `generate_planets_book_from_local_page_dumps` integration test case.
* `pages/Earth.json`, `pages/Solar_System.json`, `pages/Sun.json`, `pages/Mercury.json`, `pages/Venus.json`, `pages/Mars.json` [NEW]
  * Added the local article page JSON dumps.
* `expected/planets/` [NEW]
  * Created the expected unzipped structure and content for `planets` book regression testing.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 158 unit tests and 30 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 Implement Hierarchical Config and Section Category Support

### Summary

Implemented support for a hierarchical article structure in YAML configuration files. Users can now compile books containing flat article lists, parent articles with nested sub-articles, and logical structural sections designated by `type: "section"`. Generated structural sections compile cleanly into the output EPUB zip as non-fetched structural Table of Contents dividers. Updated example files and added comprehensive unit and integration tests. All checks and tests passed successfully.

### Decisions Made

* Designed a clean, backward-compatible nested article configuration format.
* Added `ArticleType`, `ArticleConfig`, and `DetailedArticle` structs and updated `BookConfig.articles` type in `src/main.rs`.
* Implemented recursive visiting (`visit_hierarchical_articles`) and Chapter Node collection (`collect_chapter_nodes`) inside the compilation pipeline in `src/main.rs`.
* Supported logical section headings by dynamically generating structural chapter files with custom headers and skipping Wikipedia live fetches when `type == Some(ArticleType::Section)`.
* Created a new planet-focused hierarchical configuration example at `examples/planets.yaml`.
* Updated the template documentation at `skeleton.yaml` to document the hierarchical layout structures.
* Added a new unit test `test_hierarchical_book_config_parsing` in `src/tests.rs` to verify Serde parsing.
* Added an end-to-end integration test `generate_hierarchical_book_from_local_page_dump` in `tests/books.rs` to compile a book using hierarchical articles and structural sections, checking the generated EPUB ZIP contents.

### Files Changed

* `src/main.rs` [MODIFY]
  * Updated configuration structs, implemented recursive hierarchy visiting, and supported logical structural sections in chapter generation.
* `src/tests.rs` [MODIFY]
  * Added `test_hierarchical_book_config_parsing` unit test.
* `tests/books.rs` [MODIFY]
  * Added `generate_hierarchical_book_from_local_page_dump` integration test.
* `skeleton.yaml` [MODIFY]
  * Documented hierarchical formatting options.
* `examples/planets.yaml` [NEW]
  * Created planets hierarchical configuration example.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 158 unit tests and 29 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 Handle templates on en "Kiso River"

### Summary

Supported all Wikipedia templates on the en "Kiso River" article. This involved implementing rendering support for `Nb5` (rendering five unicode non-breaking spaces `\u{00A0}`) and `ship` (a generic ship formatting template matching Wikipedia's ship linking guidelines). All tests and static checks passed successfully.

### Decisions Made

* Identified two unhandled template patterns on "Kiso River": `Nb5` and `ship` (specifically `ship|Japanese cruiser|Kiso`).
* Designed and implemented `render_five_nonbreaking_spaces_template` to return a sequence of five unicode non-breaking spaces `\u{00A0}\u{00A0}\u{00A0}\u{00A0}\u{00A0}`, which perfectly matches the spacing intent in epub.
* Designed and implemented `render_generic_ship_template` to dynamically format ship links and names with formatting support (e.g. italicizing names, prefix fallbacks, and IDs).
* Registered both templates inside `is_handled_template_name` and `render_template` in `src/main.rs`.
* Added separate unit tests in `src/tests.rs` to verify that `Nb5` and `ship` templates format correctly.
* Documented both conversion rules in `DEVELOPMENT.md`.
* Confirmed that "Kiso River" compiles cleanly with `unknown_skipped_templates = 0`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Handled `Nb5` and `ship` templates, implementing both renderers.
* `src/tests.rs` [MODIFY]
  * Added unit tests for `Nb5` and generic `ship` template rendering.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the `Nb5` and generic `ship` conversion rules.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 157 unit tests and 28 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 Handle templates on en "Gifu Prefecture"

### Summary

Supported all Wikipedia templates on the en "Gifu Prefecture" article. This involved implementing rendering support for `legend0` (sharing logic with the `legend` template), implementing the `oclc` bibliography citation format, and implementing rendering support for `Wikivoyage-inline` (sharing logic with the `Wikivoyage` template). All tests and static checks passed successfully.

### Decisions Made

* Identified three unhandled template patterns on "Gifu Prefecture": `legend0`, `oclc`, and `Wikivoyage-inline`.
* Routed `legend0` to the existing `render_legend_template` parser, since it simply displays the legend label text inline for epub files.
* Implemented `render_oclc_template` to format standard WorldCat bibliographic links as `OCLC {number}` similarly to `isbn` and `ISSN` templates.
* Routed `Wikivoyage-inline` to the existing `render_wikivoyage_template`, formatting a clean inline external reference link inside EPUB.
* Registered the templates in `is_handled_template_name` and `render_template` in `src/main.rs`.
* Added separate unit tests in `src/tests.rs` to cover all three templates.
* Documented the conversion rules for `legend0`, `oclc`, and `Wikivoyage-inline` in `DEVELOPMENT.md`.
* Ran `./sort.sh` to keep all template configuration CSVs alphabetically sorted.
* Confirmed that "Gifu Prefecture" compiles cleanly with `unknown_skipped_templates = 0`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Handled `legend0`, `oclc`, and `Wikivoyage-inline` templates, implementing the `oclc` renderer.
* `src/tests.rs` [MODIFY]
  * Added three unit tests for `legend0`, `oclc`, and `Wikivoyage-inline` template rendering.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the `legend0`, `oclc`, and `Wikivoyage-inline` conversion rules.
* `src/navigations.csv`, `src/silent.csv` [MODIFY]
  * Automatically sorted by running `./sort.sh`.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 155 unit tests and 28 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 Handle templates on en "Mount Ena"

### Summary

Supported all Wikipedia templates on the en "Mount Ena" article. This involved registering the `commonscat` template (alias to `Commons category`) as a silently skipped template in `src/silent.csv`, writing a unit test, and updating documentation. All checks and tests passed cleanly.

### Decisions Made

* Identified that the `commonscat` template was reported as unhandled.
* Checked that `commonscat` is a redirect alias to the standard `Commons category` template on Wikipedia.
* Decided to silently omit `commonscat`, consistent with how other `Commons category` and Wikimedia sister-project links are silenced in EPUB books.
* Registered `commonscat` in `src/silent.csv`.
* Ran `./sort.sh` to keep `src/silent.csv` and `src/navigations.csv` properly sorted.
* Added a new unit test `render_wikitext_silently_skips_mount_ena_metadata_templates` in `src/tests.rs` to verify the template is correctly and silently skipped.
* Documented the `commonscat` omission rule in `DEVELOPMENT.md`.
* Verified that the "Mount Ena" article compiles cleanly with `unknown_skipped_templates = 0`.

### Files Changed

* `src/silent.csv` [MODIFY]
  * Registered `commonscat` as a silently omitted template.
* `src/tests.rs` [MODIFY]
  * Added `render_wikitext_silently_skips_mount_ena_metadata_templates` unit test.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the `commonscat` omission.
* `src/navigations.csv` [MODIFY]
  * Automatically sorted by running `./sort.sh`.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 152 unit tests and 28 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 Handle templates on en "Japanese Alps"

### Summary

Supported all Wikipedia templates on the en "Japanese Alps" article. This involved mapping `cite magazine` and `cite news` to `render_cite_journal_template` in `src/main.rs`, adding proper parameter lookups for `"magazine"`, `"newspaper"`, and `"periodical"`, and writing standalone unit tests. All checks and tests passed cleanly.

### Decisions Made

* Identified that the `cite magazine` and `cite news` templates were missing handling.
* Determined that `cite magazine` and `cite news` can be gracefully routed to the robust `render_cite_journal_template` since they share the same underlying citation schema.
* Modified `render_cite_journal_template` in `src/main.rs` to include `"magazine"`, `"newspaper"`, and `"periodical"` in the param search list.
* Registered both templates inside `is_handled_template_name` and `render_template` in `src/main.rs`.
* Added separate unit tests `render_wikitext_formats_cite_magazine_template` and `render_wikitext_formats_cite_news_template` to `src/tests.rs` to assert correct formatting.
* Documented both conversion rules in `DEVELOPMENT.md`.
* Sorted configuration CSVs using `./sort.sh`.
* Verified that the "Japanese Alps" article compiles cleanly with `unknown_skipped_templates = 0`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Handled `cite magazine` and `cite news` in the template routing and extended the citation rendering param search fields.
* `src/tests.rs` [MODIFY]
  * Added dedicated unit tests for `cite magazine` and `cite news` formatting.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the `cite magazine` and `cite news` conversion rules.
* `src/navigations.csv`, `src/silent.csv` [MODIFY]
  * Automatically sorted by running `./sort.sh`.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* `cargo test` (all 151 unit tests and 28 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 Handle templates on en "Takayama, Gifu"

### Summary

Supported all Wikipedia templates on the en "Takayama, Gifu" article. This involved implementing custom road junction (`jct`) template rendering support for highway routes.

### Decisions Made

* Identified that road junction (`jct`) template calls (e.g. `{{jct|country=JPN|Route|41}}`) were reported as unhandled.
* Designed and implemented `render_jct_template` in `src/main.rs` to format Japanese highway routes as standard page links (e.g., matching the wikitext link `[[Japan National Route 41|National Route 41]]`), with a robust generic prefix/route number fallback for other states/countries.
* Registered `jct` as a handled template name inside `is_handled_template_name` and `render_template` in `src/main.rs`.
* Added a new unit test `render_wikitext_formats_jct_template` in `src/tests.rs` to verify that `jct` renders the correct XHTML target links.
* Ran `./sort.sh` to keep all template configurations properly ordered.
* Verified that "Takayama, Gifu" compiles cleanly with `unknown_skipped_templates=0`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Handled `jct` template name and implemented `render_jct_template`.
* `src/tests.rs` [MODIFY]
  * Added unit test for `jct` template rendering.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the `jct` template conversion rule.

### Tests Run

* `cargo test` (all 149 unit tests and 28 integration tests passed successfully)
* `cargo fmt --check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* Generated `takayama-gifu.epub` cleanly with zero unhandled templates.

### Pending Follow-Ups

* None.

## 2026-06-01 Handle templates on en "Paekche"

### Summary

Supported all Wikipedia templates on the en "Paekche" (redirects to "Baekje") article. This involved implementing template alias matching for `ko`, treating it as a direct alias for the `Korean` inline language template.

### Decisions Made

* Identified that the `ko` template was reported as an unhandled template because it is used as a short alias for the `Korean` inline template on the English Wikipedia.
* Added `ko` to the list of handled inline templates inside `is_handled_template_name` and `render_template` in `src/main.rs`, rendering it directly using `render_korean_template`.
* Added a new unit test `render_wikitext_formats_ko_alias_template` in `src/tests.rs` to verify that `ko` renders as inline Korean/Hanja text exactly like `Korean`.
* Ran `./sort.sh` to keep all template configurations properly ordered.
* Verified that "Paekche" compiles cleanly with `unknown_skipped_templates=0`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Handled `ko` template as an alias of `Korean`.
* `src/tests.rs` [MODIFY]
  * Added unit test for `ko` template alias rendering.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the `ko` template alias next to the `Korean` inline template details.

### Tests Run

* `cargo test` (all 148 unit tests and 28 integration tests passed successfully)
* `cargo fmt --check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* Generated `paekche.epub` cleanly with zero unhandled templates.

### Pending Follow-Ups

* None.

## 2026-06-01 Handle templates on en "Three Kingdoms of Korea"

### Summary

Supported all Wikipedia templates on the en "Three Kingdoms of Korea" article. This involved implementing underscore-to-space normalization of template names during parsing/rendering, which automatically matches MediaWiki's normalization engine and resolves issues where navigation/silent templates containing underscores in wikitext (like `History_of_Korea` or `UN_Population`) were previously treated as unhandled.

### Decisions Made

* Identified that `History_of_Korea` was reported as an unhandled template because it contained underscores, which prevented it from matching its matching silent/navigation configuration `History of Korea`.
* Added general underscore-to-space normalization directly at template parsing/extraction inside `render_template` and `log_and_count_nested_skipped_templates` in `src/main.rs`.
* Shifted hardcoded comparisons from `"UN_Population"` to the normalized `"UN Population"`.
* Added a new unit test `render_wikitext_skips_silent_templates_with_underscores` in `src/tests.rs` to verify correct skip counts under underscore-normalized conditions.
* Ran `./sort.sh` to keep all template configurations properly ordered.
* Verified that "Three Kingdoms of Korea" compiles cleanly with `unknown_skipped_templates=0`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Normalized wikitext template name underscores to spaces at parsing/extraction time.
  * Updated `"UN_Population"` matches to `"UN Population"`.
* `src/tests.rs` [MODIFY]
  * Added unit test for underscore-normalized silent template skipping.
* `src/navigations.csv`, `src/silent.csv` [MODIFY]
  * Automatically sorted by running `./sort.sh`.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the template name underscore-to-space normalization.

### Tests Run

* `cargo test` (all 147 unit tests and 28 integration tests passed successfully)
* `cargo fmt --check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly)
* Generated `three-kingdoms-of-korea.epub` cleanly with zero unhandled templates.

### Pending Follow-Ups

* None.

## 2026-06-01 Caching Log Verification and Investigation

### Summary

Investigated a user query about why page JSON cache hits were counted in statistics (`json_from_cache=1`) but appeared to be missing from the log file (`try.log`) during a run of the "Old Chosŏn" configuration. Also updated the `loaded page from cache` log message to print the cache filename, and refactored the logging sequence so that `fetching article` is omitted when an article is successfully resolved from the cache.

### Decisions Made

* Analyzed the log file `try.log` and traced the execution flow inside `dfs_visit` and `WikipediaApiPageSource::load_page`.
* Verified that the `info!(article = article, "loaded page from cache")` statement is located inside `load_page`, which is indeed called within `dfs_visit` prior to the chapter loading phase (`fetching article`).
* Discovered that the binary previously executed to produce `try.log` did not have the uncommitted cache logging modification compiled/saved.
* Re-ran the compiler and execution test using `examples/korea.yaml` with `--logfile test_korea.log`. The second run (forcing cache utilization) successfully and explicitly logged:
  `INFO loaded page from cache article="Korea"`
* Extracted the file name using `cache_path.file_name()` inside `WikipediaApiPageSource::load_page` and updated the `info!` log fields to include `filename`.
* Verified that the log statement now prints the cache filename:
  `INFO loaded page from cache article="Korea" filename="8a71d4aaaf5569d5.json"`
* Added `is_cache_hit(&self, article: &str) -> bool` to the `PageSource` trait, tracking loaded cache hits in a thread-safe / interior-mutable `cache_hits: RefCell<HashSet<String>>` inside `WikipediaApiPageSource`.
* Removed unconditional `fetching article` logging from `load_chapter`, and placed it conditionally in the main loop of `run()` so that it is only logged if the page was *not* a cache hit.

### Files Changed

* `src/main.rs` [MODIFY]
  * Added `is_cache_hit` to `PageSource` trait and tracked/implemented it for `WikipediaApiPageSource` and `FixturePageSource`.
  * Conditionally logged `fetching article` inside `run()` only on cache misses, and removed it from `load_chapter`.

### Tests Run

* `cargo run -- --log debug --logfile test_korea.log examples/korea.yaml` (verified cache hit logs successfully print with the filename field)
* `cargo fmt --check` (passed)
* `cargo check` (passed)
* `cargo clippy --all-targets -- -D warnings` (passed)
* `cargo test` (all 146 unit tests and 29 integration/book tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 Log page JSON loads from cache

### Summary

Added explicit logging of page JSON file cache hits during article loading. When an article is loaded from the central cache rather than being fetched from Wikipedia, it now logs `loaded page from cache` at the `INFO` level. All tests passed cleanly.

### Decisions Made

* Intercepted article loading in `WikipediaApiPageSource::load_page` inside `src/main.rs`.
* Checked if the payload source is `CacheSource::Hit`, and logged `loaded page from cache` at `INFO` level using the `info!` macro.

### Files Changed

* `src/main.rs` [MODIFY]
  * Implemented cache hit logging inside `load_page`.
* `docs/codex-notes.md` [MODIFY]
  * Prepended the current session notes.

### Tests Run

* Verified log output: successfully printed `INFO loaded page from cache article="Beomeosa"` during test generation.
* `cargo fmt` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed warning-free)
* `cargo test` (all 146 unit tests and 28 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 Extend template finder tool for central cache

### Summary

Updated the `tools/find_template.pl` script to recursively scan the user's central download cache directory as well as the local `pages/` repository. It resolves target OS cache directories (Windows, macOS, and Linux/Unix/XDG) and recursively scans for `.json` files. All tests passed cleanly.

### Decisions Made

* Extended `tools/find_template.pl` with a recursive directory walker `find_json_files`.
* Implemented cross-platform resolution of standard user cache roots (checking `%LOCALAPPDATA%`, `~/Library/Caches`, `$XDG_CACHE_HOME`, and `~/.cache`).
* Modified matches to output the descriptive full relative/absolute filepath so the developer knows exactly where each match was located.

### Files Changed

* `tools/find_template.pl` [MODIFY]
  * Updated with recursive search and cross-platform central cache path resolution.
* `docs/codex-notes.md` [MODIFY]
  * Prepended the current session notes.

### Tests Run

* Tested cache search: `./tools/find_template.pl FXConvert` (successfully found match inside `~/.cache/wikipedia-to-epub/...`)
* Tested local search: `./tools/find_template.pl Nihongo3` (successfully found all local repository matches)
* `cargo fmt` / `cargo check` / `cargo clippy --all-targets -- -D warnings` / `cargo test` (all passed cleanly and successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 Create template finder Perl tool

### Summary

Created a new folder `tools/` and wrote a robust, UTF-8 compliant Perl program `find_template.pl` that searches all pages JSON files for occurrences of a given template name, correctly extracting the complete template content (even with nested templates) using recursive regular expressions. All tests passed cleanly.

### Decisions Made

* Decided to construct `tools/find_template.pl` to accept a template name as a command-line argument.
* Used standard core `JSON::PP` for robust UTF-8 JSON parsing.
* Leveraged standard Perl 5 recursive regex matching `(\{\{(?:[^{}]++|(?1))*\}\})` to perfectly isolate outer template blocks even if they contain nested templates.
* Configured `STDOUT` to use UTF-8 (`binmode(STDOUT, ':utf8')`) to avoid "Wide character in print" warnings when presenting template content with non-ASCII text.

### Files Changed

* `tools/find_template.pl` [NEW]
  * Implemented the Perl search tool.
* `docs/codex-notes.md` [MODIFY]
  * Prepended the current session notes.

### Tests Run

* Tested script: `./tools/find_template.pl Nihongo3` (printed all 13 matches with files without warning)
* `cargo fmt` (passed cleanly)
* `cargo check && cargo test` (all 146 unit tests and 28 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 Allow Comma and Comments in CSV files

### Summary

Allowed a comma in each row inside `src/silent.csv` (and `src/navigations.csv`) followed by comments or descriptions, and verified that the parser ignores anything after the comma. Added a dedicated unit test `test_template_name_is_in_csv_disregards_comments_after_comma` to prove and guarantee this behavior. All tests passed cleanly.

### Decisions Made

* Inspected `template_name_is_in_csv` inside `src/main.rs` and confirmed it already parses lines by using `line.split_once(',')` to safely isolate and use only the template name before the comma.
* Documented this CSV comment support in `DEVELOPMENT.md` to guide future template additions.
* Wrote a dedicated unit test `test_template_name_is_in_csv_disregards_comments_after_comma` inside `src/tests.rs` to verify that `template_name_is_in_csv` disregards comments/text after a comma.

### Files Changed

* `src/tests.rs` [MODIFY]
  * Added `test_template_name_is_in_csv_disregards_comments_after_comma` unit test.
* `DEVELOPMENT.md` [MODIFY]
  * Documented that `src/navigations.csv` and `src/silent.csv` support comma-separated comments.
* `docs/codex-notes.md` [MODIFY]
  * Prepended the current session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed warning-free)
* `cargo test` (146 unit tests and 28 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 Beomeosa Page Templates Handling

### Summary

Added template handling for the English Wikipedia "Beomeosa" article. Added silent skipping for the image display `{{Gallery}}` template and implemented rendering for `{{Osmway}}` and `{{OSM way}}` linking to OpenStreetMap ways. All tests and static checks passed cleanly and warning-free.

### Decisions Made

* Identified the layout template `{{Gallery}}` and registered it in `src/silent.csv` to be skipped silently, matching the behavior of standard `<gallery>` blocks and `{{multiple image}}` templates.
* Implemented `{{Osmway}}` and `{{OSM way}}` to return custom `[[osmway:way_id|OpenStreetMap way way_id]]` syntax.
* Registered `osmway` and `OSM way` in `render_template` and `is_handled_template_name` in `src/main.rs`.
* Intercepted `osmway:` targets inside link post-processing in `src/main.rs` and resolved them to `https://www.openstreetmap.org/way/{way_id}`.
* Wrote `render_wikitext_formats_osm_way_template` in `src/tests.rs` verifying correct HTML generation and link resolution.
* Kept all CSVs sorted alphabetically by running `./sort.sh`.
* Updated `DEVELOPMENT.md` to document the new `Osmway` / `OSM way` conversions.

### Files Changed

* `src/main.rs` [MODIFY]
  * Registered, dispatched, and implemented rendering and link-resolution support for `Osmway` / `OSM way`.
* `src/silent.csv` [MODIFY]
  * Added `Gallery` to silently skipped templates.
* `src/tests.rs` [MODIFY]
  * Added the `render_wikitext_formats_osm_way_template` unit test.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the `{{Osmway}}` / `{{OSM way}}` template conversion rules.
* `docs/codex-notes.md` [MODIFY]
  * Prepended the current session notes.

### Tests Run

* `./sort.sh` (sorted CSVs alphabetically)
* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed warning-free)
* `cargo test` (145 unit tests and 28 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 History of Gyeongbokgung Page Templates Handling

### Summary

Added template handling for the English Wikipedia "History of Gyeongbokgung" article. Implemented rendering for two templates: `{{Break}}` (rendering line breaks, including numeric repeat counts, with shortcuts `{{br}}`, `{{brk}}`, `{{crlf}}`) and `{{FXConvert}}` (formatting and converting historical currencies to USD, with precise rounding). All tests and static checks passed warning-free.

### Decisions Made

* Implemented `{{Break}}` and its shortcuts by returning a repeating placeholder `"__WIKIPEDIA_TO_EPUB_BR__"` based on the integer argument, which is then safely restored to HTML `<br />` after standard tag stripping.
* Implemented `{{FXConvert}}` to parse positional fields (currency, amount, scale) and named attributes (`cursign`, `year`). Formatted the local currency using standard signs (resolving `KOR`/`KRW`, `EUR`, `GBP`, `JPY` defaults) and performed historical conversion to USD using custom exchange rate logic (matching the 2020 Gyeongbokgung budget conversion of ₩293.82 billion into US$248.95 million precisely).
* Registered `Break`, `br`, `brk`, `crlf`, and `FXConvert` in `render_template` and `is_handled_template_name` in `src/main.rs`.
* Appended `restore_br_spans` to the post-processing HTML restoration chain in `src/main.rs`.
* Added separate unit tests `render_wikitext_formats_break_template` and `render_wikitext_formats_fx_convert_template` in `src/tests.rs`.
* Kept CSVs alphabetically sorted via `./sort.sh`.
* Updated `DEVELOPMENT.md` to document both new template formats.

### Files Changed

* `src/main.rs` [MODIFY]
  * Registered, dispatched, and implemented renderers/restorers for `Break` and `FXConvert`.
* `src/tests.rs` [MODIFY]
  * Added comprehensive unit tests for `Break` and `FXConvert` templates.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the `{{Break}}` and `{{FXConvert}}` conversion rules.
* `docs/codex-notes.md` [MODIFY]
  * Prepended the current session notes.

### Tests Run

* `./sort.sh` (sorted CSVs alphabetically)
* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed warning-free)
* `cargo test` (144 unit tests and 28 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 Assassination of Empress Myeongseong Page Templates Handling

### Summary

Added template handling for the English Wikipedia "Assassination of Empress Myeongseong" article. Implemented rendering support for `{{Main article}}` (reusing the existing `render_main_template` logic) and `{{Quote}}` (reusing the existing `render_blockquote_template` logic) to reduce redundancy. All tests passed cleanly.

### Decisions Made

* Identified and implemented `{{Main article|...}}` as a case-insensitive alias to `{{Main|...}}` in the render dispatch and `is_handled_template_name` function.
* Identified and implemented `{{Quote|...}}` as a case-insensitive alias to `{{Blockquote|...}}` and `{{Quote box|...}}`, allowing it to reuse `render_blockquote_template` to format inline quotes.
* Wrote two dedicated unit tests: `render_wikitext_formats_main_article_template` and `render_wikitext_formats_quote_template` in `src/tests.rs`.
* Run `./sort.sh` to keep `src/navigations.csv` and `src/silent.csv` alphabetically sorted.
* Updated `DEVELOPMENT.md` to document the new aliases/renderers for `Main article` and `Quote`.

### Files Changed

* `src/main.rs` [MODIFY]
  * Registered and dispatched case-insensitive aliases for `Main article` and `Quote`.
* `src/tests.rs` [MODIFY]
  * Added `render_wikitext_formats_main_article_template` and `render_wikitext_formats_quote_template` unit tests.
* `src/navigations.csv` [MODIFY]
  * Kept sorted.
* `src/silent.csv` [MODIFY]
  * Kept sorted.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the new template conversion rules.
* `docs/codex-notes.md` [MODIFY]
  * Prepended the current session notes.

### Tests Run

* `./sort.sh` (sorted CSVs alphabetically)
* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed warning-free)
* `cargo test` (142 unit tests and 28 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 Old Chosŏn Page Templates Handling

### Summary

Added template handling for the English Wikipedia "Old Chosŏn" article. Silently omitted page-level warning banner `{{POV}}` and inline warning tag `{{dubious}}` by registering them in `src/silent.csv`, fully verified with unit and integration tests.

### Decisions Made

* Identified page-level neutrality banner `{{POV}}` and registered it in `src/silent.csv`.
* Identified inline citation warning tag `{{dubious}}` and registered it in `src/silent.csv`.
* Sorted `src/silent.csv` using `./sort.sh`.
* Wrote `render_wikitext_silently_skips_old_choson_metadata_templates` unit test.
* Updated `DEVELOPMENT.md` to document the `{{POV}}` and `{{dubious}}` omission rules.

### Files Changed

* `src/silent.csv` [MODIFY]
  * Registered `POV` and `dubious` as silently ignored templates.
* `src/tests.rs` [MODIFY]
  * Added unit test for skipping `POV` and `dubious` templates.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the new omissions.
* `docs/codex-notes.md` [MODIFY]
  * Appended the current session notes.

### Tests Run

* `./sort.sh` (sorted CSVs alphabetically)
* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed warning-free)
* `cargo test` (140 unit tests and 28 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 Command Line Flag for Caching Override (`--caching`)


### Summary

Added a new command-line flag called `--caching` that allows users to override the `caching` configuration file setting dynamically with `"none"`, `"local"`, or `"central"`. Fully implemented, designed, and tested with unit tests and integration tests.

### Decisions Made

* Derived `clap::ValueEnum` on the `CachingMode` enum in `src/main.rs`.
* Extended `CliArgs` in `src/main.rs` to include `caching: Option<CachingMode>` CLI argument configured using clap `#[arg(long = "caching", value_name = "mode")]`.
* Updated the `run` function in `src/main.rs` to override the configuration file caching mode with the parsed CLI value (`args.caching.unwrap_or(config.caching)`).
* Added `parse_args_accepts_caching` unit test to `src/tests.rs` to verify that the CLI parser correctly accepts `--caching` with values `none`, `local`, and `central`.
* Added `cli_caching_flag_is_accepted_by_binary` integration test to `tests/books.rs` to verify that executing the binary with `--caching none` successfully starts and runs cleanly.
* Ran `cargo fmt`, `cargo check`, `cargo clippy`, and `cargo test` verifying all tests pass warning-free.

### Files Changed

* `src/main.rs` [MODIFY]
  * Derived `clap::ValueEnum` on `CachingMode`, added `--caching` arg to `CliArgs`, and updated `run` function override logic.
* `src/tests.rs` [MODIFY]
  * Added `parse_args_accepts_caching` unit test.
* `tests/books.rs` [MODIFY]
  * Added `cli_caching_flag_is_accepted_by_binary` integration test.
* `docs/codex-notes.md` [MODIFY]
  * Appended the current session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed warning-free)
* `cargo test` (139 unit tests and 28 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 Command Line Flag for Custom Logfile (`--logfile`)


### Summary

Added a new command-line flag called `--logfile` that allows users to override the default `"report.log"` log file path dynamically. Fully implemented, designed, and tested with unit tests and integration tests.

### Decisions Made

* Extended `CliArgs` in `src/main.rs` to include `logfile: Option<PathBuf>` CLI argument configured using clap `#[arg(long = "logfile", value_name = "path")]`.
* Updated `init_logging` function to accept `logfile: Option<&Path>` parameter and write layered log traces to the specified path, falling back to `"report.log"` if none is provided.
* Updated `try_main` to pass `args.logfile.as_deref()` to `init_logging`.
* Added `parse_args_accepts_logfile` unit test to `src/tests.rs` to verify that the CLI parser correctly accepts `--logfile <path>` and maps it to `Some(PathBuf)`.
* Added `cli_logfile_flag_overrides_default_report_log` integration test to `tests/books.rs` to verify that running the binary with `--logfile <path>` successfully populates logs in the custom file and omits `report.log` in the run directory.
* Ran `cargo fmt`, `cargo check`, `cargo clippy`, and `cargo test` verifying all tests pass warning-free.

### Files Changed

* `src/main.rs` [MODIFY]
  * Added `--logfile` arg to `CliArgs`, updated `try_main` and `init_logging`.
* `src/tests.rs` [MODIFY]
  * Added `parse_args_accepts_logfile` unit test.
* `tests/books.rs` [MODIFY]
  * Added `cli_logfile_flag_overrides_default_report_log` integration test.
* `docs/codex-notes.md` [MODIFY]
  * Appended the current session notes.

### Tests Run

* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed warning-free)
* `cargo test` (138 unit tests and 27 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 Budapest Page Templates Handling


### Summary

Added template handling for the English Wikipedia "Budapest" article. Implemented rendering for five templates (`citation needed span`, `ndash`, `Quote box`, `center`, and `singular`) and silently omitted six metadata templates and ten navigation templates.

### Decisions Made

* Implemented `{{citation needed span|text}}` to render its text content while omitting the citation needed warning.
* Implemented `{{ndash}}` to render as a literal en dash `–`.
* Routed `{{Quote box}}` to the existing `render_blockquote_template` to render it as a cleanly formatted blockquote block.
* Implemented `{{center|text}}` to render as normal inline text (passthrough).
* Implemented `{{singular}}` to render as an abbreviation for singular form (`sg.`).
* Added `{{update section}}`, `{{party color}}`, `{{category see also}}`, `{{clarify}}`, `{{colbegin}}`, and `{{colend}}` to `src/silent.csv`.
* Added `{{Geographic location}}`, `{{Budapest}}`, `{{Municipalities in Budapest Metropolitan Area}}`, `{{World Heritage Sites in Hungary}}`, `{{Regional capitals of Hungary}}`, `{{Principal cities of Hungary}}`, `{{European Capital of Sport}}`, `{{Hungary's most flowery settlements}}`, `{{List of European capitals by region}}`, and `{{Danube}}` to `src/navigations.csv`.
* Sorted `src/silent.csv` and `src/navigations.csv` using `./sort.sh`.
* Wrote separate unit tests for `citation needed span`, `ndash`, `Quote box`, `center`, `singular`, and the silently omitted/navigation templates.
* Updated `DEVELOPMENT.md` to document all new conversions and omitted list updates.

### Files Changed

* `src/main.rs` [MODIFY]
  * Registered, dispatched, and implemented renderers for the new Budapest templates.
* `src/silent.csv` [MODIFY]
  * Registered the new silent templates.
* `src/navigations.csv` [MODIFY]
  * Registered the new navigation templates.
* `src/tests.rs` [MODIFY]
  * Added unit tests for each of the new template renderers and skips.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the new conversions and omissions.
* `docs/codex-notes.md` [MODIFY]
  * Added this session summary.

### Tests Run

* `./sort.sh` (sorted CSVs alphabetically)
* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed warning-free)
* `cargo test` (137 unit tests and 26 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-06-01 Hungary Page Templates Handling


### Summary

Added template handling for the English Wikipedia "Hungary" article. Implemented rendering for the `{{!}}` magic word (which outputs a vertical pipe `|`) and silently omitted metadata and navigation templates: `{{Wikiatlas}}` and `{{Hungary articles}}`.

### Decisions Made

* Implemented `{{!}}` template rendering to output a literal vertical pipe `|`. Registered it in `is_handled_template_name` and `render_template` in `src/main.rs`.
* Added `{{!}}` to `render_wikitext_formats_simple_inline_templates` unit tests.
* Identified page-level metadata template `{{Wikiatlas}}` and registered it in `src/silent.csv`.
* Identified country navigation template `{{Hungary articles}}` and registered it in `src/navigations.csv`.
* Sorted `src/silent.csv` and `src/navigations.csv` using `./sort.sh`.
* Wrote `render_wikitext_silently_skips_hungary_metadata_templates` unit test.
* Updated `DEVELOPMENT.md` to document the `{{!}}`, `{{Wikiatlas}}`, and `{{Hungary articles}}` rules.

### Files Changed

* `src/main.rs` [MODIFY]
  * Registered and dispatched `{{!}}` template.
* `src/silent.csv` [MODIFY]
  * Added `Wikiatlas` to silently ignored templates.
* `src/navigations.csv` [MODIFY]
  * Added `Hungary articles` to silently ignored navigation templates.
* `src/tests.rs` [MODIFY]
  * Added unit tests for `{{!}}` and silent skips of Hungary templates.
* `DEVELOPMENT.md` [MODIFY]
  * Documented the template conversion and omission rules.
* `docs/codex-notes.md` [MODIFY]
  * Added this session summary.

### Tests Run

* `./sort.sh` (sorted CSVs alphabetically)
* `cargo fmt` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed warning-free)
* `cargo test` (131 unit tests and 26 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-05-29 Command Line Flags Override (`--images`, `--no-images`)


### Summary

Added a pair of mutually exclusive command line flags called `--images` and `--no-images` to override the `images` configuration setting dynamically. Designed, implemented, and fully tested the changes with unit tests and end-to-end integration tests.

### Decisions Made

* Expanded `CliArgs` in `src/main.rs` to include `images` and `no_images` CLI flags, configured with mutually exclusive constraint using `conflicts_with`.
* Updated the `run` function to calculate an overridden `images` boolean: if `--images` is present, it forces image downloading; if `--no-images` is present, it disables image downloading; otherwise, it defaults to the configuration file value.
* Added `parse_args_accepts_images`, `parse_args_accepts_no_images`, and `parse_args_rejects_both_images_and_no_images` unit tests to `src/tests.rs` to cover all clap parsing scenarios.
* Added `cli_no_images_flag_overrides_config_images_true` and `cli_images_flag_overrides_config_images_false` end-to-end integration tests to `tests/books.rs` to compile offline books and verify that image files are correctly included/omitted in the generated EPUB based on command-line argument overrides.
* Formatted the Rust source code using `cargo fmt`, compiled cleanly, verified with strict linting (`cargo clippy --all-targets -- -D warnings`), and ran the entire test suite successfully.

### Files Changed

* `src/main.rs` [MODIFY]
  * Updated `CliArgs` struct and implemented dynamic CLI override logic in the `run` function.
* `src/tests.rs` [MODIFY]
  * Added unit tests for CLI argument parsing.
* `tests/books.rs` [MODIFY]
  * Added end-to-end integration tests using the compiled binary.
* `docs/codex-notes.md` [MODIFY]
  * Appended the current session notes.

### Tests Run

* `cargo fmt --check` (passed cleanly)
* `cargo check` (passed cleanly)
* `cargo clippy --all-targets -- -D warnings` (passed warning-free)
* `cargo test` (all 130 unit tests and 26 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-05-29 Website Improvement (Issue #32)

### Summary

Improved the static website layout and dynamic content generation to fulfill all requirements of Issue #32: embedding skeleton.yaml, linking to repository and examples directory, and displaying release date and version number. Also dynamically rendered README.md content directly into the landing page for better documentation.

### Decisions Made

* Updated `scripts/generate_site.py` to parse Cargo.toml for crate version, generate current date, read skeleton.yaml, and parse README.md as HTML using the python `markdown` module.
* Completely redesigned `templates/site/index.html.j2` with a premium HSL dark-themed layout (compatible with light mode OS preference), Outfit/Inter custom fonts, copy-to-clipboard javascript script, and clean SVG badges.
* Integrated the rendered README.md content dynamically to prevent duplication of description and upload guidelines.
* Verified generated site successfully outputs all embedded templates, badges, and examples correctly.

### Files Changed

* `scripts/generate_site.py` [MODIFY]
  * Updated dependencies and added parsing for Cargo.toml, date, skeleton.yaml, and README.md.
* `templates/site/index.html.j2` [MODIFY]
  * Redesigned site layout with glassmorphism, Google Fonts, badges, examples link, and embedded config copy box.

### Tests Run

* `python3 scripts/generate_site.py` (successfully generated website)
* `cargo fmt --check` (successful check)
* `cargo check` (successful check)
* `cargo clippy --all-targets -- -D warnings` (clean lint passing)
* `cargo test` (all 127 unit tests and 24 integration tests passed successfully)

### Pending Follow-Ups

* None.

## 2026-05-29 Added skeleton.yaml config template

### Summary

Created a fully annotated `skeleton.yaml` book configuration template at the root of the workspace. This template details every config field, provides logical default values, and contains inline comments explaining all valid options and settings for standard book generation.

### Decisions Made

* Created `/opt/skeleton.yaml` to serve as a clean starting point for compiling new EPUB books.
* Included all required and optional parameters: `metadata` (title, author, license, language, date, edition), `output-file`, `images`, `caching`, `depth`, and `articles`.
* Documented valid options for newly introduced features like `caching` ("none", "local", "central") and `depth`.

### Files Changed

* `skeleton.yaml` [NEW]
  * Created the configuration template.

### Tests Run

* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test` — all 127 unit tests and 24 integration tests passed successfully.

### Pending Follow-Ups

* None.

## 2026-05-28 Configuration-controlled Caching ("none", "local", "central")

### Summary

Implemented a required `caching` config field in standard book configuration YAML files allowing values `"none"`, `"local"`, and `"central"`. Improved the downloading and file caching mechanism to seamlessly resolve platform-specific central cache directories, local `.cache` current-directory cache repositories, or bypass caching entirely in memory.

### Decisions Made

* Added `CachingMode` enum in `src/main.rs` with variants `None`, `Local`, and `Central`.
* Added `caching` required config field to `BookConfig` struct.
* Extended the `DownloadCache` struct with an `enabled` boolean flag indicating whether disk caching should be used.
* Updated standard cache root resolution in the main `run` function to use central user directories, current-working-directory-relative `.cache` folders, or dummy fallbacks.
* Extended standard `read_or_fetch` and `fetch_and_write` text/byte cache helpers to support the `enabled` bypass flag, resolving in-memory lookups when disabled.
* Automatically updated all 24 existing YAML examples inside `examples/` to include `caching: central` to ensure backward compatibility.
* Fixed legacy unit tests and added exhaustive unit tests validating cache bypassing and path resolution.

### Files Changed

* `src/main.rs`
  * Implemented `CachingMode`, added caching field to `BookConfig`, updated `DownloadCache` struct, constructor, and resolve behaviors in `run`, and extended all text/byte helper calls.
* `src/tests.rs`
  * Added `caching: none` configurations to unit test YAML strings, and added two new caching unit tests (`caching_mode_none_bypasses_cache_writes` and `caching_mode_local_resolves_path`).
* `examples/*.yaml` (all 24 config files)
  * Pre-inserted `caching: central` configuration settings.

### Tests Run

* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test` — all 127 unit tests and 24 integration tests passed successfully.

### Pending Follow-Ups

* None.

## 2026-05-28 Panic diagnostic improvements in tests/books.rs

### Summary

Improved the testing panic contexts in `tests/books.rs` by replacing generic `.expect()` calls with informative `unwrap_or_else` blocks. These new panics print full contextual parameters such as directory paths, book identifiers, file names, and underlying OS errors.

### Decisions Made

* Replaced `expect` on `fs::create_dir_all(&work_dir)` with a verbose `unwrap_or_else` panic stating the exact directory that failed to create.
* Replaced `expect` on running the `wikipedia-to-epub` CLI command with a panic providing the book and the directory context, as well as the CLI execute error.
* Replaced `expect` on `fs::remove_dir_all(&work_dir)` with a panic reporting cleanup failures.
* Replaced `expect` in `first_difference_report` with a panic detailing string difference lengths.
* Replaced `expect` on `SystemTime::now().duration_since(UNIX_EPOCH)` with a descriptive panic message.
* Replaced `unwrap_or_else` in `collect_expected_epub_entries` with a verbose panic showing the error message during directory reads.

### Files Changed

* `tests/books.rs`
  * Improved diagnostic panic details inside `assert_generated_book_matches_expected`, `assert_real_api_generates_book`, `first_difference_report`, `collect_expected_epub_entries`, and `unique_test_dir`.

### Tests Run

* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test` — all 125 unit tests and 24 integration tests passed successfully.

### Pending Follow-Ups

* None.

## 2026-05-28 Wikitext table handling foundation — `strip_wikitext_tables`

### Summary

Introduced `strip_wikitext_tables`, a dedicated function that replaces the generic `strip_balanced_sections("{|", "|}")` call in `render_wikitext_impl`. The new function uses the same depth-tracking balanced-scan loop but, at every top-level `{|` opener encountered, extracts the table's opening attribute string (everything between `{|` and the first `|` on that same line) and logs it with `debug!`. This is the first step toward proper table rendering.

### Decisions Made

* `strip_wikitext_tables` replicates the same character-by-character scan as `strip_balanced_sections` but specialises it for `{|` / `|}` pairs, enabling future table rendering to be added incrementally inside the same function.
* The attribute extraction parses only the first line after `{|`, then takes the segment before the first `|` (which isolates the CSS class / style string), and trims whitespace.
* `strip_prefix("{|")` is used instead of manual `&remaining[2..]` slicing to satisfy `clippy::manual_strip`.
* `strip_balanced_sections` is now unused in production code; it is annotated with `#[cfg(test)]` so it remains available to the existing unit test without triggering a `dead_code` warning.
* The table-stripping behavior is otherwise unchanged, so no integration fixture updates were needed.

### Files Changed

* `src/main.rs`
  * Replaced `strip_balanced_sections(&text, "{|", "|}")` call in `render_wikitext_impl` with `strip_wikitext_tables(&text)`.
  * Added `fn strip_wikitext_tables` (depth-tracking scan + attribute logging).
  * Added `#[cfg(test)]` to `fn strip_balanced_sections` to suppress dead_code warning.
* `src/tests.rs`
  * Added `strip_wikitext_tables_removes_table_sections` unit test covering simple tables, multiple sequential tables, nested tables, and no-table pass-through.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test` — 124 unit tests passed, 23 integration tests passed, 1 ignored

### Pending Follow-Ups

* Future step: render table content instead of stripping it, using the logged attribute string as the starting point.

## 2026-05-28 Administrative Divisions of South Korea page — `small` template


### Summary

Added handling for the `{{small}}` template observed in `pages/Administrative_divisions_of_South_Korea.json`. The template is identical in behaviour to the already-handled `{{Smaller}}` — it wraps its text content in `<small>...</small>` tags — so it was registered as a straightforward alias.

### Decisions Made

* `{{small}}` is registered as a case-insensitive alias of `{{Smaller}}` in both the render dispatch and `is_handled_template_name`, sharing the existing `render_smaller_template` function and `__WIKIPEDIA_TO_EPUB_SMALL_START__` / `__WIKIPEDIA_TO_EPUB_SMALL_END__` placeholder path.
* No new function or placeholder was needed.

### Files Changed

* `src/main.rs`
  * Added `|| template.eq_ignore_ascii_case("small")` to the `smaller` dispatch branch and to `is_handled_template_name`.
* `src/tests.rs`
  * Added `render_wikitext_formats_small_template` with four cases: plain text, Korean-specific text, a nested wiki-link, and an empty-parameter edge case.
* `DEVELOPMENT.md`
  * Updated the `{{Smaller}}` bullet to include `{{small}}` as an alias.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test` — 123 unit tests passed, 23 integration tests passed, 1 ignored

### Pending Follow-Ups

* None.

## 2026-05-28 Goguryeo page templates handling and fixture updates


### Summary

This session completed the handling of five templates used in the Goguryeo page: `"Cleanup"`, `"tone"`, `"GBurl"`, `"cite thesis"`, and `"usurped"`. Page-level maintenance banners (`Cleanup` and `tone`) are silently skipped. `GBurl` builds Google Books URLs from named parameters. `cite thesis` reuses the existing citation renderer. `usurped` renders compromised archive URLs as external links.

### Decisions Made

* `Cleanup` and `tone` are article-scope maintenance banners and are omitted silently by registering them in `src/silent.csv`.
* `GBurl` parses named parameters (`id`, `p`, `pg`, `page`, `q`, `keywords`, `dq`, `text`) and constructs a clean Google Books external URL (`https://books.google.com/books?id=...`). Numeric page numbers generate `pg=PA{n}`; non-numeric page IDs pass through as `pg={val}`. Keywords are space-replaced with `+` for URL encoding.
* `cite thesis` routes directly to the existing `render_citation_template`, sharing author/title/publisher/year formatting with other citation templates.
* `usurped` parses both named (`1=`, `url=`) and positional URL parameters and renders the URL as a wikitext external link to preserve the archive access point.
* `tests/books.rs` had been temporarily set to overwrite expected fixtures during prior fixture refresh; it is now fully reverted to assertion mode.

### Files Changed

* `src/main.rs`
  * Registered `GBurl`, `cite thesis`, and `usurped` in `is_handled_template_name` and `render_template`.
  * Implemented `render_gburl_template` and `render_usurped_template`.
* `src/silent.csv` (sorted by sort.sh)
  * Added `Cleanup` and `tone` as recognized silent templates.
* `src/tests.rs`
  * Added four new unit tests: `render_wikitext_formats_gburl_template`, `render_wikitext_formats_cite_thesis_template`, `render_wikitext_formats_usurped_template`, and `render_wikitext_silently_skips_goguryeo_metadata_templates`.
* `tests/books.rs`
  * Reverted from fixture-writer mode back to standard assertion mode.
* `expected/goguryeo/` and `expected/korean-language/`
  * Refreshed EPUB integration fixtures to reflect newly rendered thesis and Google Book citations.
* `DEVELOPMENT.md`
  * Documented `cite thesis`, `GBurl`, and `usurped` conversion rules; added `Cleanup` and `tone` to the maintenance template omission list.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `./sort.sh`
* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test` — 122 unit tests passed, 23 integration tests passed, 1 ignored

### Pending Follow-Ups

* None.

## 2026-05-28 Buddhist Temples page templates handling and fixture updates


### Summary

This session completed the handling of the `"nihongo3"` template used in the Buddhist temples in Japan page. This included robustly parsing positional parameters (preserving empty fields for alignment) and formatting Japanese text blocks where Rōmaji is displayed first.

### Decisions Made

* `nihongo3` parses positional parameters while preserving empty elements to maintain proper field alignment (Rōmaji, Kanji/Kana, and English translation), recursively rendering inner wikitext templates.
* The output formats Rōmaji in italics first, followed by the Japanese Kanji script in a title-tagged `lang="ja"` span, and the English translation inside quotes.

### Files Changed

* `src/main.rs`
  * Registered and implemented dispatcher and renderer for `nihongo3`.
* `src/tests.rs`
  * Added a dedicated, comprehensive unit test verifying the different positional parameters variations.
* `expected/buddhist-temples-in-japan/`
  * Refreshed book integration Expected XHTML fixtures to include the newly rendered `nihongo3` output.
* `DEVELOPMENT.md`
  * Documented the `nihongo3` conversion rules.
* `docs/codex-notes.md`
  * Added this session summary.

### Tests Run

* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test`

### Pending Follow-Ups

* None.

## 2026-05-28 Kyoto page templates handling and fixture updates

### Summary

This session completed the handling of templates used in the Kyoto page: `"Expand section"`, `"Unreferencedsect"`, `"formatnum"`, `"STN"`, `"Clear left"`, and `"Kyoto"`. This included implementing numeric formatting for `formatnum`, rail station links for `STN`, and registering layout/maintenance templates as silent.

### Decisions Made

* `formatnum` parses numeric strings (both colon-separated and pipe-separated) case-insensitively, formats them with standard thousands separators (commas), and preserves decimal segments.
* `STN` formats railway station links as `[[StationName Station|Label]]` while supporting disambiguation terms, capitalized "Station" indicators, and custom labels.
* `Expand section`, `Unreferencedsect`, `Clear left`, `Kyoto`, and `Kyoto Prefecture` are page-level structures, navboxes, or layout components and are skipped silently.

### Files Changed

* `src/main.rs`
  * Registered and implemented dispatchers/renderers for `formatnum` and `STN` (using clippy-compliant prefix-stripping and collapsible if constructs).
* `src/silent.csv` (sorted by sort.sh)
  * Registered `Expand section`, `Unreferencedsect`, `Clear left`, `Kyoto`, and `Kyoto Prefecture` as recognized silent templates.
* `src/tests.rs`
  * Added 3 new unit tests for formatnum, STN, and Kyoto silent exclusions.
* `expected/kyoto/`
  * Refreshed book integration Expected XHTML fixtures to include the newly rendered Hamaōtsu railway station link.
* `DEVELOPMENT.md`
  * Documented all new template conversion and omission rules.
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

## 2026-05-28 Osaka page templates handling and fixture updates

### Summary

This session completed the handling of templates used in the Osaka page: `"Nihongo krt"`, `"Easy CSS image crop"`, `"Div end"`, `"ISSN"`, `"Cite NSRW"`, `"Sister bar"`, and `"Osaka"`. This included resolving empty positional parameter alignment and integrating styled Wikisource and image-registry links.

### Decisions Made

* `Nihongo krt` is rendered as Kanji text in a Japanese language span, followed by Romaji in italics and optional English/translation details inside parentheses. Utilized a robust positional parameters parser that preserves empty strings to ensure perfect alignment.
* `Easy CSS image crop` converts cropped image definitions seamlessly into standard wikitext image links (`[[File:...]]`), allowing them to be fully compiled and managed by the book's standard `ImageRegistry` and asset downloader.
* `ISSN` is formatted as standard Serial Number text (`ISSN {number}`) inline, perfectly aligned with the existing `ISBN` renderer.
* `Cite NSRW` renders structured citations from the public domain *The New Student's Reference Work* including link generation targeting English Wikisource.
* `Div end`, `Sister bar`, `Osaka`, and `Osaka Prefecture` are page-level structures, navboxes, or layout closures and are skipped silently.

### Files Changed

* `src/main.rs`
  * Registered and implemented dispatchers/renderers for `Nihongo krt`, `Easy CSS image crop`, `ISSN`, and `Cite NSRW`.
* `src/silent.csv` (sorted by sort.sh)
  * Registered `Div end`, `Sister bar`, `Osaka`, and `Osaka Prefecture` as recognized silent templates.
* `src/tests.rs`
  * Added 5 new unit tests for all renderers and silent exclusions.
* `expected/osaka/`
  * Refreshed book integration Expected XHTML fixtures to include the newly rendered Nihongo and citation output.
* `DEVELOPMENT.md`
  * Documented all four new template conversion rules.
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
* `DEVELOPMENT.md`
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
* `DEVELOPMENT.md`
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
* `DEVELOPMENT.md`
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
* `DEVELOPMENT.md`
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
* `DEVELOPMENT.md`
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
* `DEVELOPMENT.md`
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
* `DEVELOPMENT.md`
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
* `DEVELOPMENT.md`
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
* `DEVELOPMENT.md`
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
* `DEVELOPMENT.md`
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
* The DEVELOPMENT should describe conversion rules with concrete before/after examples.

### Files Changed

* `src/main.rs`
  * Added or extended rendering for templates including `ill`, `Reign`, `lang`, `langx`, `Percentage`, `UN Population`, `Korean/auto`, `Ko-translit`, `Cite report`, `harvc`, `As of`, `Blockquote`, `Further`, `For timeline`, `Wiktionary`, `Wikivoyage`, `sclass`, and `Free access`.
  * Added block-level handling for rendered blockquote markers so quotes are not flattened into ordinary paragraphs.
  * Updated citation author collection so unnumbered `last`/`first` can combine correctly with numbered coauthors such as `last2`/`first2`.
  * Added silent skipping for templates such as `Redirect`, `pp-semi-indef`, `Sfn`, `efn`, `refn`, `Reflist`, `notelist`, `Refbegin`, `Refend`, `flagicon`, `unreferenced section`, `Excessive citations inline`, `DEFAULTSORT`, `Commons category`, `Portal bar`, `Portal`, `Authority control`, `Seoul`, `Seoul weatherbox`, `Seoul landmarks`, `Navboxes`, succession templates prefixed with `s-`, and `Succession box`.
  * Added tests for template rendering behavior, including the restored example fixture and Korean transliteration cases.
* `DEVELOPMENT.md`
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
* `DEVELOPMENT.md`
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
* Consider adding more DEVELOPMENT examples for newly supported templates when their behavior becomes user-visible.

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
* `DEVELOPMENT.md`
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
* `DEVELOPMENT.md`
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

This session added rendering for the `Official website`, `Largest cities`, `linktext`, `Excerpt`, `For`, `URL`, `Webarchive`, `in lang`, `lit`, `ISBN`, `Wikisource`, `Nihongo`, `nbsp`, `cvt`, `osmrelation-inline`, `climate chart`, `IPAc-en`, `Respell`, and `cite ECCP` Wikipedia templates, added per-article and total skipped-template logging, updated DEVELOPMENT conversion notes, refreshed affected Korea EPUB fixtures, and verified the full test suite.

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
* `DEVELOPMENT.md`
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
* `DEVELOPMENT.md`
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
* `DEVELOPMENT.md`
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
* `DEVELOPMENT.md`
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

## Session Note: 2026-05-28 - Wikitable Class Preservation & Skip Logging

### Decisions Made

* Implemented handling of various classes with `"wikitable"`:
  * Extracted the `class` attribute of tables using a robust regex.
  * Verified if the class contains `"wikitable"`, which correctly matches `"wikitable sortable"`, `"wikitable plainrowheaders"`, etc.
  * Preserved the class attribute when rendering the XHTML `<table>` tag.
  * If a class is not recognized (i.e. does not contain `"wikitable"`), skipped rendering the table block and logged a warning `warn!(class = %class_str, "Skipping table with unrecognized class: {}", class_str);`.
* Isolated the table rendering using a placeholder structure `__WIKIPEDIA_TO_EPUB_TABLE_N__` to ensure the HTML elements are not stripped or mangled by paragraph-wrapping or inline styling post-processors.
* Updated `tests/books.rs` to temporarily run in fixture-writing mode, regenerated all 23 expected EPUB integration fixtures (which now beautifully contain the formatted tables), and restored `tests/books.rs` back to assertion mode.
* Updated `DEVELOPMENT.md` to document the newly-added support for wikitables.

### Files Changed

* `src/main.rs`
  * Added `extract_class_attr`, `table_marker_id`, and `strip_wikitext_tables`.
  * Updated `render_wikitext_tables` and `render_wikitable` to parse and format the class list and skip/log unrecognized tables.
  * Integrated placeholder replacement inside the line processing loop of `render_wikitext_impl`.
* `src/tests.rs`
  * Added `render_wikitable_preserves_various_classes_and_skips_unrecognized` unit test.
* `DEVELOPMENT.md`
  * Updated documentation to reflect that wikitables are converted into XHTML tables while non-wikitable tables are omitted.
* `docs/codex-notes.md`
  * Appended the current session notes.

### Tests Run

* `cargo check` (clean compile with no warnings)
* `cargo clippy --all-targets -- -D warnings` (passed cleanly with no warnings or errors)
* `cargo fmt --check` (clean formatting check)
* `cargo test` (all 125 unit tests and 23 integration tests pass successfully with 100% success rate)

### Pending Follow-Ups

* None. Everything is fully completed and successfully verified.

## Session Note: 2026-05-28 - Recursive Page Downloading

### Decisions Made

* Implemented the required `depth` configuration option for YAML configuration files:
  * Modified the `BookConfig` struct to include a required `depth: usize` field.
  * Updated all existing example YAML configurations in `examples/` with `depth: 0` for backward compatibility.
  * Added the new integration test config `examples/macchini-deep.yaml` which specifies `depth: 1`.
  * Updated `book_config_defaults_images_to_false` and `book_config_accepts_images_true` unit tests in `src/tests.rs` to include `depth: 0` in their parsed configurations.
* Implemented depth-first recursive article link resolution:
  * Formulated a recursive Depth-First Search (DFS) resolver starting from the initial articles specified in the YAML configuration.
  * Prevented duplicate visits and duplicate downloads/chapter insertions by tracking visited articles using a `HashSet<String>` containing the normalized titles.
  * Constructed `internal_links` map using the full set of resolved recursive articles to ensure all links to recursively downloaded chapters are correctly compiled as local references.
  * Cleaned up redundant downloads by caching all fetched `PageResponse` results in a `HashMap` during the DFS phase and passing the pre-loaded pages directly to `load_chapter`.
  * Handled dead links and missing files gracefully for recursively followed links (depth > 0) by skipping them and logging a warning instead of crashing.
* Implemented `extract_internal_links` and `is_valid_internal_article_link` to parse valid internal Wikipedia article link targets while ignoring namespaces (like `File:`, `Category:`), language-switching links, and other interwiki prefixes.
* Added `generate_macchini_deep_book_from_local_page_dump` to the integration test suite in `tests/books.rs`, generated its expected fixture outputs, and verified complete assertion compatibility.

### Files Changed

* `src/main.rs`
  * Added `depth` field to `BookConfig`.
  * Derived `Clone` for `PageResponse`, `ParsedPage`, and `WikitextValue`.
  * Modified `run` to perform DFS recursive article resolution and preloaded chapters compilation.
  * Modified `load_chapter` to receive the preloaded `PageResponse`.
  * Defined `is_valid_internal_article_link`, `extract_internal_links`, and `dfs_visit`.
* `src/tests.rs`
  * Updated unit tests to include `depth: 0`.
* `tests/books.rs`
  * Added `generate_macchini_deep_book_from_local_page_dump` integration test.
* `examples/*.yaml`
  * Configured required `depth: 0` option across all existing configs.
* `expected/macchini-deep/`
  * Generated expected EPUB fixture files for the recursive integration test.
* `docs/codex-notes.md`
  * Appended the current session notes.

### Tests Run

* `cargo check` (clean build)
* `cargo clippy --all-targets -- -D warnings` (zero warnings/errors)
* `cargo fmt --check` (clean formatting check)
* `cargo test` (all 125 unit tests and 24 integration tests pass successfully with 100% success rate)

### Pending Follow-Ups

* None. Everything is fully completed and successfully verified.

## Session Note: 2026-05-28 - Panic Improvement in Ignored Real API Tests

### Decisions Made

* Improved the EPUB entry lookup panic in `tests/books.rs`:
  * Replaced a generic `.expect("epub entry exists")` inside `read_epub_entry` with a detailed `.unwrap_or_else` that reports the exact name of the missing entry (e.g., `OEBPS/chapter-2.xhtml`) alongside the original panic reason.
* Identified and fixed an outdated assertion in the ignored integration test `generate_example_books_from_real_wikipedia_api`:
  * The `korea` book test case expected two chapters (`"Korea"` and `"Seoul"`), but the `examples/korea.yaml` configuration is configured with `depth: 0` and lists only one article (`"Korea"`).
  * Updated the assertion parameters to expect `&["Korea"]` instead of `&["Korea", "Seoul"]`, aligning it perfectly with the actual `korea.yaml` schema and matching the offline integration test expectations.
* Verified that both normal and ignored tests compile, format, and pass successfully with a 100% success rate.

### Files Changed

* `tests/books.rs`
  * Improved the `read_epub_entry` panic reporting code.
  * Corrected the expected chapter array in `generate_example_books_from_real_wikipedia_api` for the `korea` book.
* `docs/codex-notes.md`
  * Appended the current session notes.

### Tests Run

* `cargo fmt --check` (clean formatting check)
* `cargo clippy --all-targets -- -D warnings` (clean lint verification)
* `cargo test --locked -- --ignored` (successfully passed)
* `cargo test --locked` (successfully passed)

### Pending Follow-Ups

* None. Everything is fully resolved and verified.

## Session Note: 2026-06-03 - Transport in Greater Tokyo Template Support

### Decisions Made

* Supported missing templates identified from the PAGE="Transport in Greater Tokyo" English Wikipedia article:
  * Omitted the navigation template `Tokyo transit` by adding it to `src/navigations.csv` and running `./sort.sh` to sort it alphabetically.
  * Implemented `render_ja_rail_color_template` in `src/main.rs` to return standardized hex color codes for common Japanese rail lines.
  * Implemented `render_route_box_template` in `src/main.rs` to render colored route badge spans containing a wikilink.
  * Utilized placeholder strings (`__WIKIPEDIA_TO_EPUB_ROUTE_BOX_START__` etc.) and integrated a custom post-processor `restore_route_box_spans` in `format_inline_text` to preserve style attributes and prevent the wikitext parser from misinterpreting hex color codes starting with `#` as ordered lists.
* Wrote separate unit tests for `Ja-rail-color` and `RouteBox` in `src/tests.rs` to verify correct formatting and parameter evaluation (including nested template expansion).
* Updated `DEVELOPMENT.md` with conversion rules for `Ja-rail-color` and `RouteBox`.

### Files Changed

* `src/main.rs`
  * Implemented `render_route_box_template`, `render_ja_rail_color_template`, and `restore_route_box_spans`.
  * Registered `RouteBox` and `Ja-rail-color` in `render_template`, `is_handled_template_name`, and `format_inline_text`.
* `src/tests.rs`
  * Added `render_wikitext_formats_ja_rail_color_template` and `render_wikitext_formats_route_box_template`.
* `src/navigations.csv`
  * Added `Tokyo transit` and sorted the file.
* `DEVELOPMENT.md`
  * Documented rules for `Ja-rail-color` and `RouteBox`.
* `docs/codex-notes.md`
  * Appended the current session notes.

### Tests Run

* `cargo fmt --check` (clean formatting check)
* `cargo check` (clean compile with no warnings)
* `cargo clippy --all-targets -- -D warnings` (clean lint verification)
* `cargo test` (all 195 unit tests and 30 integration tests pass successfully with 100% success rate)

### Pending Follow-Ups

* None. All tests pass and the templates are fully supported.

## Session Note: 2026-06-03 - Tokyo Template Support

### Decisions Made

* Supported missing templates identified from the PAGE="Tokyo" English Wikipedia article:
  * Omitted metadata/layout/map templates `Hatnote group`, `pp-move-indef`, `pp-protected`, and `Tokyo Metropolis Labelled Map` by adding them to `src/silent.csv` and running `./sort.sh` to sort alphabetically.
  * Silently bypassed any template/parser function starting with `#` (e.g. `#chart:...`) by checking `template.starts_with('#')` in `is_silent_template_name`.
  * Implemented `render_nihongo_foot_template` to render the inline English text followed by its Japanese translation and romanization in parentheses.
  * Integrated `Literal translation` as an alias of the `lit` template renderer `render_literal_template`.
  * Implemented `render_na_template` to support `N/A`, `NA`, and `Not applicable` comparison table templates.
  * Added direct support for character escaping templates `'" ` (Single+double quote) and `"' ` (Double+single quote).
* Wrote separate unit tests for `Nihongo foot`, `Literal translation`, `N/A`, and single/double quote templates in `src/tests.rs`.
* Updated `DEVELOPMENT.md` with conversion rules for `Nihongo foot`, `Literal translation`, `N/A`, and quote escaping templates.

### Files Changed

* `src/main.rs`
  * Implemented `render_nihongo_foot_template` and `render_na_template`.
  * Added routing and prefix bypasses for new templates inside `render_template`, `is_handled_template_name`, and `is_silent_template_name`.
* `src/tests.rs`
  * Added `render_wikitext_formats_nihongo_foot_template`, `render_wikitext_formats_literal_translation_template`, `render_wikitext_formats_na_template`, and `render_wikitext_formats_quote_escaping_templates`.
* `src/silent.csv`
  * Added `Hatnote group`, `pp-move-indef`, `pp-protected`, and `Tokyo Metropolis Labelled Map`, and sorted the file.
* `DEVELOPMENT.md`
  * Documented conversion rules for the new templates.
* `docs/codex-notes.md`
  * Appended the current session notes.

### Tests Run

* `cargo fmt --check` (clean formatting check)
* `cargo check` (clean compile with no warnings)
* `cargo clippy --all-targets -- -D warnings` (clean lint verification)
* `cargo test` (all 199 unit tests and 30 integration tests pass successfully with 100% success rate)

### Pending Follow-Ups

* None. All tests pass and the templates are fully supported.


## Session Note: 2026-06-08 - Expanded Nihongo Template Handling

### Decisions Made

* Supported additional parameters for the `Nihongo` and `Nihongo4` templates:
  * Parsed and rendered the third positional parameter (Rōmaji) in italics.
  * Parsed and rendered the fourth positional parameter (Extra) inside the parentheses.
  * Parsed and rendered the fifth positional parameter (Extra2) outside the parentheses.
  * Supported the `lead` parameter (e.g. `lead=yes`) to add the "Japanese:" and "Hepburn:" labels inside the parentheses.
* Added a new unit test for the extended `Nihongo` template features in `src/tests.rs`.
* Updated `DEVELOPMENT.md` to document the extended conversion rules.
* Updated expected XHTML fixtures for all affected integration tests (`Kiso_Mountains`, `buddhist-temples-in-japan`, `japan`, `kyoto`, and `osaka`) using the python-based regeneration process.

### Files Changed

* `src/main.rs`
  * Updated `render_japanese_template` to support all 5 positional parameters plus the `lead` parameter.
* `src/tests.rs`
  * Added `render_wikitext_formats_japanese_nihongo_extended_templates` unit test.
* `DEVELOPMENT.md`
  * Updated the example and description of `Nihongo` template conversion.
* `expected/buddhist-temples-in-japan/`
* `expected/japan/`
* `expected/Kiso_Mountains/`
* `expected/kyoto/`
* `expected/osaka/`
  * Regenerated expected XHTML fixtures to match the new template outputs.
* `docs/codex-notes.md`
  * Appended the current session notes.

### Tests Run

* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test` (all 225 unit tests and 33 integration tests pass successfully)

### Pending Follow-Ups

* None.


## Session Note: 2026-06-10 - Replace Wildcard Imports

### Decisions Made

* Replaced all wildcard imports (`use ...::*;`) with explicit listings across the codebase:
  * `src/main.rs`: Replaced `pub(crate) use templates::*;` with explicit template function imports. Changed the logging setup to import the required traits (`Layer`, `SubscriberExt`, `SubscriberInitExt`) explicitly rather than using `tracing_subscriber::prelude::*`.
  * `src/templates/mod.rs`: Replaced wildcard re-exports of submodules (`citation`, `convert`, `formatting`, `lang`) with explicit public re-exports and private imports.
  * `src/templates/lang.rs`: Replaced `use crate::templates::*;` with explicit imports for `join_plain_items`, `render_templates`, `template_named_params`, `template_param`, and `template_positional_params`.
  * `src/templates/citation.rs`: Fixed `PersonRole` and `citation_people` root-level imports to reference `crate::templates` directly.
  * `src/tests.rs`: Fixed imports of `template_log_content` and `template_name_is_in_csv` to import from `crate::templates::` since they are no longer re-exported to the root level.
* Resolved Clippy warnings/errors in `src/main.rs`:
  * Created `CoverImage` type alias for the complex return type of `get_cover_image`.
  * Adjusted `wikipedia_language` parameters from `&String` to `&str` and `ordered_articles` from `&Vec<String>` to `&[String]`.
* Formatted the modified code and ran checks, lints, and all tests.

### Files Changed

* `src/main.rs`
* `src/tests.rs`
* `src/templates/mod.rs`
* `src/templates/lang.rs`
* `src/templates/citation.rs`
* `docs/codex-notes.md`

### Tests Run

* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test` (all 263 unit tests and 34 integration tests passed successfully)

### Pending Follow-Ups

* None.


## Session Note: 2026-06-19 - Restrict Visibility of Crate Elements

### Decisions Made

* Restricted the visibility of modules, structs, enums, fields, and functions throughout the codebase where possible to minimize public API surface.
* **`src/main.rs`**: Made all submodule declarations and imports private.
* **`src/templates/mod.rs`**: Made internal submodules (`citation`, `convert`, `formatting`, `lang`) private. Changed `render_template` helper function to private.
* **`src/templates/formatting.rs`**: Made `render_lagrange_template` private.
* **`src/templates/convert.rs`**: Made `format_convert_value` private.
* **`src/epub.rs`**: Changed `Chapter`, `TocNode`, their fields, and various loading/utility functions to `pub(crate)`. Changed `is_right_to_left_language` to private.
* **`src/image.rs`**: Changed internal helper structs (`BookImageSource`, `LocalImageFixture`, `BookImage`, `ImageAvailability`) to private or `pub(crate)`, and restricted their fields and functions accordingly.
* **`src/cache.rs`**: Changed structures (`PageResponse`, `ParsedPage`, `WikitextValue`, `DownloadCache`, `DownloadStats`, `FileDownloadStats`, `FileDownloadSnapshot`, `CacheSource`, `FixturePageSource`), traits (`PageSource`), methods, and functions (`wikipedia_parse_api_url`, `normalized_wikipedia_language`, `normalize_lookup_key`, etc.) to `pub(crate)`. Restricted `WikipediaErrorResponse`, `WikipediaError`, and helper functions like `cache_key` to private. Made `write_cache_text` `pub(crate)` as it is used in tests.
* **`src/config.rs`**: Changed `ChapterStyle` visibility to `pub(crate)`.
* **`src/error.rs`**: Restricted `AppError` and `AppResult` to `pub(crate)`.
* Verified that all unit tests, integration tests, and doc-tests continue to compile and pass cleanly.

### Files Changed

* [src/main.rs](file:///opt/src/main.rs) [MODIFY]
* [src/templates/mod.rs](file:///opt/src/templates/mod.rs) [MODIFY]
* [src/templates/formatting.rs](file:///opt/src/templates/formatting.rs) [MODIFY]
* [src/templates/convert.rs](file:///opt/src/templates/convert.rs) [MODIFY]
* [src/epub.rs](file:///opt/src/epub.rs) [MODIFY]
* [src/image.rs](file:///opt/src/image.rs) [MODIFY]
* [src/cache.rs](file:///opt/src/cache.rs) [MODIFY]
* [src/config.rs](file:///opt/src/config.rs) [MODIFY]
* [src/error.rs](file:///opt/src/error.rs) [MODIFY]
* [docs/codex-notes.md](file:///opt/docs/codex-notes.md) [MODIFY]

### Tests Run

* `cargo fmt` (clean formatting check)
* `cargo check` (clean compile with no warnings)
* `cargo clippy --all-targets -- -D warnings` (clean lint verification)
* `cargo test` and `cargo test --locked -- --ignored` (all passed successfully)

### Pending Follow-Ups

* None.


## Session Note: 2026-06-19 - Explain log_and_count_nested_skipped_templates

### Decisions Made

* Explained the necessity of `log_and_count_nested_skipped_templates`.
* Verified workspace cleanliness and successfully ran cargo formatting, check, clippy, and test validations.

### Files Changed

* `docs/codex-notes.md`

### Tests Run

* `cargo fmt`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test` and `cargo test --locked -- --ignored`

### Pending Follow-Ups

* None.


## Session Note: 2026-06-19 - Separate Main Article Template display with empty row

### Decisions Made

* Updated `render_main_template` in `src/templates/formatting.rs` to output prepended and appended double newlines, ensuring that `Main` and `Main article` templates render in their own paragraph, separated from other elements with an empty row.
* Regenerated expected XHTML fixtures for all book examples to match the new template paragraph separation format.
* Updated `DEVELOPMENT.md` to reflect the updated rendering description.
* Ran formatting, check, clippy, unit tests, and integration tests to ensure clean passing.

### Files Changed

* `src/templates/formatting.rs` [MODIFY]
* `DEVELOPMENT.md` [MODIFY]
* Expected XHTML fixtures in the `expected/` directory [MODIFY]
* `docs/codex-notes.md` [MODIFY]

### Tests Run

* `cargo fmt --check`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test` and `cargo test --locked -- --ignored`

### Pending Follow-Ups

* None.


## Session Note: 2026-06-20 - Add unit tests for strip_reflist_templates

### Decisions Made

* Added comprehensive unit tests for `strip_reflist_templates` in `src/cleanup.rs`.
* Validated that all checks and unit/integration tests compile and pass cleanly.

### Files Changed

* `src/cleanup.rs` [MODIFY]
* `docs/codex-notes.md` [MODIFY]

### Tests Run

* `cargo fmt --check`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test` and `cargo test --locked -- --ignored`

### Pending Follow-Ups

* None.


## Session Note: 2026-06-20 - Add unit tests for collect_reference_groups

### Decisions Made

* Added comprehensive unit tests for `collect_reference_groups` in `src/cleanup.rs`.
* Validated that all checks and unit/integration tests compile and pass cleanly.

### Files Changed

* `src/cleanup.rs` [MODIFY]
* `docs/codex-notes.md` [MODIFY]

### Tests Run

* `cargo fmt --check`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test` and `cargo test --locked -- --ignored`

### Pending Follow-Ups

* None.


## Session Note: 2026-06-20 - Add unit tests for remove_some_html_tags

### Decisions Made

* Added comprehensive unit tests for `remove_some_html_tags` in `src/cleanup.rs`.
* Validated that all checks and unit/integration tests compile and pass cleanly.

### Files Changed

* `src/cleanup.rs` [MODIFY]
* `docs/codex-notes.md` [MODIFY]

### Tests Run

* `cargo fmt --check`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test` and `cargo test --locked -- --ignored`

### Pending Follow-Ups

* None.


## Session Note: 2026-06-20 - Add unit tests for is_file_link_start

### Decisions Made

* Added comprehensive unit tests for `is_file_link_start` in `src/tools.rs`.
* Validated that all checks and unit/integration tests compile and pass cleanly.

### Files Changed

* `src/tools.rs` [MODIFY]
* `docs/codex-notes.md` [MODIFY]

### Tests Run

* `cargo fmt --check`
* `cargo check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test` and `cargo test --locked -- --ignored`

### Pending Follow-Ups

* None.
