use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use regex::Regex;
use zip::ZipArchive;

#[test]
fn generate_macchini_book_from_local_page_dump() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let work_dir = unique_test_dir(repo);
    fs::create_dir_all(&work_dir).expect("test output directory is created");

    let output = Command::new(env!("CARGO_BIN_EXE_wikipedia-to-epub"))
        .current_dir(&work_dir)
        .arg(repo.join("examples/macchini.yaml"))
        .arg("--local")
        .arg(repo.join("pages"))
        .output()
        .expect("wikipedia-to-epub runs");

    assert!(
        output.status.success(),
        "wikipedia-to-epub failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "Created macchini.epub\n"
    );

    let output_file = work_dir.join("macchini.epub");
    assert!(output_file.is_file());

    let file = File::open(&output_file).expect("generated epub opens");
    let mut epub = ZipArchive::new(file).expect("generated epub is a zip archive");
    let expected_dir = repo.join("expected/macchini");
    let expected_entries = expected_epub_entries(&expected_dir);
    let mut generated_entries = epub
        .file_names()
        .map(str::to_string)
        .collect::<Vec<String>>();
    generated_entries.sort();
    assert_eq!(generated_entries, expected_entries);

    for entry_name in expected_entries {
        let generated = normalize_epub_entry(&entry_name, &read_epub_entry(&mut epub, &entry_name));
        let expected_path = expected_dir.join(&entry_name);
        let expected = normalize_epub_entry(
            &entry_name,
            &fs::read_to_string(&expected_path).expect("expected epub entry reads"),
        );
        assert_eq!(generated, expected, "EPUB entry differs: {entry_name}");
    }

    fs::remove_dir_all(&work_dir).expect("test output directory is cleaned up");
}

fn expected_epub_entries(expected_dir: &Path) -> Vec<String> {
    let mut entries = Vec::new();
    collect_expected_epub_entries(expected_dir, expected_dir, &mut entries);
    entries.sort();
    entries
}

fn collect_expected_epub_entries(root: &Path, dir: &Path, entries: &mut Vec<String>) {
    for entry in fs::read_dir(dir).expect("expected epub directory reads") {
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

fn normalize_epub_entry(name: &str, content: &str) -> String {
    if matches!(name, "OEBPS/content.opf" | "OEBPS/toc.ncx") {
        return Regex::new(r"urn:wikipedia-to-epub:\d+")
            .unwrap()
            .replace_all(content, "urn:wikipedia-to-epub:normalized")
            .into_owned();
    }

    content.to_string()
}

fn unique_test_dir(repo: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is after unix epoch")
        .as_nanos();
    repo.join("target")
        .join("test-output")
        .join(format!("macchini-{}-{nanos}", std::process::id()))
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
