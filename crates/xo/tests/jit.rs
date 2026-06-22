use std::fs;
use std::path::Path;
use std::process::Command;

mod support;

use support::{
    assert_output_success, assert_stdout_eq, fixture_dirs, fixture_filter_active, output_with_stdin,
};

#[test]
fn run_jit_executes_supported_php_fixtures() {
    let fixtures = fixture_dirs("tests/php");
    if fixtures.is_empty() && fixture_filter_active() {
        return;
    }

    for fixture in fixtures {
        if fixture.join("unsupported.txt").is_file() {
            continue;
        }

        assert_jit_fixture(&fixture, "program.php");
    }
}

#[test]
fn run_jit_executes_supported_echo_fixtures() {
    let fixtures = fixture_dirs("tests/echo");
    if fixtures.is_empty() && fixture_filter_active() {
        return;
    }

    for fixture in fixtures {
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

    assert_output_success(
        &output,
        &format!("xo run --jit for {}", program_path.display()),
    );
    assert_stdout_eq(
        &output.stdout,
        &expected_stdout,
        &program_path.display().to_string(),
    );
}
