use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const ITERATIONS: usize = 100;

#[test]
#[ignore = "benchmark is opt-in and requires php and clang on PATH"]
fn benchmark_php_fixtures_against_php() {
    assert_tool_exists("php");

    let fixtures = fixture_dirs();
    assert!(!fixtures.is_empty(), "expected at least one PHP fixture");

    for fixture in fixtures {
        let program_path = fixture.join("program.php");
        let stdin_path = fixture.join("stdin.txt");
        let stdout_path = fixture.join("stdout.txt");
        let artifact_dir = artifact_dir_for(&fixture);

        fs::create_dir_all(&artifact_dir)
            .unwrap_or_else(|err| panic!("failed to create {}: {err}", artifact_dir.display()));

        let expected_stdout = fs::read(&stdout_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", stdout_path.display()));
        let stdin = fs::read(&stdin_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", stdin_path.display()));

        let echo_binary = build_echo_binary(&program_path, &artifact_dir);

        let php_first = output_with_stdin(Command::new("php").arg(&program_path), &stdin);
        assert_success(&php_first, "php");
        assert_eq!(php_first.stdout, expected_stdout, "php output mismatch");

        let echo_first = output_with_stdin(&mut Command::new(&echo_binary), &stdin);
        assert_success(&echo_first, "Echo binary");
        assert_eq!(echo_first.stdout, expected_stdout, "Echo output mismatch");

        let php_duration = time_iterations(|| {
            let output = output_with_stdin(Command::new("php").arg(&program_path), &stdin);
            assert_success(&output, "php");
        });

        let echo_duration = time_iterations(|| {
            let output = output_with_stdin(&mut Command::new(&echo_binary), &stdin);
            assert_success(&output, "Echo binary");
        });

        let fixture_name = fixture
            .file_name()
            .and_then(|name| name.to_str())
            .expect("fixture path should have UTF-8 file name");
        let report = format_report(fixture_name, php_duration, echo_duration, &echo_binary);

        print!("{report}");
        fs::write(artifact_dir.join("benchmark.txt"), report)
            .unwrap_or_else(|err| panic!("failed to write benchmark report: {err}"));
    }
}

fn time_iterations(mut f: impl FnMut()) -> Duration {
    let start = Instant::now();

    for _ in 0..ITERATIONS {
        f();
    }

    start.elapsed()
}

fn build_echo_binary(program_path: &Path, artifact_dir: &Path) -> PathBuf {
    let echo_binary = artifact_dir.join("benchmark-program");
    let mut build = Command::new(env!("CARGO_BIN_EXE_xo"));
    build
        .arg("build")
        .arg(program_path)
        .arg("-o")
        .arg(&echo_binary);
    assert_success(
        &build.output().expect("failed to build Echo binary"),
        "xo build",
    );

    echo_binary
}

fn format_report(
    fixture_name: &str,
    php_duration: Duration,
    echo_duration: Duration,
    echo_binary: &Path,
) -> String {
    let php_avg = php_duration.as_secs_f64() * 1_000_000.0 / ITERATIONS as f64;
    let echo_avg = echo_duration.as_secs_f64() * 1_000_000.0 / ITERATIONS as f64;
    let speedup = php_avg / echo_avg;
    let summary = if speedup >= 1.0 {
        format!("Echo is {:.2}x faster than PHP", speedup)
    } else {
        format!("Echo is {:.2}x slower than PHP", 1.0 / speedup)
    };

    format!(
        "{fixture_name}\n{summary}\niterations: {ITERATIONS}\necho_binary: {}\necho_build_timing: excluded; binary built once and reused\nphp_avg_us: {:.3}\necho_avg_us: {:.3}\necho_speedup_vs_php: {:.3}x\nphp_total_ms: {:.3}\necho_total_ms: {:.3}\n\n",
        echo_binary.display(),
        php_avg,
        echo_avg,
        speedup,
        php_duration.as_secs_f64() * 1_000.0,
        echo_duration.as_secs_f64() * 1_000.0,
    )
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

fn assert_success(output: &std::process::Output, label: &str) {
    assert!(
        output.status.success(),
        "{label} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_tool_exists(tool: &str) {
    let output = Command::new("which")
        .arg(tool)
        .output()
        .unwrap_or_else(|err| panic!("failed to check for {tool}: {err}"));

    assert_success(&output, &format!("which {tool}"));
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

fn artifact_dir_for(fixture: &Path) -> PathBuf {
    let name = fixture
        .file_name()
        .and_then(|name| name.to_str())
        .expect("fixture path should have UTF-8 file name");

    workspace_root().join("test-results/php").join(name)
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xo should be two levels below the workspace root")
        .to_path_buf()
}
