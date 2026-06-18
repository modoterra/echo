use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn piped_repl_runs_input_until_quit() {
    let output = repl_output(b"echo \"hi\";\n:quit\n");

    assert!(
        output.status.success(),
        "xo repl failed with status {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"hi");
}

#[test]
fn piped_repl_prints_bare_expression_results() {
    let output = repl_output(b"5\n5+3\n:quit\n");

    assert!(
        output.status.success(),
        "xo repl failed with status {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"58");
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("Finished `dev` profile"),
        "repl should not print internal cargo build status"
    );
}

#[test]
fn piped_repl_reports_diagnostics_and_continues() {
    let output = repl_output(b"echo ;\necho \"ok\";\n:exit\n");

    assert!(
        output.status.success(),
        "xo repl failed with status {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"ok");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("error:"),
        "expected a diagnostic on stderr"
    );
}

fn repl_output(input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_xo"))
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn xo repl");

    child
        .stdin
        .as_mut()
        .expect("repl stdin should be piped")
        .write_all(input)
        .expect("failed to write repl input");

    child
        .wait_with_output()
        .expect("failed to wait for xo repl")
}
