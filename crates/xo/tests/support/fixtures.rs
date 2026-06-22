use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xo crate should be two levels below workspace root")
        .to_path_buf()
}

pub fn fixture_dirs(relative: &str) -> Vec<PathBuf> {
    let root = workspace_root().join(relative);
    let mut dirs = fs::read_dir(&root)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", root.display()))
        .map(|entry| entry.expect("failed to read fixture entry").path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();

    dirs.sort();
    if let Some(filter) = fixture_filter() {
        dirs.retain(|path| {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                return false;
            };

            filter.iter().any(|needle| name.contains(needle))
        });
    }
    dirs
}

pub fn fixture_filter_active() -> bool {
    fixture_filter().is_some()
}

fn fixture_filter() -> Option<Vec<String>> {
    let value = env::var("ECHO_FIXTURE").ok()?;
    let filters = value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();

    (!filters.is_empty()).then_some(filters)
}
