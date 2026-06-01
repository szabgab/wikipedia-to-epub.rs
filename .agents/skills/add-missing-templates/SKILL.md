---
name: add-missing-templates-handle
description: >-
  Add, update, or test custom Wikipedia templates in the wikipedia-to-epub converter.
  Use when the user requests support for Wikipedia templates on a page.
  Handle templates on LANGUAGE PAGE.
---

When adding Wikipedia template handling:

Given a LANGUAGE and a PAGE title:

* Create a temporary book (yaml file) including this LANGUAGE and PAGE. Without images. Central cache
* Generate the book using `cargo run -- --log debug book.yaml`
* Observe the missing templates by running `grep "removing unhandled wikitext template" report.log`.

* It there are no missing templates, you can finish here.

* If there are missing templates do the following:

* Download the rendered page from Wikipedia `https://LANGUAGE.wikipedia.org/wiki/PAGE`

* For each missing TEMPLATE then do the following:
    * Check how the TEMPLATE is rendered in the downloaded wikipedia page.
    * Check `https://en.wikipedia.org/wiki/Template:TEMPLATE` for an explanation on what to do with the TEMPLATE.
    * Decide if the template needs to be rendered or skipped.
    * If skipped update the `src/silent.csv` or the `src/navigations.csv` file.

    * If needs to be rendered add a renderer.
    * Add unit tests for the template for various use-cases.
    * Update DEVELOPMENT.md conversion rules.

* If integration tests fail because expected XHTML intentionally changed, update expected fixtures.
* Run:
  * cargo test <focused test if useful>
  * `cargo test --test books` if fixtures are affected
  * `cargo test` before finalizing
  * `cargo fmt` before finalizing
  * `cargo check` before finalizing
  * `cargo clippy --all-targets -- -D warnings` before finalizing
  * Run `./sort.sh` before finalizing.
* Update docs/codex-notes.md before ending the session.

