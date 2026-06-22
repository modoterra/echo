use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_PATH_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    O0,
    O1,
    O2,
    O3,
    Oz,
}

impl OptimizationLevel {
    const fn clang_arg(self) -> &'static str {
        match self {
            Self::O0 => "-O0",
            Self::O1 => "-O1",
            Self::O2 => "-O2",
            Self::O3 => "-O3",
            Self::Oz => "-Oz",
        }
    }

    const fn opt_pipeline(self) -> Option<&'static str> {
        match self {
            Self::O0 => None,
            Self::O1 => Some("default<O1>"),
            Self::O2 => Some("default<O2>"),
            Self::O3 => Some("default<O3>"),
            Self::Oz => Some("default<Oz>"),
        }
    }
}

pub fn parse_optimization_level(value: &str) -> Result<OptimizationLevel, String> {
    match value {
        "0" => Ok(OptimizationLevel::O0),
        "1" => Ok(OptimizationLevel::O1),
        "2" => Ok(OptimizationLevel::O2),
        "3" => Ok(OptimizationLevel::O3),
        "z" | "Z" => Ok(OptimizationLevel::Oz),
        _ => Err(format!("expected one of 0, 1, 2, 3, or z, got `{value}`")),
    }
}

pub fn build_ir_binary(file: &Path, ir: &str, optimization: OptimizationLevel, output: &Path) {
    ensure_runtime_library();
    build_ir_binary_quiet(file, ir, optimization, output);
}

pub fn verify_ir(file: &Path, ir: &str) {
    let ir_path = write_temp_ir(file, ir);
    let output = ProcessCommand::new("opt")
        .arg("-disable-output")
        .arg("-passes=verify")
        .arg(&ir_path)
        .output()
        .unwrap_or_else(|err| {
            eprintln!("error: failed to run opt verifier: {err}");
            std::process::exit(1);
        });
    let _ = fs::remove_file(ir_path);

    if !output.status.success() {
        eprintln!("error: generated LLVM IR failed verification");
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(output.status.code().unwrap_or(1));
    }
}

pub fn optimize_ir(file: &Path, ir: &str, optimization: OptimizationLevel) -> String {
    let Some(pipeline) = optimization.opt_pipeline() else {
        return ir.to_string();
    };

    let ir_path = write_temp_ir(file, ir);
    let output = ProcessCommand::new("opt")
        .arg("-S")
        .arg(format!("-passes={pipeline}"))
        .arg(&ir_path)
        .output()
        .unwrap_or_else(|err| {
            eprintln!("error: failed to run opt: {err}");
            std::process::exit(1);
        });
    let _ = fs::remove_file(ir_path);

    if !output.status.success() {
        eprintln!("error: opt exited with {}", output.status);
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(output.status.code().unwrap_or(1));
    }

    String::from_utf8(output.stdout).unwrap_or_else(|err| {
        eprintln!("error: opt emitted non-UTF-8 IR: {err}");
        std::process::exit(1);
    })
}

pub fn temp_path(file: &Path, extension: &str) -> PathBuf {
    let stem = file
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("program");

    let counter = TEMP_PATH_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);

    std::env::temp_dir().join(format!(
        "echo-{stem}-{}-{nanos}-{counter}.{extension}",
        std::process::id(),
    ))
}

pub fn run_command(command: &mut ProcessCommand) {
    let status = command.status().unwrap_or_else(|err| {
        eprintln!("error: failed to run {command:?}: {err}");
        std::process::exit(1);
    });

    if !status.success() {
        eprintln!("error: {command:?} exited with {status}");
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn build_ir_binary_quiet(file: &Path, ir: &str, optimization: OptimizationLevel, output: &Path) {
    let ir_path = write_temp_ir(file, ir);
    run_command(
        ProcessCommand::new("clang")
            .arg(optimization.clang_arg())
            .arg("-x")
            .arg("ir")
            .arg(&ir_path)
            .arg("-x")
            .arg("none")
            .arg(runtime_library_path())
            .arg("-o")
            .arg(output),
    );
    let _ = fs::remove_file(ir_path);
}

fn write_temp_ir(file: &Path, ir: &str) -> PathBuf {
    create_temp_file(file, "ll", ir.as_bytes())
}

fn create_temp_file(file: &Path, extension: &str, contents: &[u8]) -> PathBuf {
    for _ in 0..100 {
        let path = temp_path(file, extension);
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut temp_file) => {
                temp_file.write_all(contents).unwrap_or_else(|err| {
                    eprintln!("error: failed to write {}: {err}", path.display());
                    std::process::exit(1);
                });
                return path;
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                eprintln!("error: failed to create {}: {err}", path.display());
                std::process::exit(1);
            }
        }
    }

    eprintln!("error: failed to allocate unique temporary {extension} path");
    std::process::exit(1);
}

fn runtime_library_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xo should be two levels below the workspace root")
        .join("target/debug/libecho_runtime.a")
}

fn ensure_runtime_library() {
    run_command(
        ProcessCommand::new("cargo")
            .arg("build")
            .arg("-p")
            .arg("echo_runtime"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp_ir_files_are_unique_for_repeated_builds() {
        let source = PathBuf::from("program.php");
        let first = create_temp_file(&source, "ll", b"first");
        let second = create_temp_file(&source, "ll", b"second");

        assert_ne!(first, second);
        assert_eq!(fs::read(&first).expect("first temp file"), b"first");
        assert_eq!(fs::read(&second).expect("second temp file"), b"second");

        let _ = fs::remove_file(first);
        let _ = fs::remove_file(second);
    }

    #[test]
    fn parses_optimization_levels() {
        assert_eq!(parse_optimization_level("0"), Ok(OptimizationLevel::O0));
        assert_eq!(parse_optimization_level("1"), Ok(OptimizationLevel::O1));
        assert_eq!(parse_optimization_level("2"), Ok(OptimizationLevel::O2));
        assert_eq!(parse_optimization_level("3"), Ok(OptimizationLevel::O3));
        assert_eq!(parse_optimization_level("z"), Ok(OptimizationLevel::Oz));
        assert!(parse_optimization_level("4").is_err());
    }
}
