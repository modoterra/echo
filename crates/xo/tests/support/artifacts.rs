use std::fs;
use std::path::{Path, PathBuf};

use super::fixtures::workspace_root;

pub struct RunArtifacts {
    current_run_dir: PathBuf,
}

impl RunArtifacts {
    pub fn new(suite: &str) -> Self {
        let root = workspace_root()
            .join("test-results")
            .join(suite)
            .join(".runs");
        fs::create_dir_all(&root)
            .unwrap_or_else(|err| panic!("failed to create {}: {err}", root.display()));
        remove_stale_run_dirs(&root);
        let current_run_dir = root.join(std::process::id().to_string());
        fs::create_dir_all(&current_run_dir)
            .unwrap_or_else(|err| panic!("failed to create {}: {err}", current_run_dir.display()));

        Self { current_run_dir }
    }
}

impl Drop for RunArtifacts {
    fn drop(&mut self) {
        if self.current_run_dir.exists() {
            fs::remove_dir_all(&self.current_run_dir).unwrap_or_else(|err| {
                panic!("failed to remove {}: {err}", self.current_run_dir.display())
            });
        }
    }
}

pub fn artifact_dir(suite: &str, fixture: &Path) -> PathBuf {
    artifact_dir_for_label(suite, fixture_name(fixture))
}

pub fn artifact_dir_for_label(suite: &str, label: &str) -> PathBuf {
    workspace_root()
        .join("test-results")
        .join(suite)
        .join(label)
}

pub fn run_artifact_dir(suite: &str, fixture: &Path) -> PathBuf {
    run_artifact_dir_for_label(suite, fixture_name(fixture))
}

pub fn run_artifact_dir_for_label(suite: &str, label: &str) -> PathBuf {
    workspace_root()
        .join("test-results")
        .join(suite)
        .join(".runs")
        .join(std::process::id().to_string())
        .join(label)
}

pub fn reset_dir(path: &Path) {
    if path.exists() {
        fs::remove_dir_all(path)
            .unwrap_or_else(|err| panic!("failed to remove {}: {err}", path.display()));
    }

    fs::create_dir_all(path)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", path.display()));
}

pub fn write_artifact(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
}

pub fn write_empty_execution_artifacts(artifact_dir: &Path) {
    write_artifact(&artifact_dir.join("ir.ll"), b"");
    write_artifact(&artifact_dir.join("run.stdout"), b"");
    write_artifact(&artifact_dir.join("run.stderr"), b"");
    write_artifact(&artifact_dir.join("binary.stdout"), b"");
    write_artifact(&artifact_dir.join("binary.stderr"), b"");
}

fn fixture_name(fixture: &Path) -> &str {
    fixture
        .file_name()
        .and_then(|name| name.to_str())
        .expect("fixture path should have UTF-8 file name")
}

fn remove_stale_run_dirs(root: &Path) {
    let current_pid = std::process::id().to_string();
    for entry in
        fs::read_dir(root).unwrap_or_else(|err| panic!("failed to read {}: {err}", root.display()))
    {
        let path = entry.expect("failed to read run artifact entry").path();
        let is_current = path.file_name().and_then(|name| name.to_str()) == Some(&current_pid);
        if path.is_dir() && !is_current {
            fs::remove_dir_all(&path)
                .unwrap_or_else(|err| panic!("failed to remove {}: {err}", path.display()));
        }
    }
}
