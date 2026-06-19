use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[test]
fn run_jit_executes_supported_php_fixtures() {
    for fixture in fixture_dirs("tests/php") {
        if fixture.join("unsupported.txt").is_file() {
            continue;
        }

        assert_jit_fixture(&fixture, "program.php");
    }
}

#[test]
fn run_jit_executes_supported_echo_fixtures() {
    for fixture in fixture_dirs("tests/echo") {
        if fixture.join("unsupported.txt").is_file() {
            continue;
        }

        assert_jit_fixture(&fixture, "program.echo");
    }
}

fn assert_jit_fixture(fixture: &Path, program_file: &str) {
    let program_path = fixture.join(program_file);
    let stdin = fs::read(fixture.join("stdin.txt")).expect("fixture stdin should be readable");
    let expected_stdout =
        fs::read(fixture.join("stdout.txt")).expect("fixture stdout should be readable");

    let output = output_with_stdin(
        Command::new(env!("CARGO_BIN_EXE_xo"))
            .arg("run")
            .arg("--jit")
            .arg(&program_path),
        &stdin,
    );

    assert!(
        output.status.success(),
        "xo run --jit failed for {}\nstdout:\n{}\nstderr:\n{}",
        program_path.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, expected_stdout, "{}", program_path.display());
}

fn fixture_dirs(relative: &str) -> Vec<PathBuf> {
    let root = workspace_root().join(relative);
    let mut dirs = fs::read_dir(&root)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", root.display()))
        .map(|entry| entry.expect("failed to read fixture entry").path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();

    dirs.sort();
    dirs
}

fn output_with_stdin(command: &mut Command, stdin: &[u8]) -> std::process::Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|err| panic!("failed to run {command:?}: {err}"));

    use std::io::Write;
    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(stdin)
        .expect("failed to write fixture stdin");

    child
        .wait_with_output()
        .expect("failed to wait for command")
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xo crate should be two levels below workspace root")
}
