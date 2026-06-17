# Contributing


## Handle templates

* Create a book YAML file with a few pages.
* Generate the book: `cargo run -- --log debug examples/korea.yaml`
* `grep "removing unhandled wikitext" report.log`

## Release and publish

* `git push`
* Wait for the CI to finish.
* Update version number in `Cargo.toml` and in the comment below.
* `cargo build`     (to update `Cargo.lock`)
* `git add .`
* `git commit -m "update version to v0.1.4"`
* `git push`
* Wait for the CI to finish.
* `git tag -a v0.1.4 -m "publish version v0.1.4"`
* `git push --tags`
* Pushing a tag starting with `v` triggers GitHub Actions to verify the tag matches `Cargo.toml`, build binaries for Linux, macOS, and Windows, publish a GitHub release with those assets, and deploy a GitHub Pages site based on `README.md`.
* `cargo publish`

