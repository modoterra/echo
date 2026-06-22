use std::io::{self, Write};
use std::process::{Command, Stdio};

pub fn command_output(mut command: Command) -> std::process::Output {
    command
        .output()
        .unwrap_or_else(|err| panic!("failed to run {command:?}: {err}"))
}

pub fn output_with_stdin(command: &mut Command, stdin: &[u8]) -> std::process::Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|err| panic!("failed to run {command:?}: {err}"));

    write_fixture_stdin(child.stdin.as_mut().expect("stdin should be piped"), stdin);

    child
        .wait_with_output()
        .expect("failed to wait for command")
}

pub fn write_fixture_stdin(writer: &mut impl Write, stdin: &[u8]) {
    match writer.write_all(stdin) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => {}
        Err(err) => panic!("failed to write fixture stdin: {err}"),
    }
}

pub fn assert_output_success(output: &std::process::Output, label: &str) {
    assert!(
        output.status.success(),
        "{label} failed\nstdout:\n{}\nstderr:\n{}",
        preview_bytes(&output.stdout),
        preview_bytes(&output.stderr)
    );
}

pub fn assert_stdout_eq(actual: &[u8], expected: &[u8], label: &str) {
    if actual == expected {
        return;
    }

    let first_diff = first_diff_index(actual, expected)
        .map(|index| index.to_string())
        .unwrap_or_else(|| "length-only".to_string());

    panic!(
        "{label} stdout mismatch\nactual_len: {}\nexpected_len: {}\nfirst_diff: {}\nactual_preview:\n{}\nexpected_preview:\n{}",
        actual.len(),
        expected.len(),
        first_diff,
        preview_bytes(actual),
        preview_bytes(expected),
    );
}

pub fn preview_bytes(bytes: &[u8]) -> String {
    const LIMIT: usize = 512;
    let mut preview = bytes
        .iter()
        .take(LIMIT)
        .flat_map(|byte| std::ascii::escape_default(*byte))
        .map(char::from)
        .collect::<String>();

    if bytes.len() > LIMIT {
        preview.push_str(&format!("\n... <{} bytes truncated>", bytes.len() - LIMIT));
    }

    preview
}

fn first_diff_index(actual: &[u8], expected: &[u8]) -> Option<usize> {
    actual
        .iter()
        .zip(expected)
        .position(|(actual, expected)| actual != expected)
}
