use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn repo_root() -> PathBuf {
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
