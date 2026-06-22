use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

mod support;

use support::{
    ResourceMetrics, RunArtifacts, artifact_dir_for_label, assert_output_success, assert_stdout_eq,
    assert_tool_exists, benchmark_iterations, fixture_dirs, output_with_stdin,
    output_with_stdin_and_resources, run_artifact_dir_for_label, time_iterations, workspace_root,
};

const DEFAULT_ITERATIONS: usize = 100;

#[test]
#[ignore = "benchmark is opt-in and requires clang on PATH"]
fn benchmark_echo_fixtures() {
    assert_tool_exists("clang");

    let _run_artifacts = RunArtifacts::new("echo");
    let iterations = benchmark_iterations(DEFAULT_ITERATIONS);
    let fixtures = fixture_cases();
    assert!(!fixtures.is_empty(), "expected at least one fixture");
    let suite_start = Instant::now();
    let mut rows = Vec::new();

    for fixture in fixtures {
        if fixture.path.join("unsupported.txt").is_file() {
            continue;
        }

        let program_path = fixture.path.join(fixture.program_file);
        let stdin_path = fixture.path.join("stdin.txt");
        let stdout_path = fixture.path.join("stdout.txt");
        let artifact_dir = artifact_dir_for_label("echo", &fixture.label());

        fs::create_dir_all(&artifact_dir)
            .unwrap_or_else(|err| panic!("failed to create {}: {err}", artifact_dir.display()));

        let expected_stdout = fs::read(&stdout_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", stdout_path.display()));
        let stdin = fs::read(&stdin_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", stdin_path.display()));

        let echo_binary = build_echo_binary(
            &program_path,
            &run_artifact_dir_for_label("echo", &fixture.label()),
        );

        let (binary_first, binary_resources) =
            output_with_stdin_and_resources(&mut Command::new(&echo_binary), &stdin);
        assert_success(&binary_first, "Echo binary");
        assert_stdout_eq(&binary_first.stdout, &expected_stdout, "Echo binary");

        let binary_duration = time_iterations(iterations, || {
            let output = output_with_stdin(&mut Command::new(&echo_binary), &stdin);
            assert_success(&output, "Echo binary");
        });

        let row = BenchmarkRow::new(
            &fixture.label(),
            binary_duration,
            binary_resources,
            echo_binary.clone(),
            iterations,
        );
        let report = row.format_report();

        print!("{report}");
        fs::write(artifact_dir.join("benchmark.txt"), report)
            .unwrap_or_else(|err| panic!("failed to write benchmark report: {err}"));
        rows.push(row);
    }

    assert!(!rows.is_empty(), "expected at least one executable fixture");
    write_suite_artifacts(&rows, suite_start.elapsed());
}

#[derive(Debug)]
struct FixtureCase {
    suite: &'static str,
    path: PathBuf,
    program_file: &'static str,
}

impl FixtureCase {
    fn label(&self) -> String {
        let name = self
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("fixture path should have UTF-8 file name");

        format!("{}/{name}", self.suite)
    }
}

#[derive(Debug)]
struct BenchmarkRow {
    fixture: String,
    iterations: usize,
    echo_binary: PathBuf,
    binary_total: Duration,
    binary_resources: Option<ResourceMetrics>,
}

impl BenchmarkRow {
    fn new(
        fixture: &str,
        binary_total: Duration,
        binary_resources: Option<ResourceMetrics>,
        echo_binary: PathBuf,
        iterations: usize,
    ) -> Self {
        Self {
            fixture: fixture.to_string(),
            iterations,
            echo_binary,
            binary_total,
            binary_resources,
        }
    }

    fn binary_avg_us(&self) -> f64 {
        self.binary_total.as_secs_f64() * 1_000_000.0 / self.iterations as f64
    }

    fn format_report(&self) -> String {
        format!(
            "{}\nEcho binary benchmark\niterations: {}\necho_binary: {}\necho_build_timing: excluded; binary built once and reused\necho_binary_avg_us: {:.3}\necho_binary_total_ms: {:.3}\necho_binary_max_rss_kb: {}\necho_binary_user_cpu_s: {}\necho_binary_system_cpu_s: {}\n\n",
            self.fixture,
            self.iterations,
            self.echo_binary.display(),
            self.binary_avg_us(),
            self.binary_total.as_secs_f64() * 1_000.0,
            optional_u64(self.binary_resources.as_ref().and_then(|m| m.max_rss_kb)),
            optional_f64(self.binary_resources.as_ref().and_then(|m| m.user_cpu_s)),
            optional_f64(self.binary_resources.as_ref().and_then(|m| m.system_cpu_s)),
        )
    }
}

fn build_echo_binary(program_path: &Path, artifact_dir: &Path) -> PathBuf {
    fs::create_dir_all(artifact_dir)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", artifact_dir.display()));
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

fn write_suite_artifacts(rows: &[BenchmarkRow], suite_duration: Duration) {
    let root = workspace_root().join("test-results/echo");
    fs::create_dir_all(root.join("graphs"))
        .unwrap_or_else(|err| panic!("failed to create benchmark graph directory: {err}"));
    let suite_json = suite_json(rows, suite_duration);

    fs::write(root.join("benchmark-suite.csv"), suite_csv(rows))
        .expect("failed to write benchmark suite csv");
    fs::write(root.join("benchmark-suite.json"), &suite_json)
        .expect("failed to write benchmark suite json");
    fs::write(
        root.join("benchmark-summary.md"),
        suite_markdown(rows, suite_duration),
    )
    .expect("failed to write benchmark suite markdown");
    fs::write(root.join("benchmark.html"), benchmark_html(&suite_json))
        .expect("failed to write benchmark html viewer");
}

fn suite_csv(rows: &[BenchmarkRow]) -> String {
    let mut csv = String::from(
        "fixture,iterations,echo_binary_avg_us,echo_binary_total_ms,echo_binary_user_cpu_s,echo_binary_system_cpu_s,echo_binary_elapsed_wall_s,echo_binary_max_rss_kb,echo_binary_minor_page_faults,echo_binary_major_page_faults,echo_binary_voluntary_context_switches,echo_binary_involuntary_context_switches\n",
    );

    for row in rows {
        csv.push_str(&format!(
            "{},{},{:.3},{:.3},{},{},{},{},{},{},{},{}\n",
            row.fixture,
            row.iterations,
            row.binary_avg_us(),
            row.binary_total.as_secs_f64() * 1_000.0,
            optional_f64(row.binary_resources.as_ref().and_then(|m| m.user_cpu_s)),
            optional_f64(row.binary_resources.as_ref().and_then(|m| m.system_cpu_s)),
            optional_f64(row.binary_resources.as_ref().and_then(|m| m.elapsed_wall_s)),
            optional_u64(row.binary_resources.as_ref().and_then(|m| m.max_rss_kb)),
            optional_u64(
                row.binary_resources
                    .as_ref()
                    .and_then(|m| m.minor_page_faults)
            ),
            optional_u64(
                row.binary_resources
                    .as_ref()
                    .and_then(|m| m.major_page_faults)
            ),
            optional_u64(
                row.binary_resources
                    .as_ref()
                    .and_then(|m| m.voluntary_context_switches)
            ),
            optional_u64(
                row.binary_resources
                    .as_ref()
                    .and_then(|m| m.involuntary_context_switches)
            ),
        ));
    }

    csv
}

fn suite_json(rows: &[BenchmarkRow], suite_duration: Duration) -> String {
    let iterations = suite_iterations(rows);
    let mut json = format!(
        "{{\n  \"title\": \"Echo Binary Fixture Benchmark Results\",\n  \"results_path\": \"test-results/echo/benchmark.html\",\n  \"benchmark_kind\": \"single\",\n  \"candidate_label\": \"Echo binary\",\n  \"iterations\": {iterations},\n  \"suite_total_ms\": {:.3},\n  \"rows\": [\n",
        suite_duration.as_secs_f64() * 1_000.0
    );

    for (index, row) in rows.iter().enumerate() {
        if index > 0 {
            json.push_str(",\n");
        }
        json.push_str(&format!(
            "    {{\"fixture\":\"{}\",\"echo_avg_us\":{:.3},\"echo_total_ms\":{:.3},\"echo_max_rss_kb\":{},\"echo_user_cpu_s\":{},\"echo_system_cpu_s\":{}}}",
            json_escape(&row.fixture),
            row.binary_avg_us(),
            row.binary_total.as_secs_f64() * 1_000.0,
            json_optional_u64(row.binary_resources.as_ref().and_then(|m| m.max_rss_kb)),
            json_optional_f64(row.binary_resources.as_ref().and_then(|m| m.user_cpu_s)),
            json_optional_f64(row.binary_resources.as_ref().and_then(|m| m.system_cpu_s)),
        ));
    }

    json.push_str("\n  ]\n}\n");
    json
}

fn suite_markdown(rows: &[BenchmarkRow], suite_duration: Duration) -> String {
    let iterations = suite_iterations(rows);
    let binary_total_ms = rows
        .iter()
        .map(|row| row.binary_total.as_secs_f64() * 1_000.0)
        .sum::<f64>();

    let mut markdown = format!(
        "# Echo Binary Benchmark Summary\n\n- fixtures: {}\n- iterations per fixture: {iterations}\n- suite wall time ms: {:.3}\n- echo binary measured total ms: {:.3}\n\n| Fixture | Echo binary avg us | Echo binary RSS KB |\n| --- | ---: | ---: |\n",
        rows.len(),
        suite_duration.as_secs_f64() * 1_000.0,
        binary_total_ms,
    );

    for row in rows {
        markdown.push_str(&format!(
            "| {} | {:.3} | {} |\n",
            row.fixture,
            row.binary_avg_us(),
            optional_u64(row.binary_resources.as_ref().and_then(|m| m.max_rss_kb)),
        ));
    }

    markdown
}

fn suite_iterations(rows: &[BenchmarkRow]) -> usize {
    rows.first()
        .map(|row| row.iterations)
        .unwrap_or(DEFAULT_ITERATIONS)
}

fn benchmark_html(suite_json: &str) -> String {
    include_str!("../../../docs/benchmarks/benchmark.html")
        .replace("__BENCHMARK_DATA_JSON__", &html_escape(suite_json))
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn optional_f64(value: Option<f64>) -> String {
    value.map(|value| format!("{value:.3}")).unwrap_or_default()
}

fn optional_u64(value: Option<u64>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

fn json_optional_f64(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.3}"))
        .unwrap_or_else(|| "null".to_string())
}

fn json_optional_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string())
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn assert_success(output: &std::process::Output, label: &str) {
    assert_output_success(output, label);
}

fn fixture_cases() -> Vec<FixtureCase> {
    let mut fixtures = fixture_dirs("tests/php")
        .into_iter()
        .map(|path| FixtureCase {
            suite: "php",
            path,
            program_file: "program.php",
        })
        .collect::<Vec<_>>();

    fixtures.extend(
        fixture_dirs("tests/echo")
            .into_iter()
            .map(|path| FixtureCase {
                suite: "echo",
                path,
                program_file: "program.echo",
            }),
    );

    fixtures
}
