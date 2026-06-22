use std::fs;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::process::write_fixture_stdin;

const LINUX_CLK_TCK: f64 = 100.0;

#[derive(Debug, Clone, Default)]
pub struct ResourceMetrics {
    pub user_cpu_s: Option<f64>,
    pub system_cpu_s: Option<f64>,
    pub elapsed_wall_s: Option<f64>,
    pub max_rss_kb: Option<u64>,
    pub minor_page_faults: Option<u64>,
    pub major_page_faults: Option<u64>,
    pub voluntary_context_switches: Option<u64>,
    pub involuntary_context_switches: Option<u64>,
}

pub fn output_with_stdin_and_resources(
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
    drop(child_stdin);

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
