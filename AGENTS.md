
* Always run cargo test before finalizing code changes.
* When adding a new wikipedia template always write a unit-test and also update the README.md file.

* Before exiting codex do the following:
    * Update docs/codex-notes.md with a concise summary of this Codex session, including decisions made, files changed, tests run, and any pending follow-ups.

* Keep `src/navigations.csv` and `src/silent.csv` sorted.


# wikipedia-to-epub.rs Codex Instructions

When adding or changing Wikipedia template handling:

* Add or update the renderer.
* Add unit tests for each template/case.
* Update README.md conversion rules.
* If integration tests fail because expected XHTML intentionally changed, update expected fixtures.
* Run:
  * cargo test <focused test if useful>
  * cargo test --test books if fixtures are affected
  * cargo test before finalizing
* Update docs/codex-notes.md before ending the session.

