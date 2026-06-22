use std::env;
use std::process::Command;
use std::time::{Duration, Instant};

use super::process::assert_output_success;

pub fn benchmark_iterations(default_iterations: usize) -> usize {
    let iterations = match env::var("ECHO_BENCH_ITERATIONS") {
        Ok(value) => value.parse().unwrap_or_else(|err| {
            panic!("ECHO_BENCH_ITERATIONS must be a positive integer, got {value:?}: {err}")
        }),
        Err(env::VarError::NotPresent) => default_iterations,
        Err(err) => panic!("failed to read ECHO_BENCH_ITERATIONS: {err}"),
    };

    assert!(
        iterations > 0,
        "ECHO_BENCH_ITERATIONS must be greater than zero"
    );
    iterations
}

pub fn time_iterations(iterations: usize, mut f: impl FnMut()) -> Duration {
    let start = Instant::now();

    for _ in 0..iterations {
        f();
    }

    start.elapsed()
}

pub fn assert_tool_exists(tool: &str) {
    let output = Command::new("which")
        .arg(tool)
        .output()
        .unwrap_or_else(|err| panic!("failed to check for {tool}: {err}"));

    assert_output_success(&output, &format!("which {tool}"));
}
