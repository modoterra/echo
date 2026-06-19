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
    let output = repl_output(b"5\n5+3\n3-5\n\"3.2\" + \"3.4\"\nnull + 5\n-2 ** 2\n:quit\n");

    assert!(
        output.status.success(),
        "xo repl failed with status {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"58-26.65-4");
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("Finished `dev` profile"),
        "repl should not print internal cargo build status"
    );
}

#[test]
fn piped_repl_prints_bare_function_call_results() {
    let output = repl_output(
        b"abs(-7)\nbasename(\"/etc/passwd\")\ndirname(\"/etc/passwd\")\ndechex(47)\ndecbin(26)\ndecoct(264)\nescapeshellarg(\"a b\")\nescapeshellcmd(\"a;b\")\ncount(explode(\",\", \"a,b\"))\nfile_exists(\"Cargo.toml\")\nis_dir(\".\")\nis_file(\"Cargo.toml\")\nis_link(\"/proc/self/exe\")\nis_float(42)\nis_double(\"4.2\")\nfunction_exists(\"is_float\")\nis_finite(42)\nis_finite(\"1e9999\")\nis_infinite(\"1e9999\")\nis_nan(\"1e9999\")\nis_object({ test: 5 })\nfunction_exists(\"is_object\")\narray_is_list([1, 2])\nfunction_exists(\"is_resource\")\nis_callable(\"strlen\")\n:quit\n",
    );

    assert!(
        output.status.success(),
        "xo repl failed with status {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout,
        b"7passwd/etc2f11010410'a b'a\\;b2111111111111"
    );
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
fn piped_repl_keeps_array_after_append() {
    let output = repl_output(b"let $a = [];\n$a;\n$a[] = 2;\n$a\n:quit\n");

    assert!(
        output.status.success(),
        "xo repl failed with status {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"ArrayArray");
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("requires array target"),
        "array append should not produce semantic diagnostic, got:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn piped_repl_rejects_list_append_without_corrupting_list() {
    let output = repl_output(b"let $a = {};\n$a;\n$a[] = 2;\n$a\n:quit\n");

    assert!(
        output.status.success(),
        "xo repl failed with status {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"ListList");
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("PHP array append syntax requires array target, found list"),
        "expected list append diagnostic on stderr, got:\n{}",
        String::from_utf8_lossy(&output.stderr)
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
