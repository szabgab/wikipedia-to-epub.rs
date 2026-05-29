# Contributing

## Release and publish

* Update version number in Cargo.toml and in the comment below.
* `cargo build`
* `git add .`
* `git commit -m "update version to v0.1.0"`
* `git push`
* `git tag -a v0.1.0 -m "publish version v0.1.0"`
* `git push --tags`
* Pushing a tag starting with `v` triggers GitHub Actions to verify the tag matches `Cargo.toml`, build binaries for Linux, macOS, and Windows, publish a GitHub release with those assets, and deploy a GitHub Pages site based on `README.md`.
* `cargo publish`

