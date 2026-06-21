mod common;

use common::{repo_root, unique_test_dir};
use std::{fs, process::Command};

#[test]
fn generate_book_fails_if_front_matter_file_is_missing() {
    let repo = repo_root();
    let work_dir = unique_test_dir(&repo, "missing-frontmatter-book");
    fs::create_dir_all(&work_dir).unwrap();

    let config_path = work_dir.join("book.yaml");
    let yaml = r#"chapters: title
metadata:
  title: "Japan"
  author: "Wikipedia contributors"
  language: en
  edition: First edition
output-file: output.epub
cover: "None"
links_to_pages: false
links_to_excluded_pages: emphasize
caching: none
depth: 0
front_matter:
  - non_existent_frontmatter.md
articles:
  - "Japan"
"#;
    fs::write(&config_path, yaml).unwrap();

    let mut command = Command::new(env!("CARGO_BIN_EXE_wikipedia-to-epub"));
    command
        .current_dir(&work_dir)
        .arg(&config_path)
        .arg("--local")
        .arg(repo.join("pages"))
        .arg("--caching")
        .arg("none")
        .arg("--log")
        .arg("WARN");
    let output = command.output().unwrap();

    assert!(
        !output.status.success(),
        "run should fail because of missing front matter"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("front matter file not found:"),
        "error message should complain about missing front matter: {}",
        stderr
    );

    fs::remove_dir_all(&work_dir).unwrap();
}
