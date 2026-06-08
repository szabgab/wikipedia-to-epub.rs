# Wikipedia to epub

A command line tool written in Rust that given a configuration file such as `examples/korea.yaml` will generate an `.epub` file that can used on an Amazon Kindle, along with an `.html` report that summarizes the included page hierarchy and linked-but-excluded Wikipedia pages.

## Install

Download binaries from [Wikipedia to epub](https://wikipedia-to-epub.code-maven.com/).

If you have Rust on your computer you can also install with:

```
cargo install wikipedia-to-epub
```

## First book

To get started copy the content of the `skeleton.yaml` file from the repository or from the web site, adjust the fields to your liking and run the `wikipedia-to-epub` command.

Each run creates the EPUB named by `output-file` and a companion HTML report with the same basename and an `.html` extension.


## Amazon

You can upload your book to your Kindle via this link: [Send to Kindle](https://www.amazon.com/sendtokindle)
