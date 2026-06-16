use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn php_compatibility_fixtures_parse() {
    let fixtures = fixture_dirs();
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

fn fixture_dirs() -> Vec<PathBuf> {
    let root = workspace_root().join("tests/php");
    let mut dirs = fs::read_dir(&root)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", root.display()))
        .map(|entry| entry.expect("failed to read fixture entry").path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();

    dirs.sort();
    dirs
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("echo_parser should be two levels below the workspace root")
        .to_path_buf()
}
