use std::collections::VecDeque;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_ITERATIONS: usize = 100;
const LINUX_CLK_TCK: f64 = 100.0;

#[test]
#[ignore = "benchmark is opt-in and requires php and clang on PATH"]
fn benchmark_php_fixtures_against_php() {
    assert_tool_exists("php");

    let iterations = benchmark_iterations();
    let fixtures = fixture_dirs();
    assert!(!fixtures.is_empty(), "expected at least one PHP fixture");
    let jobs = benchmark_jobs(fixtures.len());
    let suite_start = Instant::now();
    let queue = Arc::new(Mutex::new(VecDeque::from(fixtures)));
    let rows = Arc::new(Mutex::new(Vec::new()));

    println!(
        "running {} PHP benchmark fixtures with {jobs} worker(s)",
        queue.lock().expect("benchmark queue lock poisoned").len()
    );

    thread::scope(|scope| {
        for _ in 0..jobs {
            let queue = Arc::clone(&queue);
            let rows = Arc::clone(&rows);

            scope.spawn(move || {
                loop {
                    let fixture = queue
                        .lock()
                        .expect("benchmark queue lock poisoned")
                        .pop_front();

                    let Some(fixture) = fixture else {
                        break;
                    };

                    let row = benchmark_fixture(&fixture, iterations);
                    rows.lock().expect("benchmark rows lock poisoned").push(row);
                }
            });
        }
    });

    let mut rows = Arc::try_unwrap(rows)
        .expect("benchmark worker still holds rows")
        .into_inner()
        .expect("benchmark rows lock poisoned");
    rows.sort_by(|left, right| left.fixture.cmp(&right.fixture));
    for row in &rows {
        print!("{}", row.format_report());
    }
    write_suite_artifacts(&rows, suite_start.elapsed());
}

fn benchmark_fixture(fixture: &Path, iterations: usize) -> BenchmarkRow {
    let program_path = fixture.join("program.php");
    let stdin_path = fixture.join("stdin.txt");
    let stdout_path = fixture.join("stdout.txt");
    let artifact_dir = artifact_dir_for(fixture);

    fs::create_dir_all(&artifact_dir)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", artifact_dir.display()));

    let expected_stdout = fs::read(&stdout_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", stdout_path.display()));
    let stdin = fs::read(&stdin_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", stdin_path.display()));

    let echo_binary = build_echo_binary(&program_path, &run_artifact_dir_for(fixture));

    let (php_first, php_resources) =
        output_with_stdin_and_resources(Command::new("php").arg(&program_path), &stdin);
    assert_success(&php_first, "php");
    assert_eq!(php_first.stdout, expected_stdout, "php output mismatch");

    let (echo_first, echo_resources) =
        output_with_stdin_and_resources(&mut Command::new(&echo_binary), &stdin);
    assert_success(&echo_first, "Echo binary");
    assert_eq!(echo_first.stdout, expected_stdout, "Echo output mismatch");

    let php_duration = time_iterations(iterations, || {
        let output = output_with_stdin(Command::new("php").arg(&program_path), &stdin);
        assert_success(&output, "php");
    });

    let echo_duration = time_iterations(iterations, || {
        let output = output_with_stdin(&mut Command::new(&echo_binary), &stdin);
        assert_success(&output, "Echo binary");
    });

    let fixture_name = fixture
        .file_name()
        .and_then(|name| name.to_str())
        .expect("fixture path should have UTF-8 file name");
    let row = BenchmarkRow::new(
        fixture_name,
        php_duration,
        echo_duration,
        php_resources,
        echo_resources,
        echo_binary.clone(),
        iterations,
    );
    let report = row.format_report();

    fs::write(artifact_dir.join("benchmark.txt"), report)
        .unwrap_or_else(|err| panic!("failed to write benchmark report: {err}"));
    row
}

#[derive(Debug)]
struct BenchmarkRow {
    fixture: String,
    iterations: usize,
    echo_binary: PathBuf,
    php_total: Duration,
    echo_total: Duration,
    php_resources: Option<ResourceMetrics>,
    echo_resources: Option<ResourceMetrics>,
}

impl BenchmarkRow {
    fn new(
        fixture: &str,
        php_total: Duration,
        echo_total: Duration,
        php_resources: Option<ResourceMetrics>,
        echo_resources: Option<ResourceMetrics>,
        echo_binary: PathBuf,
        iterations: usize,
    ) -> Self {
        Self {
            fixture: fixture.to_string(),
            iterations,
            echo_binary,
            php_total,
            echo_total,
            php_resources,
            echo_resources,
        }
    }

    fn php_avg_us(&self) -> f64 {
        self.php_total.as_secs_f64() * 1_000_000.0 / self.iterations as f64
    }

    fn echo_avg_us(&self) -> f64 {
        self.echo_total.as_secs_f64() * 1_000_000.0 / self.iterations as f64
    }

    fn speedup(&self) -> f64 {
        self.php_avg_us() / self.echo_avg_us()
    }

    fn format_report(&self) -> String {
        let summary = if self.speedup() >= 1.0 {
            format!("Echo is {:.2}x faster than PHP", self.speedup())
        } else {
            format!("Echo is {:.2}x slower than PHP", 1.0 / self.speedup())
        };

        format!(
            "{}\n{summary}\niterations: {}\necho_binary: {}\necho_build_timing: excluded; binary built once and reused\nphp_avg_us: {:.3}\necho_avg_us: {:.3}\necho_speedup_vs_php: {:.3}x\nphp_total_ms: {:.3}\necho_total_ms: {:.3}\nphp_max_rss_kb: {}\necho_max_rss_kb: {}\nphp_user_cpu_s: {}\necho_user_cpu_s: {}\nphp_system_cpu_s: {}\necho_system_cpu_s: {}\n\n",
            self.fixture,
            self.iterations,
            self.echo_binary.display(),
            self.php_avg_us(),
            self.echo_avg_us(),
            self.speedup(),
            self.php_total.as_secs_f64() * 1_000.0,
            self.echo_total.as_secs_f64() * 1_000.0,
            optional_u64(self.php_resources.as_ref().and_then(|m| m.max_rss_kb)),
            optional_u64(self.echo_resources.as_ref().and_then(|m| m.max_rss_kb)),
            optional_f64(self.php_resources.as_ref().and_then(|m| m.user_cpu_s)),
            optional_f64(self.echo_resources.as_ref().and_then(|m| m.user_cpu_s)),
            optional_f64(self.php_resources.as_ref().and_then(|m| m.system_cpu_s)),
            optional_f64(self.echo_resources.as_ref().and_then(|m| m.system_cpu_s)),
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

fn benchmark_jobs(fixture_count: usize) -> usize {
    let jobs = match env::var("ECHO_BENCH_JOBS") {
        Ok(value) => value.parse().unwrap_or_else(|err| {
            panic!("ECHO_BENCH_JOBS must be a positive integer, got {value:?}: {err}")
        }),
        Err(env::VarError::NotPresent) => thread::available_parallelism()
            .map(|parallelism| parallelism.get())
            .unwrap_or(1),
        Err(err) => panic!("failed to read ECHO_BENCH_JOBS: {err}"),
    };

    assert!(jobs > 0, "ECHO_BENCH_JOBS must be greater than zero");
    jobs.min(fixture_count.max(1))
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

    write_fixture_stdin(child.stdin.as_mut().expect("stdin should be piped"), stdin);

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

    let mut child_stdin = child.stdin.take().expect("stdin should be piped");
    write_fixture_stdin(&mut child_stdin, stdin);

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

fn write_fixture_stdin(writer: &mut impl Write, stdin: &[u8]) {
    match writer.write_all(stdin) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => {}
        Err(err) => panic!("failed to write fixture stdin: {err}"),
    }
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
    let root = workspace_root().join("test-results/php");
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
        "fixture,iterations,php_avg_us,echo_avg_us,speedup,php_total_ms,echo_total_ms,php_user_cpu_s,echo_user_cpu_s,php_system_cpu_s,echo_system_cpu_s,php_elapsed_wall_s,echo_elapsed_wall_s,php_max_rss_kb,echo_max_rss_kb,php_minor_page_faults,echo_minor_page_faults,php_major_page_faults,echo_major_page_faults,php_voluntary_context_switches,echo_voluntary_context_switches,php_involuntary_context_switches,echo_involuntary_context_switches\n",
    );

    for row in rows {
        csv.push_str(&format!(
            "{},{},{:.3},{:.3},{:.6},{:.3},{:.3},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            row.fixture,
            row.iterations,
            row.php_avg_us(),
            row.echo_avg_us(),
            row.speedup(),
            row.php_total.as_secs_f64() * 1_000.0,
            row.echo_total.as_secs_f64() * 1_000.0,
            optional_f64(row.php_resources.as_ref().and_then(|m| m.user_cpu_s)),
            optional_f64(row.echo_resources.as_ref().and_then(|m| m.user_cpu_s)),
            optional_f64(row.php_resources.as_ref().and_then(|m| m.system_cpu_s)),
            optional_f64(row.echo_resources.as_ref().and_then(|m| m.system_cpu_s)),
            optional_f64(row.php_resources.as_ref().and_then(|m| m.elapsed_wall_s)),
            optional_f64(row.echo_resources.as_ref().and_then(|m| m.elapsed_wall_s)),
            optional_u64(row.php_resources.as_ref().and_then(|m| m.max_rss_kb)),
            optional_u64(row.echo_resources.as_ref().and_then(|m| m.max_rss_kb)),
            optional_u64(row.php_resources.as_ref().and_then(|m| m.minor_page_faults)),
            optional_u64(
                row.echo_resources
                    .as_ref()
                    .and_then(|m| m.minor_page_faults)
            ),
            optional_u64(row.php_resources.as_ref().and_then(|m| m.major_page_faults)),
            optional_u64(
                row.echo_resources
                    .as_ref()
                    .and_then(|m| m.major_page_faults)
            ),
            optional_u64(
                row.php_resources
                    .as_ref()
                    .and_then(|m| m.voluntary_context_switches)
            ),
            optional_u64(
                row.echo_resources
                    .as_ref()
                    .and_then(|m| m.voluntary_context_switches)
            ),
            optional_u64(
                row.php_resources
                    .as_ref()
                    .and_then(|m| m.involuntary_context_switches)
            ),
            optional_u64(
                row.echo_resources
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
        "{{\n  \"iterations\": {iterations},\n  \"suite_total_ms\": {:.3},\n  \"rows\": [\n",
        suite_duration.as_secs_f64() * 1_000.0
    );

    for (index, row) in rows.iter().enumerate() {
        if index > 0 {
            json.push_str(",\n");
        }
        json.push_str(&format!(
            "    {{\"fixture\":\"{}\",\"php_avg_us\":{:.3},\"echo_avg_us\":{:.3},\"speedup\":{:.6},\"php_total_ms\":{:.3},\"echo_total_ms\":{:.3},\"php_max_rss_kb\":{},\"echo_max_rss_kb\":{},\"php_user_cpu_s\":{},\"echo_user_cpu_s\":{},\"php_system_cpu_s\":{},\"echo_system_cpu_s\":{}}}",
            json_escape(&row.fixture),
            row.php_avg_us(),
            row.echo_avg_us(),
            row.speedup(),
            row.php_total.as_secs_f64() * 1_000.0,
            row.echo_total.as_secs_f64() * 1_000.0,
            json_optional_u64(row.php_resources.as_ref().and_then(|m| m.max_rss_kb)),
            json_optional_u64(row.echo_resources.as_ref().and_then(|m| m.max_rss_kb)),
            json_optional_f64(row.php_resources.as_ref().and_then(|m| m.user_cpu_s)),
            json_optional_f64(row.echo_resources.as_ref().and_then(|m| m.user_cpu_s)),
            json_optional_f64(row.php_resources.as_ref().and_then(|m| m.system_cpu_s)),
            json_optional_f64(row.echo_resources.as_ref().and_then(|m| m.system_cpu_s)),
        ));
    }

    json.push_str("\n  ]\n}\n");
    json
}

fn suite_markdown(rows: &[BenchmarkRow], suite_duration: Duration) -> String {
    let iterations = suite_iterations(rows);
    let php_total_ms = rows
        .iter()
        .map(|row| row.php_total.as_secs_f64() * 1_000.0)
        .sum::<f64>();
    let echo_total_ms = rows
        .iter()
        .map(|row| row.echo_total.as_secs_f64() * 1_000.0)
        .sum::<f64>();

    let mut markdown = format!(
        "# PHP Benchmark Summary\n\n- fixtures: {}\n- iterations per fixture: {iterations}\n- suite wall time ms: {:.3}\n- php measured total ms: {:.3}\n- echo measured total ms: {:.3}\n- aggregate speedup: {:.3}x\n\n| Fixture | PHP avg us | Echo avg us | Speedup | PHP RSS KB | Echo RSS KB |\n| --- | ---: | ---: | ---: | ---: | ---: |\n",
        rows.len(),
        suite_duration.as_secs_f64() * 1_000.0,
        php_total_ms,
        echo_total_ms,
        php_total_ms / echo_total_ms,
    );

    for row in rows {
        markdown.push_str(&format!(
            "| {} | {:.3} | {:.3} | {:.3}x | {} | {} |\n",
            row.fixture,
            row.php_avg_us(),
            row.echo_avg_us(),
            row.speedup(),
            optional_u64(row.php_resources.as_ref().and_then(|m| m.max_rss_kb)),
            optional_u64(row.echo_resources.as_ref().and_then(|m| m.max_rss_kb)),
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

fn run_artifact_dir_for(fixture: &Path) -> PathBuf {
    let name = fixture
        .file_name()
        .and_then(|name| name.to_str())
        .expect("fixture path should have UTF-8 file name");

    workspace_root()
        .join("test-results/php/.runs")
        .join(std::process::id().to_string())
        .join(name)
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xo should be two levels below the workspace root")
        .to_path_buf()
}
