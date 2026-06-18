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
fn piped_repl_prints_bare_function_call_results() {
    let output = repl_output(
        b"abs(-7)\nbasename(\"/etc/passwd\")\ndirname(\"/etc/passwd\")\ndechex(47)\ndecbin(26)\ndecoct(264)\nis_float(42)\nis_double(\"4.2\")\nfunction_exists(\"is_float\")\nis_finite(42)\nis_finite(\"1e9999\")\nis_infinite(\"1e9999\")\nis_nan(\"1e9999\")\nis_object({ test: 5 })\nfunction_exists(\"is_object\")\narray_is_list([1, 2])\nfunction_exists(\"is_resource\")\nis_callable(\"strlen\")\n:quit\n",
    );

    assert!(
        output.status.success(),
        "xo repl failed with status {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"7passwd/etc2f1101041011111111");
}

#[test]
fn piped_repl_keeps_statement_context() {
    let output = repl_output(b"let $a = \"ok\";\necho $a;\n:quit\n");

    assert!(
        output.status.success(),
        "xo repl failed with status {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"ok");
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
