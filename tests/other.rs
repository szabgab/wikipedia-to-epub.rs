mod common;

use common::{extract_opf_date, open_epub, repo_root, unique_test_dir, zip_entries};
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

#[test]
fn cli_logfile_flag_overrides_default_report_log() {
    let repo = repo_root();
    let work_dir = unique_test_dir(&repo, "logfile-override");
    fs::create_dir_all(&work_dir).unwrap();

    let custom_log = work_dir.join("custom_report.log");
    let yaml_path = repo.join("examples/busan.yaml");
    let mut command = Command::new(env!("CARGO_BIN_EXE_wikipedia-to-epub"));
    command
        .current_dir(&work_dir)
        .arg(&yaml_path)
        .arg("--local")
        .arg(repo.join("pages"))
        .arg("--logfile")
        .arg(&custom_log)
        .arg("--log")
        .arg("INFO");
    let expected_dir = repo.join("expected").join("busan");
    if let Some(date) = extract_opf_date(&expected_dir) {
        command.env("WIKIPEDIA_TO_EPUB_MOCK_DATE", date);
    } else {
        command.env("WIKIPEDIA_TO_EPUB_MOCK_DATE", "2026-06-06");
    }
    let output = command.output().unwrap();

    assert!(output.status.success(), "run failed: {:?}", output);
    assert!(custom_log.is_file(), "custom log file was not created");
    assert!(
        !work_dir.join("report.log").exists(),
        "default report.log should not be created"
    );

    let log_content = fs::read_to_string(&custom_log).unwrap();
    assert!(
        log_content.contains("starting wikipedia-to-epub"),
        "log content missing expected starting message"
    );

    fs::remove_dir_all(&work_dir).unwrap();
}

#[test]
fn cli_caching_flag_is_accepted_by_binary() {
    let repo = repo_root();
    let work_dir = unique_test_dir(&repo, "caching-flag-acceptance");
    fs::create_dir_all(&work_dir).unwrap();

    let yaml_path = repo.join("examples/busan.yaml");
    let mut command = Command::new(env!("CARGO_BIN_EXE_wikipedia-to-epub"));
    command
        .current_dir(&work_dir)
        .arg(&yaml_path)
        .arg("--local")
        .arg(repo.join("pages"))
        .arg("--caching")
        .arg("none")
        .arg("--log")
        .arg("WARN");
    let expected_dir = repo.join("expected").join("busan");
    if let Some(date) = extract_opf_date(&expected_dir) {
        command.env("WIKIPEDIA_TO_EPUB_MOCK_DATE", date);
    } else {
        command.env("WIKIPEDIA_TO_EPUB_MOCK_DATE", "2026-06-06");
    }
    let output = command.output().unwrap();

    assert!(output.status.success(), "run failed: {:?}", output);
    fs::remove_dir_all(&work_dir).unwrap();
}

#[test]
fn cli_no_images_flag_overrides_config_images_true() {
    let repo = repo_root();
    let work_dir = unique_test_dir(&repo, "busan-images-override");
    fs::create_dir_all(&work_dir).unwrap();

    let output_file_name = "busan-images.epub";
    let yaml_path = repo.join("examples/busan-images.yaml");
    let mut command = Command::new(env!("CARGO_BIN_EXE_wikipedia-to-epub"));
    command
        .current_dir(&work_dir)
        .arg(&yaml_path)
        .arg("--local")
        .arg(repo.join("pages"))
        .arg("--no-images")
        .arg("--log")
        .arg("WARN");
    let expected_dir = repo.join("expected").join("busan-images");
    if let Some(date) = extract_opf_date(&expected_dir) {
        command.env("WIKIPEDIA_TO_EPUB_MOCK_DATE", date);
    } else {
        command.env("WIKIPEDIA_TO_EPUB_MOCK_DATE", "2026-06-06");
    }
    let output = command.output().unwrap();

    assert!(output.status.success(), "run failed: {:?}", output);
    let output_file = work_dir.join(output_file_name);
    assert!(output_file.is_file());

    let epub = open_epub(&output_file);
    let entries = zip_entries(&epub);
    // Verify that NO images are included in the generated EPUB
    assert!(!entries.iter().any(|entry| entry.contains("OEBPS/images/")));

    fs::remove_dir_all(&work_dir).unwrap();
}

#[test]
fn cli_images_flag_overrides_config_images_false() {
    let repo = repo_root();
    let work_dir = unique_test_dir(&repo, "busan-no-images-override");
    fs::create_dir_all(&work_dir).unwrap();

    let output_file_name = "busan.epub";
    let yaml_path = repo.join("examples/busan.yaml");
    let mut command = Command::new(env!("CARGO_BIN_EXE_wikipedia-to-epub"));
    command
        .current_dir(&work_dir)
        .arg(&yaml_path)
        .arg("--local")
        .arg(repo.join("pages"))
        .arg("--images")
        .arg("--log")
        .arg("WARN");
    let expected_dir = repo.join("expected").join("busan");
    if let Some(date) = extract_opf_date(&expected_dir) {
        command.env("WIKIPEDIA_TO_EPUB_MOCK_DATE", date);
    } else {
        command.env("WIKIPEDIA_TO_EPUB_MOCK_DATE", "2026-06-06");
    }
    let output = command.output().unwrap();

    assert!(output.status.success(), "run failed: {:?}", output);
    let output_file = work_dir.join(output_file_name);
    assert!(output_file.is_file());

    let epub = open_epub(&output_file);
    let entries = zip_entries(&epub);
    // Verify that images ARE included in the generated EPUB
    assert!(entries.iter().any(|entry| entry.contains("OEBPS/images/")));

    fs::remove_dir_all(&work_dir).unwrap();
}
