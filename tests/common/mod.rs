use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use zip::ZipArchive;

pub(crate) fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(crate) fn unique_test_dir(repo: &Path, test_name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|err| panic!("system time is before unix epoch: {:?}", err))
        .as_nanos();
    repo.join("target")
        .join("test-output")
        .join(format!("{test_name}-{}-{nanos}", std::process::id()))
}

pub(crate) fn open_epub(path: &Path) -> ZipArchive<File> {
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

pub(crate) fn zip_entries(epub: &ZipArchive<File>) -> Vec<String> {
    let mut entries = epub.file_names().map(str::to_string).collect::<Vec<_>>();
    entries.sort();
    entries
}

pub(crate) fn extract_opf_date(expected_dir: &Path) -> Option<String> {
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
