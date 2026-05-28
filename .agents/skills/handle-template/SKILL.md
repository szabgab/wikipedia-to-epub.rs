---
name: handle-template
description: >-
  Add, update, or test custom Wikipedia templates in the wikipedia-to-epub converter.
  Use when the user requests support for a new Wikipedia template, or modifications
  to template rendering logic.
---

When adding or changing Wikipedia template handling:

* Add or update the renderer.
* Add unit tests for each template/case.
* Update README.md conversion rules.
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

