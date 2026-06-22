use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::{Mutex, MutexGuard};

mod support;

use support::{
    RunArtifacts, artifact_dir, assert_output_success, assert_stdout_eq, command_output,
    fixture_dirs, fixture_filter_active, output_with_stdin, reset_dir, run_artifact_dir,
    write_artifact, write_empty_execution_artifacts,
};

static FIXTURE_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn php_fixtures_work_end_to_end() {
    let _lock = fixture_lock();
    let _run_artifacts = RunArtifacts::new("php");
    let fixtures = fixture_dirs("tests/php");
    if fixtures.is_empty() && fixture_filter_active() {
        return;
    }
    assert!(!fixtures.is_empty(), "expected at least one PHP fixture");

    for fixture in fixtures {
        let program_path = fixture.join("program.php");
        let stdin_path = fixture.join("stdin.txt");
        let stdout_path = fixture.join("stdout.txt");
        let unsupported_path = fixture.join("unsupported.txt");
        let artifact_dir = artifact_dir("php", &fixture);

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
            write_empty_execution_artifacts(&artifact_dir);
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
        assert_stdout_eq(
            &run_output.stdout,
            &expected_stdout,
            &program_path.display().to_string(),
        );

        let binary_dir = run_artifact_dir("php", &fixture);
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
        assert_stdout_eq(
            &binary_output.stdout,
            &expected_stdout,
            &binary_path.display().to_string(),
        );
    }
}

#[test]
fn echo_fixtures_are_exercised() {
    let _lock = fixture_lock();
    let _run_artifacts = RunArtifacts::new("echo");
    let fixtures = fixture_dirs("tests/echo");
    if fixtures.is_empty() && fixture_filter_active() {
        return;
    }
    assert!(!fixtures.is_empty(), "expected at least one Echo fixture");

    for fixture in fixtures {
        let program_path = fixture.join("program.echo");
        let stdin_path = fixture.join("stdin.txt");
        let stdout_path = fixture.join("stdout.txt");
        let unsupported_path = fixture.join("unsupported.txt");
        let artifact_dir = artifact_dir("echo", &fixture);

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
        if unsupported {
            write_empty_execution_artifacts(&artifact_dir);
            continue;
        }
        assert_output_success(&ast_output, "xo ast");

        let ir_output = command_output(command("ir", &program_path));
        write_artifact(&artifact_dir.join("ir.ll"), &ir_output.stdout);

        let mut run = command("run", &program_path);
        let run_output = output_with_stdin(&mut run, &stdin);
        write_artifact(&artifact_dir.join("run.stdout"), &run_output.stdout);
        write_artifact(&artifact_dir.join("run.stderr"), &run_output.stderr);

        let binary_dir = run_artifact_dir("echo", &fixture);
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
        assert_stdout_eq(
            &run_output.stdout,
            &expected_stdout,
            &program_path.display().to_string(),
        );
        assert_output_success(&build_output, "xo build");

        let binary_output = output_with_stdin(&mut Command::new(&binary_path), &stdin);
        assert_output_success(&binary_output, &format!("{}", binary_path.display()));
        write_artifact(&artifact_dir.join("binary.stdout"), &binary_output.stdout);
        write_artifact(&artifact_dir.join("binary.stderr"), &binary_output.stderr);
        assert_stdout_eq(
            &binary_output.stdout,
            &expected_stdout,
            &binary_path.display().to_string(),
        );
    }
}

fn fixture_lock() -> MutexGuard<'static, ()> {
    FIXTURE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
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
