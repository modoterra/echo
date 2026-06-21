use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_ITERATIONS: usize = 100;
const LINUX_CLK_TCK: f64 = 100.0;

#[test]
#[ignore = "benchmark is opt-in and requires clang on PATH"]
fn benchmark_echo_fixtures() {
    assert_tool_exists("clang");

    let _run_artifacts = RunArtifacts::new(workspace_root().join("test-results/echo/.runs"));
    let iterations = benchmark_iterations();
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
        let artifact_dir = artifact_dir_for(&fixture);

        fs::create_dir_all(&artifact_dir)
            .unwrap_or_else(|err| panic!("failed to create {}: {err}", artifact_dir.display()));

        let expected_stdout = fs::read(&stdout_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", stdout_path.display()));
        let stdin = fs::read(&stdin_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", stdin_path.display()));

        let echo_binary = build_echo_binary(&program_path, &run_artifact_dir_for(&fixture));

        let (binary_first, binary_resources) =
            output_with_stdin_and_resources(&mut Command::new(&echo_binary), &stdin);
        assert_success(&binary_first, "Echo binary");
        assert_eq!(
            binary_first.stdout, expected_stdout,
            "Echo binary output mismatch"
        );

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

#[derive(Debug, Clone, Default)]
struct ResourceMetrics {
    user_cpu_s: Option<f64>,
    system_cpu_s: Option<f64>,
    elapsed_wall_s: Option<f64>,
    max_rss_kb: Option<u64>,
    minor_page_faults: Option<u64>,
    major_page_faults: Option<u64>,
    voluntary_context_switches: Option<u64>,
    involuntary_context_switches: Option<u64>,
}

fn benchmark_iterations() -> usize {
    let iterations = match env::var("ECHO_BENCH_ITERATIONS") {
        Ok(value) => value.parse().unwrap_or_else(|err| {
            panic!("ECHO_BENCH_ITERATIONS must be a positive integer, got {value:?}: {err}")
        }),
        Err(env::VarError::NotPresent) => DEFAULT_ITERATIONS,
        Err(err) => panic!("failed to read ECHO_BENCH_ITERATIONS: {err}"),
    };

    assert!(
        iterations > 0,
        "ECHO_BENCH_ITERATIONS must be greater than zero"
    );
    iterations
}

fn time_iterations(iterations: usize, mut f: impl FnMut()) -> Duration {
    let start = Instant::now();

    for _ in 0..iterations {
        f();
    }

    start.elapsed()
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

fn output_with_stdin_and_resources(
    command: &mut Command,
    stdin: &[u8],
) -> (std::process::Output, Option<ResourceMetrics>) {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|err| panic!("failed to run {command:?}: {err}"));

    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(stdin)
        .expect("failed to write fixture stdin");

    let pid = child.id();
    let mut metrics = ResourceMetrics::default();
    let start = Instant::now();

    loop {
        merge_proc_metrics(&mut metrics, pid);
        match child.try_wait().expect("failed to poll command") {
            Some(_) => break,
            None => thread::sleep(Duration::from_millis(1)),
        }
    }

    merge_proc_metrics(&mut metrics, pid);
    metrics.elapsed_wall_s = Some(start.elapsed().as_secs_f64());
    let output = child
        .wait_with_output()
        .expect("failed to wait for command");

    (output, Some(metrics))
}

fn merge_proc_metrics(metrics: &mut ResourceMetrics, pid: u32) {
    if let Ok(status) = fs::read_to_string(format!("/proc/{pid}/status")) {
        for line in status.lines() {
            if let Some(value) = line.strip_prefix("VmHWM:") {
                metrics.max_rss_kb = max_option(metrics.max_rss_kb, parse_status_kb(value));
            } else if let Some(value) = line.strip_prefix("VmRSS:") {
                metrics.max_rss_kb = max_option(metrics.max_rss_kb, parse_status_kb(value));
            } else if let Some(value) = line.strip_prefix("voluntary_ctxt_switches:") {
                metrics.voluntary_context_switches = parse_status_u64(value);
            } else if let Some(value) = line.strip_prefix("nonvoluntary_ctxt_switches:") {
                metrics.involuntary_context_switches = parse_status_u64(value);
            }
        }
    }

    if let Ok(stat) = fs::read_to_string(format!("/proc/{pid}/stat")) {
        if let Some(after_comm) = stat.rsplit_once(") ").map(|(_, after)| after) {
            let fields = after_comm.split_whitespace().collect::<Vec<_>>();
            metrics.minor_page_faults = fields.get(7).and_then(|value| value.parse().ok());
            metrics.major_page_faults = fields.get(9).and_then(|value| value.parse().ok());
            metrics.user_cpu_s = fields
                .get(11)
                .and_then(|value| value.parse::<f64>().ok())
                .map(|ticks| ticks / LINUX_CLK_TCK);
            metrics.system_cpu_s = fields
                .get(12)
                .and_then(|value| value.parse::<f64>().ok())
                .map(|ticks| ticks / LINUX_CLK_TCK);
        }
    }
}

fn parse_status_kb(value: &str) -> Option<u64> {
    value.split_whitespace().next()?.parse().ok()
}

fn parse_status_u64(value: &str) -> Option<u64> {
    value.trim().parse().ok()
}

fn max_option(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
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

fn artifact_dir_for(fixture: &FixtureCase) -> PathBuf {
    workspace_root()
        .join("test-results/echo")
        .join(fixture.label())
}

fn run_artifact_dir_for(fixture: &FixtureCase) -> PathBuf {
    workspace_root()
        .join("test-results/echo/.runs")
        .join(std::process::id().to_string())
        .join(fixture.label())
}

struct RunArtifacts {
    current_run_dir: PathBuf,
}

impl RunArtifacts {
    fn new(root: PathBuf) -> Self {
        fs::create_dir_all(&root)
            .unwrap_or_else(|err| panic!("failed to create {}: {err}", root.display()));
        remove_stale_run_dirs(&root);
        let current_run_dir = root.join(std::process::id().to_string());
        fs::create_dir_all(&current_run_dir)
            .unwrap_or_else(|err| panic!("failed to create {}: {err}", current_run_dir.display()));

        Self { current_run_dir }
    }
}

impl Drop for RunArtifacts {
    fn drop(&mut self) {
        if self.current_run_dir.exists() {
            fs::remove_dir_all(&self.current_run_dir).unwrap_or_else(|err| {
                panic!("failed to remove {}: {err}", self.current_run_dir.display())
            });
        }
    }
}

fn remove_stale_run_dirs(root: &Path) {
    let current_pid = std::process::id().to_string();
    for entry in
        fs::read_dir(root).unwrap_or_else(|err| panic!("failed to read {}: {err}", root.display()))
    {
        let path = entry.expect("failed to read run artifact entry").path();
        let is_current = path.file_name().and_then(|name| name.to_str()) == Some(&current_pid);
        if path.is_dir() && !is_current {
            fs::remove_dir_all(&path)
                .unwrap_or_else(|err| panic!("failed to remove {}: {err}", path.display()));
        }
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xo should be two levels below the workspace root")
        .to_path_buf()
}
