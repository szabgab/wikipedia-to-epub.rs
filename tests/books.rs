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
fn generate_japan_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("japan");
}

#[test]
fn generate_korea_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("korea");
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
fn generate_spanish_corea_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("es-corea");
}

#[test]
fn generate_hebrew_korea_book_from_local_page_dumps() {
    assert_generated_book_matches_expected("he-korea");
}

#[test]
fn generate_busan_images_book_from_local_page_dump() {
    assert_generated_book_matches_expected("busan-images");
}

#[test]
#[ignore = "hits the real Wikipedia API"]
fn generate_example_books_from_real_wikipedia_api() {
    assert_real_api_generates_book("korea", &["Korea", "Seoul"]);
    assert_real_api_generates_book("macchini", &["Macchini", "Licia Macchini"]);
}

fn assert_generated_book_matches_expected(book: &str) {
    let repo = repo_root();
    let work_dir = unique_test_dir(&repo, book);
    fs::create_dir_all(&work_dir).expect("test output directory is created");

    let output_file_name = format!("{book}.epub");
    let output = Command::new(env!("CARGO_BIN_EXE_wikipedia-to-epub"))
        .current_dir(&work_dir)
        .arg(repo.join(format!("examples/{book}.yaml")))
        .arg("--local")
        .arg(repo.join("pages"))
        .arg("--log")
        .arg("WARN")
        .output()
        .expect("wikipedia-to-epub runs");

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

    for entry_name in expected_entries {
        let generated = normalize_epub_entry(&entry_name, &read_epub_entry(&mut epub, &entry_name));
        let expected_path = expected_dir.join(&entry_name);
        let expected = normalize_epub_entry(
            &entry_name,
            &fs::read_to_string(&expected_path).expect("expected epub entry reads"),
        );
        assert_text_matches_expected(&entry_name, &generated, &expected);
    }

    fs::remove_dir_all(&work_dir).expect("test output directory is cleaned up");
}

fn assert_real_api_generates_book(book: &str, chapter_titles: &[&str]) {
    let repo = repo_root();
    let work_dir = unique_test_dir(&repo, &format!("{book}-real-api"));
    fs::create_dir_all(&work_dir).expect("test output directory is created");

    let output_file_name = format!("{book}.epub");
    let output = Command::new(env!("CARGO_BIN_EXE_wikipedia-to-epub"))
        .current_dir(&work_dir)
        .arg(repo.join(format!("examples/{book}.yaml")))
        .arg("--log")
        .arg("WARN")
        .output()
        .expect("wikipedia-to-epub runs");

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
        let chapter = read_epub_entry(&mut epub, &format!("OEBPS/chapter-{}.xhtml", index + 1));
        assert!(
            chapter.contains(&format!("<title>{title}</title>")),
            "chapter {} is missing expected title {title:?}",
            index + 1
        );
        assert!(
            chapter.contains(&format!("<h1>{title}</h1>")),
            "chapter {} is missing expected heading {title:?}",
            index + 1
        );
    }

    fs::remove_dir_all(&work_dir).expect("test output directory is cleaned up");
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
    let file = File::open(path).expect("generated epub opens");
    ZipArchive::new(file).expect("generated epub is a zip archive")
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
    let mut entry = epub.by_name(name).expect("epub entry exists");
    let mut content = String::new();
    entry
        .read_to_string(&mut content)
        .expect("epub entry is valid utf-8");
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
        .expect("strings differ, so a changed diff op must exist");

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
    for entry in fs::read_dir(dir).expect(format!("expected epub directory reads {dir:?}").as_str())
    {
        let path = entry.expect("expected epub directory entry reads").path();
        if path.is_dir() {
            collect_expected_epub_entries(root, &path, entries);
        } else {
            let relative = path
                .strip_prefix(root)
                .expect("expected path is under expected root")
                .to_string_lossy()
                .replace('\\', "/");
            entries.push(relative);
        }
    }
}

fn unique_test_dir(repo: &Path, test_name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is after unix epoch")
        .as_nanos();
    repo.join("target")
        .join("test-output")
        .join(format!("{test_name}-{}-{nanos}", std::process::id()))
}
