use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use crate::build::{OptimizationLevel, build_ir_binary, temp_path, verify_ir};
use crate::source::{ModeOverride, compile_ir};

pub fn run_tests(path: &Path, mode: ModeOverride) {
    let tests = collect_test_files(path);
    if tests.is_empty() {
        eprintln!("error: no .echo tests found in {}", path.display());
        std::process::exit(1);
    }

    let mut failures = 0;
    for test in tests {
        let binary_path = temp_path(&test, "test");
        build_binary(&test, mode, &binary_path);
        let output = ProcessCommand::new(&binary_path)
            .output()
            .unwrap_or_else(|err| {
                eprintln!("error: failed to run {}: {err}", binary_path.display());
                std::process::exit(1);
            });
        let _ = fs::remove_file(&binary_path);

        print!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));

        if output.status.success() {
            println!("ok {}", test.display());
        } else {
            failures += 1;
            eprintln!("FAILED {}", test.display());
        }
    }

    if failures > 0 {
        eprintln!("{failures} test(s) failed");
        std::process::exit(1);
    }
}

fn build_binary(file: &Path, mode: ModeOverride, output: &Path) {
    let ir = compile_ir(file, mode);
    verify_ir(file, &ir);
    build_ir_binary(file, &ir, OptimizationLevel::O0, output);
}

fn collect_test_files(path: &Path) -> Vec<PathBuf> {
    let mut tests = Vec::new();
    collect_test_files_into(path, &mut tests);
    tests.sort();
    tests
}

fn collect_test_files_into(path: &Path, tests: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path.extension().and_then(|extension| extension.to_str()) == Some("echo") {
            tests.push(path.to_path_buf());
        }
        return;
    }

    let entries = fs::read_dir(path).unwrap_or_else(|err| {
        eprintln!("error: failed to read {}: {err}", path.display());
        std::process::exit(1);
    });

    for entry in entries {
        let path = entry
            .unwrap_or_else(|err| {
                eprintln!("error: failed to read test directory entry: {err}");
                std::process::exit(1);
            })
            .path();
        if path.is_dir() {
            collect_test_files_into(&path, tests);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("echo") {
            tests.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "xo-test-runner-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos()
        ))
    }

    #[test]
    fn collect_test_files_accepts_single_echo_file() {
        let root = unique_temp_dir();
        fs::create_dir_all(&root).expect("temp dir");
        let test = root.join("sample.echo");
        fs::write(&test, "echo \"ok\"").expect("test file");

        assert_eq!(collect_test_files(&test), vec![test.clone()]);

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn collect_test_files_recurses_and_sorts_echo_files() {
        let root = unique_temp_dir();
        let nested = root.join("nested");
        fs::create_dir_all(&nested).expect("nested temp dir");
        let first = root.join("a.echo");
        let second = nested.join("b.echo");
        fs::write(&second, "echo \"b\"").expect("second test file");
        fs::write(root.join("ignored.txt"), "ignored").expect("ignored file");
        fs::write(&first, "echo \"a\"").expect("first test file");

        assert_eq!(collect_test_files(&root), vec![first, second]);

        fs::remove_dir_all(root).expect("cleanup");
    }
}
