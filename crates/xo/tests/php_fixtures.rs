use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[test]
fn php_fixtures_work_end_to_end() {
    let fixtures = fixture_dirs("tests/php");
    assert!(!fixtures.is_empty(), "expected at least one PHP fixture");

    for fixture in fixtures {
        let program_path = fixture.join("program.php");
        let stdin_path = fixture.join("stdin.txt");
        let stdout_path = fixture.join("stdout.txt");
        let unsupported_path = fixture.join("unsupported.txt");
        let artifact_dir = artifact_dir_for(&fixture);

        assert!(program_path.is_file(), "missing {}", program_path.display());
        assert!(stdin_path.is_file(), "missing {}", stdin_path.display());
        assert!(stdout_path.is_file(), "missing {}", stdout_path.display());

        let unsupported = unsupported_path.is_file();
        let expected_stdout = fs::read(&stdout_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", stdout_path.display()));
        let stdin = fs::read(&stdin_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", stdin_path.display()));

        reset_dir(&artifact_dir);

        let ast_output = command_output(command("ast", &program_path));
        assert_output_success(&ast_output, "xo ast");
        write_artifact(&artifact_dir.join("ast.txt"), &ast_output.stdout);

        if unsupported {
            write_artifact(&artifact_dir.join("ir.ll"), b"");
            write_artifact(&artifact_dir.join("run.stdout"), b"");
            write_artifact(&artifact_dir.join("run.stderr"), b"");
            write_artifact(&artifact_dir.join("binary.stdout"), b"");
            write_artifact(&artifact_dir.join("binary.stderr"), b"");
            continue;
        }

        let ir_output = command_output(command("ir", &program_path));
        assert_output_success(&ir_output, "xo ir");
        write_artifact(&artifact_dir.join("ir.ll"), &ir_output.stdout);

        let mut run = command("run", &program_path);
        let run_output = output_with_stdin(&mut run, &stdin);
        assert_output_success(&run_output, "xo run");
        write_artifact(&artifact_dir.join("run.stdout"), &run_output.stdout);
        write_artifact(&artifact_dir.join("run.stderr"), &run_output.stderr);
        assert_eq!(
            run_output.stdout,
            expected_stdout,
            "{}",
            program_path.display()
        );

        let binary_dir = run_artifact_dir_for(&fixture);
        fs::create_dir_all(&binary_dir)
            .unwrap_or_else(|err| panic!("failed to create {}: {err}", binary_dir.display()));
        let binary_path = binary_dir.join("program");
        let mut build = Command::new(env!("CARGO_BIN_EXE_xo"));
        build
            .arg("build")
            .arg(&program_path)
            .arg("-o")
            .arg(&binary_path);
        assert_command_success(build);

        let binary_output = output_with_stdin(&mut Command::new(&binary_path), &stdin);
        assert_output_success(&binary_output, &format!("{}", binary_path.display()));
        write_artifact(&artifact_dir.join("binary.stdout"), &binary_output.stdout);
        write_artifact(&artifact_dir.join("binary.stderr"), &binary_output.stderr);
        assert_eq!(
            binary_output.stdout,
            expected_stdout,
            "{}",
            binary_path.display()
        );
    }
}

#[test]
fn echo_fixtures_are_exercised() {
    let fixtures = fixture_dirs("tests/echo");
    assert!(!fixtures.is_empty(), "expected at least one Echo fixture");

    for fixture in fixtures {
        let program_path = fixture.join("program.echo");
        let stdin_path = fixture.join("stdin.txt");
        let stdout_path = fixture.join("stdout.txt");
        let unsupported_path = fixture.join("unsupported.txt");
        let artifact_dir = echo_artifact_dir_for(&fixture);

        assert!(program_path.is_file(), "missing {}", program_path.display());
        assert!(stdin_path.is_file(), "missing {}", stdin_path.display());
        assert!(stdout_path.is_file(), "missing {}", stdout_path.display());

        let stdin = fs::read(&stdin_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", stdin_path.display()));
        let expected_stdout = fs::read(&stdout_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", stdout_path.display()));
        let unsupported = unsupported_path.is_file();

        reset_dir(&artifact_dir);

        let ast_output = command_output(command("ast", &program_path));
        write_artifact(&artifact_dir.join("ast.txt"), &ast_output.stdout);
        assert_output_success(&ast_output, "xo ast");

        let ir_output = command_output(command("ir", &program_path));
        write_artifact(&artifact_dir.join("ir.ll"), &ir_output.stdout);

        if unsupported {
            write_artifact(&artifact_dir.join("run.stdout"), b"");
            write_artifact(&artifact_dir.join("run.stderr"), b"");
            write_artifact(&artifact_dir.join("binary.stdout"), b"");
            write_artifact(&artifact_dir.join("binary.stderr"), b"");
            continue;
        }

        let mut run = command("run", &program_path);
        let run_output = output_with_stdin(&mut run, &stdin);
        write_artifact(&artifact_dir.join("run.stdout"), &run_output.stdout);
        write_artifact(&artifact_dir.join("run.stderr"), &run_output.stderr);

        let binary_dir = echo_run_artifact_dir_for(&fixture);
        fs::create_dir_all(&binary_dir)
            .unwrap_or_else(|err| panic!("failed to create {}: {err}", binary_dir.display()));
        let binary_path = binary_dir.join("program");
        let mut build = Command::new(env!("CARGO_BIN_EXE_xo"));
        build
            .arg("build")
            .arg(&program_path)
            .arg("-o")
            .arg(&binary_path);
        let build_output = command_output(build);

        assert_output_success(&ir_output, "xo ir");
        assert_output_success(&run_output, "xo run");
        assert_eq!(
            run_output.stdout,
            expected_stdout,
            "{}",
            program_path.display()
        );
        assert_output_success(&build_output, "xo build");

        let binary_output = output_with_stdin(&mut Command::new(&binary_path), &stdin);
        assert_output_success(&binary_output, &format!("{}", binary_path.display()));
        write_artifact(&artifact_dir.join("binary.stdout"), &binary_output.stdout);
        write_artifact(&artifact_dir.join("binary.stderr"), &binary_output.stderr);
        assert_eq!(
            binary_output.stdout,
            expected_stdout,
            "{}",
            binary_path.display()
        );
    }
}

fn command(subcommand: &str, program_path: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_xo"));
    command.arg(subcommand).arg(program_path);
    command
}

fn assert_command_success(command: Command) {
    let label = format!("{command:?}");
    let output = command_output(command);

    assert_output_success(&output, &label);
}

fn command_output(mut command: Command) -> std::process::Output {
    command
        .output()
        .unwrap_or_else(|err| panic!("failed to run {command:?}: {err}"))
}

fn output_with_stdin(command: &mut Command, stdin: &[u8]) -> std::process::Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|err| panic!("failed to run {command:?}: {err}"));

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

fn assert_output_success(output: &std::process::Output, label: &str) {
    assert!(
        output.status.success(),
        "{label} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
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

fn artifact_dir_for(fixture: &Path) -> PathBuf {
    let name = fixture
        .file_name()
        .and_then(|name| name.to_str())
        .expect("fixture path should have UTF-8 file name");

    workspace_root().join("test-results/php").join(name)
}

fn echo_artifact_dir_for(fixture: &Path) -> PathBuf {
    let name = fixture
        .file_name()
        .and_then(|name| name.to_str())
        .expect("fixture path should have UTF-8 file name");

    workspace_root().join("test-results/echo").join(name)
}

fn run_artifact_dir_for(fixture: &Path) -> PathBuf {
    let name = fixture
        .file_name()
        .and_then(|name| name.to_str())
        .expect("fixture path should have UTF-8 file name");

    workspace_root()
        .join("test-results/php/.runs")
        .join(std::process::id().to_string())
        .join(name)
}

fn echo_run_artifact_dir_for(fixture: &Path) -> PathBuf {
    let name = fixture
        .file_name()
        .and_then(|name| name.to_str())
        .expect("fixture path should have UTF-8 file name");

    workspace_root()
        .join("test-results/echo/.runs")
        .join(std::process::id().to_string())
        .join(name)
}

fn reset_dir(path: &Path) {
    if path.exists() {
        fs::remove_dir_all(path)
            .unwrap_or_else(|err| panic!("failed to remove {}: {err}", path.display()));
    }

    fs::create_dir_all(path)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", path.display()));
}

fn write_artifact(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xo should be two levels below the workspace root")
        .to_path_buf()
}
