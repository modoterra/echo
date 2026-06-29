use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn php_compatibility_fixtures_parse() {
    let fixtures = php_fixture_dirs();
    if fixtures.is_empty() && fixture_filter_active() {
        return;
    }
    assert!(!fixtures.is_empty(), "expected at least one PHP fixture");

    for fixture in fixtures {
        let program_path = fixture.join("program.php");
        let stdin_path = fixture.join("stdin.txt");
        let stdout_path = fixture.join("stdout.txt");

        assert!(program_path.is_file(), "missing {}", program_path.display());
        assert!(stdin_path.is_file(), "missing {}", stdin_path.display());
        assert!(stdout_path.is_file(), "missing {}", stdout_path.display());

        let source = fs::read_to_string(&program_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", program_path.display()));

        echo_parser::parse(&source).unwrap_or_else(|diagnostics| {
            panic!(
                "{} failed to parse: {diagnostics:#?}",
                program_path.display()
            )
        });
    }
}

#[test]
fn echo_language_fixtures_parse() {
    let fixtures = echo_fixture_dirs();
    if fixtures.is_empty() && fixture_filter_active() {
        return;
    }
    assert!(!fixtures.is_empty(), "expected at least one Echo fixture");

    for fixture in fixtures {
        let program_path = fixture.join("program.echo");
        let unsupported_path = fixture.join("unsupported.txt");

        assert!(program_path.is_file(), "missing {}", program_path.display());
        if unsupported_path.is_file() {
            continue;
        }

        let source = fs::read_to_string(&program_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", program_path.display()));

        echo_parser::parse(&source).unwrap_or_else(|diagnostics| {
            panic!(
                "{} failed to parse: {diagnostics:#?}",
                program_path.display()
            )
        });
    }
}

fn php_fixture_dirs() -> Vec<PathBuf> {
    fixture_dirs("tests/php")
}

fn echo_fixture_dirs() -> Vec<PathBuf> {
    fixture_dirs("tests/echo")
}

fn fixture_dirs(relative: &str) -> Vec<PathBuf> {
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

fn fixture_filter_active() -> bool {
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

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("echo_parser should be two levels below the workspace root")
        .to_path_buf()
}
