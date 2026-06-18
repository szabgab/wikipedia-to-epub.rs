use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use regex::Regex;
use similar::{DiffTag, TextDiff};
use zip::ZipArchive;

#[test]
fn generate_macchini_book_from_local_page_dump() {
    assert_generated_book_matches_expected("macchini");
}

#[test]
fn generate_macchini_deep_book_from_local_page_dump() {
    assert_generated_book_matches_expected("macchini-deep");
}

#[test]
fn generate_administrative_divisions_of_south_korea_book_from_local_page_dump() {
    assert_generated_book_matches_expected("administrative-divisions-of-south-korea");
}

#[test]
fn generate_goguryeo_book_from_local_page_dump() {
    assert_generated_book_matches_expected("goguryeo");
}

#[test]
fn generate_japan_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("japan");
}

#[test]
fn generate_osaka_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("osaka");
}

#[test]
fn generate_buddhist_temples_in_japan_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("buddhist-temples-in-japan");
}

#[test]
fn generate_kyoto_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("kyoto");
}

#[test]
fn generate_korea_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("korea");
}

#[test]
fn generate_korean_language_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("korean-language");
}

#[test]
fn generate_busan_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("busan");
}

#[test]
fn generate_history_of_korea_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("history-of-korea");
}

#[test]
fn generate_seoul_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("seoul");
}

#[test]
fn generate_joseon_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("joseon");
}

#[test]
fn generate_sejong_the_great_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("sejong-the-great");
}

#[test]
fn generate_south_korea_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("south-korea");
}

#[test]
fn generate_north_korea_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("north-korea");
}

#[test]
fn generate_hangul_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("hangul");
}

#[test]
fn generate_korean_war_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("korean-war");
}

#[test]
fn generate_han_dynasty_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("han-dynasty");
}

#[test]
fn generate_parhae_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("parhae");
}

#[test]
fn generate_planets_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("planets");
}

#[test]
fn generate_spanish_corea_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("es-corea");
}

#[test]
fn generate_korea_in_hebrew_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("korea-in-hebrew");
}

#[test]
fn generate_busan_images_book_from_local_page_dump() {
    assert_generated_book_matches_expected("busan-images");
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
#[ignore = "hits the real Wikipedia API"]
fn generate_example_books_from_real_wikipedia_api() {
    assert_real_api_generates_book("korea", &["Korea"]);
    assert_real_api_generates_book("macchini", &["Macchini", "Licia Macchini"]);
}

fn assert_generated_book_matches_expected(book: &str) {
    let repo = repo_root();
    let work_dir = unique_test_dir(&repo, book);
    fs::create_dir_all(&work_dir).unwrap_or_else(|err| {
        panic!(
            "failed to create test output directory {}: {:?}",
            work_dir.display(),
            err
        )
    });

    let output_file_name = format!("{book}.epub");
    let yaml_path = repo.join(format!("examples/{book}.yaml"));
    let mut command = Command::new(env!("CARGO_BIN_EXE_wikipedia-to-epub"));
    command
        .current_dir(&work_dir)
        .arg(&yaml_path)
        .arg("--local")
        .arg(repo.join("pages"))
        .arg("--log")
        .arg("WARN");

    let expected_dir = repo.join("expected").join(book);
    if let Some(date) = extract_opf_date(&expected_dir) {
        command.env("WIKIPEDIA_TO_EPUB_MOCK_DATE", date);
    }

    let output = command.output().unwrap_or_else(|err| {
        panic!(
            "failed to execute wikipedia-to-epub binary for book '{}' at {}: {:?}",
            book,
            work_dir.display(),
            err
        )
    });

    assert!(
        output.status.success(),
        "wikipedia-to-epub failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_cli_stdout(&output.stdout, &output_file_name);

    let output_file = work_dir.join(output_file_name);
    assert!(output_file.is_file());

    let mut epub = open_epub(&output_file);
    let expected_entries = expected_epub_entries(&expected_dir);
    assert_eq!(zip_entries(&epub), expected_entries);

    for entry_name in expected_entries {
        let expected_path = expected_dir.join(&entry_name);
        let name_lower = entry_name.to_lowercase();
        let is_binary = name_lower.ends_with(".png")
            || name_lower.ends_with(".jpg")
            || name_lower.ends_with(".jpeg")
            || name_lower.ends_with(".gif");

        if is_binary {
            let mut entry = epub
                .by_name(&entry_name)
                .unwrap_or_else(|err| panic!("epub entry '{}' exists: {:?}", entry_name, err));
            let mut generated_bytes = Vec::new();
            entry.read_to_end(&mut generated_bytes).unwrap();

            let expected_bytes = fs::read(&expected_path).unwrap_or_else(|err| {
                panic!(
                    "expected epub entry '{}' reads: {:?}",
                    expected_path.display(),
                    err
                )
            });
            assert_eq!(
                generated_bytes, expected_bytes,
                "binary file mismatch for {}",
                entry_name
            );
        } else {
            let generated =
                normalize_epub_entry(&entry_name, &read_epub_entry(&mut epub, &entry_name));
            let expected = normalize_epub_entry(
                &entry_name,
                &fs::read_to_string(&expected_path).unwrap_or_else(|err| {
                    panic!(
                        "expected epub entry '{}' reads: {:?}",
                        expected_path.display(),
                        err
                    )
                }),
            );
            assert_text_matches_expected(&entry_name, &generated, &expected);
        }
    }

    fs::remove_dir_all(&work_dir).unwrap_or_else(|err| {
        panic!(
            "failed to clean up test output directory {}: {:?}",
            work_dir.display(),
            err
        )
    });
}

fn assert_real_api_generates_book(book: &str, chapter_titles: &[&str]) {
    let repo = repo_root();
    let work_dir = unique_test_dir(&repo, &format!("{book}-real-api"));
    fs::create_dir_all(&work_dir).unwrap_or_else(|err| {
        panic!(
            "failed to create real-api test output directory {}: {:?}",
            work_dir.display(),
            err
        )
    });

    let output_file_name = format!("{book}.epub");
    let output = Command::new(env!("CARGO_BIN_EXE_wikipedia-to-epub"))
        .current_dir(&work_dir)
        .arg(repo.join(format!("examples/{book}.yaml")))
        .arg("--log")
        .arg("WARN")
        .output()
        .unwrap_or_else(|err| {
            panic!(
                "failed to execute wikipedia-to-epub binary for real-api book '{}' at {}: {:?}",
                book,
                work_dir.display(),
                err
            )
        });

    assert!(
        output.status.success(),
        "wikipedia-to-epub failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_cli_stdout(&output.stdout, &output_file_name);

    let output_file = work_dir.join(output_file_name);
    assert!(output_file.is_file());

    let mut epub = open_epub(&output_file);
    let expected_dir = repo.join("expected").join(book);
    let expected_entries = expected_epub_entries(&expected_dir);
    assert_eq!(zip_entries(&epub), expected_entries);

    for (index, title) in chapter_titles.iter().enumerate() {
        let filename = sanitize_chapter_filename(title);
        let chapter = read_epub_entry(&mut epub, &format!("OEBPS/{filename}"));
        let normalized = chapter.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            normalized.contains(&format!("<title> {title} </title>")),
            "chapter {} is missing expected title {title:?}\nchapter content:\n{}",
            index + 1,
            chapter
        );
        assert!(
            normalized.contains(&format!("<h1> {title} </h1>")),
            "chapter {} is missing expected heading {title:?}\nchapter content:\n{}",
            index + 1,
            chapter
        );
    }

    fs::remove_dir_all(&work_dir).unwrap_or_else(|err| {
        panic!(
            "failed to clean up real-api test output directory {}: {:?}",
            work_dir.display(),
            err
        )
    });
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn assert_cli_stdout(stdout: &[u8], output_file_name: &str) {
    let stdout = String::from_utf8_lossy(stdout);
    assert!(
        stdout.contains(&format!("Created {output_file_name}\n")),
        "stdout is missing created message:\n{stdout}"
    );
    assert!(
        Regex::new(r"(?m)^Skipped templates: recognized=\d+, unknown=\d+$")
            .unwrap()
            .is_match(&stdout),
        "stdout is missing skipped-template totals:\n{stdout}"
    );
}

fn open_epub(path: &Path) -> ZipArchive<File> {
    let file = File::open(path)
        .unwrap_or_else(|err| panic!("generated epub '{}' opens: {:?}", path.display(), err));
    ZipArchive::new(file).unwrap_or_else(|err| {
        panic!(
            "generated epub '{}' is a zip archive: {:?}",
            path.display(),
            err
        )
    })
}

fn zip_entries(epub: &ZipArchive<File>) -> Vec<String> {
    let mut entries = epub.file_names().map(str::to_string).collect::<Vec<_>>();
    entries.sort();
    entries
}

fn read_epub_entry<R: std::io::Read + std::io::Seek>(
    epub: &mut ZipArchive<R>,
    name: &str,
) -> String {
    let mut entry = epub
        .by_name(name)
        .unwrap_or_else(|err| panic!("epub entry '{}' exists: {:?}", name, err));
    let mut content = String::new();
    entry
        .read_to_string(&mut content)
        .unwrap_or_else(|err| panic!("epub entry '{}' is valid utf-8: {:?}", name, err));
    content
}

fn expected_epub_entries(expected_dir: &Path) -> Vec<String> {
    let mut entries = Vec::new();
    collect_expected_epub_entries(expected_dir, expected_dir, &mut entries);
    entries.sort();
    entries
}

fn normalize_epub_entry(name: &str, content: &str) -> String {
    if matches!(name, "OEBPS/content.opf" | "OEBPS/toc.ncx") {
        return Regex::new(r"urn:wikipedia-to-epub:\d+")
            .unwrap()
            .replace_all(content, "urn:wikipedia-to-epub:normalized")
            .into_owned();
    }

    content.to_string()
}

fn assert_text_matches_expected(entry_name: &str, generated: &str, expected: &str) {
    if generated == expected {
        return;
    }

    panic!(
        "EPUB entry differs: {entry_name}\n{}",
        first_difference_report(generated, expected)
    );
}

fn first_difference_report(generated: &str, expected: &str) -> String {
    let diff = TextDiff::from_chars(expected, generated);
    let first_change = diff
        .ops()
        .iter()
        .find(|op| !matches!(op.tag(), DiffTag::Equal))
        .unwrap_or_else(|| {
            panic!(
                "strings differ but no changed diff op found. Expected len: {} chars, generated len: {} chars",
                expected.chars().count(),
                generated.chars().count()
            )
        });

    let expected_index = first_change.old_range().start;
    let generated_index = first_change.new_range().start;
    let shared_index = expected_index.min(generated_index);
    let (line, column) = line_column_at(expected, shared_index);
    let expected_char = expected.chars().nth(expected_index);
    let generated_char = generated.chars().nth(generated_index);

    format!(
        "first difference at line {line}, column {column}\nexpected char: {:?}\ngenerated char: {:?}\nexpected context:\n{:?}\ngenerated context:\n{:?}\nexpected length: {} chars\ngenerated length: {} chars",
        expected_char,
        generated_char,
        snippet_at(expected, expected_index),
        snippet_at(generated, generated_index),
        expected.chars().count(),
        generated.chars().count(),
    )
}

fn line_column_at(text: &str, char_index: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;

    for ch in text.chars().take(char_index) {
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    (line, column)
}

fn snippet_at(text: &str, char_index: usize) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let before_start = char_index.saturating_sub(200);
    let after_end = (char_index + 200).min(chars.len());
    let before: String = chars[before_start..char_index].iter().collect();
    let after: String = chars[char_index..after_end].iter().collect();
    format!("{before}<HERE>{after}")
}

fn collect_expected_epub_entries(root: &Path, dir: &Path, entries: &mut Vec<String>) {
    for entry in fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("expected epub directory reads {dir:?}: {:?}", err))
    {
        let path = entry
            .unwrap_or_else(|err| {
                panic!(
                    "expected epub directory entry in {} reads: {:?}",
                    dir.display(),
                    err
                )
            })
            .path();
        if path.is_dir() {
            collect_expected_epub_entries(root, &path, entries);
        } else {
            let relative = path
                .strip_prefix(root)
                .unwrap_or_else(|err| {
                    panic!(
                        "expected path {} is under expected root {}: {:?}",
                        path.display(),
                        root.display(),
                        err
                    )
                })
                .to_string_lossy()
                .replace('\\', "/");
            entries.push(relative);
        }
    }
}

fn unique_test_dir(repo: &Path, test_name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|err| panic!("system time is before unix epoch: {:?}", err))
        .as_nanos();
    repo.join("target")
        .join("test-output")
        .join(format!("{test_name}-{}-{nanos}", std::process::id()))
}

#[test]
fn generate_hierarchical_book_from_local_page_dump() {
    let repo = repo_root();
    let work_dir = unique_test_dir(&repo, "hierarchical-book");
    fs::create_dir_all(&work_dir).unwrap();

    let config_path = work_dir.join("book.yaml");
    let yaml = r#"chapters: title
metadata:
  title: "Japan and Osaka"
  author: "Wikipedia contributors"
  language: en
  edition: First edition
output-file: output.epub
cover: "None"
links_to_pages: false
links_to_excluded_pages: emphasize
caching: none
depth: 0
articles:
  - "Japan"
  - title: "Osaka Info"
    type: "section"
    articles:
      - "Osaka"
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
    command.env("WIKIPEDIA_TO_EPUB_MOCK_DATE", "2026-06-06");
    let output = command.output().unwrap();

    assert!(output.status.success(), "run failed: {:?}", output);

    let epub_path = work_dir.join("output.epub");
    assert!(epub_path.exists(), "output.epub should be created");
    let report_path = work_dir.join("output.html");
    assert!(report_path.exists(), "output.html should be created");

    let zip_file = File::open(&epub_path).unwrap();
    let mut archive = ZipArchive::new(zip_file).unwrap();

    assert!(
        archive.by_name("OEBPS/Japan.xhtml").is_ok(),
        "Japan should exist"
    );
    assert!(
        archive.by_name("OEBPS/Osaka_Info.xhtml").is_ok(),
        "Osaka_Info should exist"
    );
    assert!(
        archive.by_name("OEBPS/Osaka.xhtml").is_ok(),
        "Osaka should exist"
    );

    let mut chapter2 = archive.by_name("OEBPS/Osaka_Info.xhtml").unwrap();
    let mut chapter2_content = String::new();
    chapter2.read_to_string(&mut chapter2_content).unwrap();
    assert!(
        Regex::new(r"<h1>\s*Osaka Info\s*</h1>")
            .unwrap()
            .is_match(&chapter2_content),
        "chapter2 should have section title"
    );

    let report = fs::read_to_string(&report_path).unwrap();
    assert!(report.contains("<h2>Included pages</h2>"), "{report}");
    assert!(report.contains("Japan"), "{report}");
    assert!(report.contains("Osaka Info"), "{report}");
    assert!(report.contains("Osaka"), "{report}");
    assert!(
        report.contains("https://en.wikipedia.org/wiki/Tokyo"),
        "{report}"
    );

    fs::remove_dir_all(&work_dir).unwrap();
}

#[test]
fn generate_numbered_chapters_book_from_local_page_dump() {
    let repo = repo_root();
    let work_dir = unique_test_dir(&repo, "numbered-book");
    fs::create_dir_all(&work_dir).unwrap();

    let config_path = work_dir.join("book.yaml");
    let yaml = r#"chapters: numbered-title
metadata:
  title: "Japan and Osaka"
  author: "Wikipedia contributors"
  language: en
  edition: First edition
output-file: output.epub
cover: "None"
links_to_pages: false
links_to_excluded_pages: emphasize
caching: none
depth: 0
articles:
  - "Japan"
  - title: "Osaka Info"
    type: "section"
    articles:
      - "Osaka"
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
    command.env("WIKIPEDIA_TO_EPUB_MOCK_DATE", "2026-06-06");
    let output = command.output().unwrap();

    assert!(output.status.success(), "run failed: {:?}", output);

    let epub_path = work_dir.join("output.epub");
    assert!(epub_path.exists(), "output.epub should be created");

    let zip_file = File::open(&epub_path).unwrap();
    let mut archive = ZipArchive::new(zip_file).unwrap();

    assert!(
        archive.by_name("OEBPS/Japan.xhtml").is_ok(),
        "Japan should exist"
    );
    assert!(
        archive.by_name("OEBPS/Osaka_Info.xhtml").is_ok(),
        "Osaka_Info should exist"
    );
    assert!(
        archive.by_name("OEBPS/Osaka.xhtml").is_ok(),
        "Osaka should exist"
    );

    {
        let mut chapter1 = archive.by_name("OEBPS/Japan.xhtml").unwrap();
        let mut chapter1_content = String::new();
        chapter1.read_to_string(&mut chapter1_content).unwrap();
        assert!(
            Regex::new(r"<h1>\s*1 Japan\s*</h1>")
                .unwrap()
                .is_match(&chapter1_content),
            "chapter1 should have numbered title"
        );
    }

    {
        let mut chapter2 = archive.by_name("OEBPS/Osaka_Info.xhtml").unwrap();
        let mut chapter2_content = String::new();
        chapter2.read_to_string(&mut chapter2_content).unwrap();
        assert!(
            Regex::new(r"<h1>\s*2 Osaka Info\s*</h1>")
                .unwrap()
                .is_match(&chapter2_content),
            "chapter2 should have numbered section title"
        );
    }

    {
        let mut chapter3 = archive.by_name("OEBPS/Osaka.xhtml").unwrap();
        let mut chapter3_content = String::new();
        chapter3.read_to_string(&mut chapter3_content).unwrap();
        assert!(
            Regex::new(r"<h1>\s*2.1 Osaka\s*</h1>")
                .unwrap()
                .is_match(&chapter3_content),
            "chapter3 should have nested numbered title"
        );
    }

    fs::remove_dir_all(&work_dir).unwrap();
}

#[test]
fn cli_output_flag_overrides_config_output_file() {
    let repo = repo_root();
    let work_dir = unique_test_dir(&repo, "cli-output-flag");
    fs::create_dir_all(&work_dir).unwrap();

    let config_path = work_dir.join("book.yaml");
    let yaml = r#"chapters: title
metadata:
  title: "Japan"
  author: "Wikipedia contributors"
  language: en
  edition: First edition
output-file: config-output.epub
cover: "None"
links_to_pages: false
links_to_excluded_pages: emphasize
caching: none
depth: 0
articles:
  - "Japan"
"#;
    fs::write(&config_path, yaml).unwrap();

    let overridden_output = work_dir.join("overridden.epub");

    let mut command = Command::new(env!("CARGO_BIN_EXE_wikipedia-to-epub"));
    command
        .current_dir(&work_dir)
        .arg(&config_path)
        .arg("--local")
        .arg(repo.join("pages"))
        .arg("--caching")
        .arg("none")
        .arg("--output")
        .arg(&overridden_output)
        .arg("--log")
        .arg("WARN");
    command.env("WIKIPEDIA_TO_EPUB_MOCK_DATE", "2026-06-06");
    let output = command.output().unwrap();

    assert!(output.status.success(), "run failed: {:?}", output);

    assert!(
        overridden_output.exists(),
        "overridden.epub should be created"
    );
    assert!(
        !work_dir.join("config-output.epub").exists(),
        "config-output.epub should NOT be created"
    );

    fs::remove_dir_all(&work_dir).unwrap();
}

fn sanitize_chapter_filename(title: &str) -> String {
    let ascii_title = any_ascii::any_ascii(title);
    let sanitized: String = ascii_title
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("{}.xhtml", sanitized)
}

fn extract_opf_date(expected_dir: &Path) -> Option<String> {
    let opf_path = expected_dir.join("OEBPS").join("content.opf");
    let content = fs::read_to_string(opf_path).ok()?;
    let start_tag = "<dc:date>";
    let end_tag = "</dc:date>";
    if let (Some(start_idx), Some(end_idx)) = (content.find(start_tag), content.find(end_tag)) {
        let start = start_idx + start_tag.len();
        if start < end_idx {
            return Some(content[start..end_idx].trim().to_string());
        }
    }
    None
}

#[test]
fn generate_kiso_mountains_book_from_local_page_dump() {
    assert_generated_book_matches_expected("Kiso_Mountains");
}

#[test]
fn generate_battle_of_sekigahara_book_from_local_page_dump() {
    assert_generated_book_matches_expected("Battle_of_Sekigahara");
}

#[test]
fn generate_statistical_model_book_from_local_page_dump() {
    assert_generated_book_matches_expected("Statistical_model");
}

#[test]
fn generate_variance_book_from_local_page_dump() {
    assert_generated_book_matches_expected("Variance");
}

#[test]
fn generate_statistics_book_from_local_page_dump() {
    assert_generated_book_matches_expected("Statistics");
}

#[test]
fn generate_normal_distribution_book_from_local_page_dump() {
    assert_generated_book_matches_expected("Normal_distribution");
}

#[test]
fn generate_standard_deviation_book_from_local_page_dump() {
    assert_generated_book_matches_expected("Standard_deviation");
}

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
fn generate_matsumoto_airport_book_from_local_page_dump() {
    assert_generated_book_matches_expected("Matsumoto_Airport");
}
