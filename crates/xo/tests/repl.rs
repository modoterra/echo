use std::process::Command;

mod support;

use support::{assert_output_success, assert_stdout_eq, output_with_stdin};

#[test]
fn piped_repl_runs_input_until_quit() {
    assert_repl_stdout(b"echo \"hi\";\n:quit\n", b"hi");
}

#[test]
fn piped_repl_prints_bare_expression_results() {
    let output = repl_output(b"5\n5+3\n3-5\n\"3.2\" + \"3.4\"\nnull + 5\n-2 ** 2\n:quit\n");

    assert_repl_success(&output);
    assert_stdout_eq(&output.stdout, b"58-26.65-4", "xo repl");
    assert!(
        !repl_stderr(&output).contains("Finished `dev` profile"),
        "repl should not print internal cargo build status"
    );
}

#[test]
fn piped_repl_prints_bare_function_call_results() {
    let output = repl_output(
        b"abs(-7)\nbasename(\"/etc/passwd\")\ndirname(\"/etc/passwd\")\ndechex(47)\ndecbin(26)\ndecoct(264)\nescapeshellarg(\"a b\")\nescapeshellcmd(\"a;b\")\ncount(explode(\",\", \"a,b\"))\nfile_exists(\"Cargo.toml\")\nis_dir(\".\")\nis_file(\"Cargo.toml\")\nis_link(\"/proc/self/exe\")\nis_float(42)\nis_double(\"4.2\")\nfunction_exists(\"is_float\")\nis_finite(42)\nis_finite(\"1e9999\")\nis_infinite(\"1e9999\")\nis_nan(\"1e9999\")\nis_object({ test: 5 })\nfunction_exists(\"is_object\")\narray_is_list([1, 2])\nfunction_exists(\"is_resource\")\nis_callable(\"strlen\")\n:quit\n",
    );

    assert_repl_success(&output);
    assert_stdout_eq(
        &output.stdout,
        b"7passwd/etc2f11010410'a b'a\\;b2111111111111",
        "xo repl",
    );
}

#[test]
fn piped_repl_keeps_statement_context() {
    assert_repl_stdout(b"let $a = \"ok\";\necho $a;\n:quit\n", b"ok");
}

#[test]
fn piped_repl_echo_statement_keeps_php_array_output() {
    assert_repl_stdout(b"let $a = [];\n$a[] = 2;\necho $a;\n:quit\n", b"Array");
}

#[test]
fn piped_repl_keeps_array_after_append() {
    let output = repl_output(b"let $a = [];\n$a;\n$a[] = 2;\n$a\n:quit\n");

    assert_repl_success(&output);
    assert_stdout_eq(&output.stdout, b"Array []Array [0 => 2]", "xo repl");
    assert!(
        !repl_stderr(&output).contains("requires array target"),
        "array append should not produce semantic diagnostic, got:\n{}",
        repl_stderr(&output)
    );
}

#[test]
fn piped_repl_reads_array_and_list_indexes() {
    assert_repl_stdout(b"let $a = [];\n$a[] = 4;\n$a[0];\n{7}[0];\n:quit\n", b"47");
}

#[test]
fn piped_repl_rejects_object_index_access() {
    let output = repl_output(b"{ a: 3 }[\"a\"];\n:quit\n");

    assert_repl_success(&output);
    assert_stdout_eq(&output.stdout, b"", "xo repl");
    assert_repl_stderr_contains(
        &output,
        "index access requires array or list target, found object",
    );
}

#[test]
fn piped_repl_rejects_list_append_without_corrupting_list() {
    let output = repl_output(b"let $a = {};\n$a;\n$a[] = 2;\n$a\n:quit\n");

    assert_repl_success(&output);
    assert_stdout_eq(&output.stdout, b"List []List []", "xo repl");
    assert_repl_stderr_contains(
        &output,
        "PHP array append syntax requires array target, found list",
    );
}

#[test]
fn piped_repl_reports_diagnostics_and_continues() {
    let output = repl_output(b"echo ;\necho \"ok\";\n:exit\n");

    assert_repl_success(&output);
    assert_stdout_eq(&output.stdout, b"ok", "xo repl");
    assert_repl_stderr_contains(&output, "error:");
}

#[test]
fn piped_repl_can_join_spawned_process_handle_later() {
    assert_repl_stdout(b"$proc = spawn \"exit 7\"\njoin $proc\n:quit\n", b"7");
}

fn assert_repl_stdout(input: &[u8], expected_stdout: &[u8]) {
    let output = repl_output(input);
    assert_repl_success(&output);
    assert_stdout_eq(&output.stdout, expected_stdout, "xo repl");
}

fn assert_repl_success(output: &std::process::Output) {
    assert_output_success(output, "xo repl");
}

fn assert_repl_stderr_contains(output: &std::process::Output, expected: &str) {
    let stderr = repl_stderr(output);
    assert!(
        stderr.contains(expected),
        "expected REPL stderr to contain {expected:?}, got:\n{stderr}"
    );
}

fn repl_stderr(output: &std::process::Output) -> std::borrow::Cow<'_, str> {
    String::from_utf8_lossy(&output.stderr)
}

fn repl_output(input: &[u8]) -> std::process::Output {
    output_with_stdin(Command::new(env!("CARGO_BIN_EXE_xo")).arg("repl"), input)
}
