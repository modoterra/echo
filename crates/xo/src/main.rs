use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Parser, Subcommand};
use echo_source::{SourceFile, SourceMode};

static TEMP_PATH_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Parser)]
#[command(name = "xo")]
#[command(about = "Echo language toolchain")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Lex {
        file: PathBuf,
    },
    Ast {
        #[command(flatten)]
        mode: ModeOverride,
        file: PathBuf,
    },
    Ir {
        #[command(flatten)]
        mode: ModeOverride,
        file: PathBuf,
    },
    Run {
        #[command(flatten)]
        mode: ModeOverride,
        file: PathBuf,
    },
    Build {
        #[command(flatten)]
        mode: ModeOverride,
        #[command(flatten)]
        optimization: OptimizationOptions,
        file: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(long)]
        emit_ir: bool,
    },
}

#[derive(Debug, Clone, Copy, clap::Args)]
struct ModeOverride {
    /// Force strict mode, rejecting unsafe PHP compatibility patterns.
    #[arg(long, conflicts_with = "unsafe_mode")]
    strict: bool,

    /// Force Echo unsafe/superset mode, allowing PHP compatibility patterns.
    #[arg(long = "unsafe", conflicts_with = "strict")]
    unsafe_mode: bool,
}

impl ModeOverride {
    fn apply(self, default: SourceMode) -> SourceMode {
        if self.strict {
            SourceMode::Strict
        } else if self.unsafe_mode {
            SourceMode::Echo
        } else {
            default
        }
    }
}

#[derive(Debug, Clone, Copy, clap::Args)]
struct OptimizationOptions {
    /// LLVM optimization level for build output.
    #[arg(short = 'O', default_value = "0", value_parser = parse_optimization_level)]
    level: OptimizationLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OptimizationLevel {
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

fn parse_optimization_level(value: &str) -> Result<OptimizationLevel, String> {
    match value {
        "0" => Ok(OptimizationLevel::O0),
        "1" => Ok(OptimizationLevel::O1),
        "2" => Ok(OptimizationLevel::O2),
        "3" => Ok(OptimizationLevel::O3),
        "z" | "Z" => Ok(OptimizationLevel::Oz),
        _ => Err(format!("expected one of 0, 1, 2, 3, or z, got `{value}`")),
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Lex { file } => {
            let source = read_source(&file);

            match echo_lexer::lex(&source) {
                Ok(tokens) => {
                    for token in tokens {
                        println!("{token:#?}");
                    }
                }
                Err(diagnostics) => {
                    print_diagnostics(diagnostics);
                    std::process::exit(1);
                }
            }
        }
        Command::Ast { mode, file } => {
            let source = read_source_file(&file, mode);

            match echo_parser::parse_with_mode(&source.text, source.mode) {
                Ok(program) => {
                    println!("{program:#?}");
                }
                Err(diagnostics) => {
                    print_diagnostics(diagnostics);
                    std::process::exit(1);
                }
            }
        }
        Command::Ir { mode, file } => {
            let ir = compile_ir(&file, mode);
            print!("{ir}");
        }
        Command::Run { mode, file } => {
            let binary_path = temp_path(&file, "program");
            build_binary(&file, mode, OptimizationLevel::O0, &binary_path);
            run_command(&mut ProcessCommand::new(&binary_path));
            let _ = fs::remove_file(binary_path);
        }
        Command::Build {
            mode,
            optimization,
            file,
            output,
            emit_ir,
        } => {
            if emit_ir {
                let ir = compile_ir(&file, mode);
                print!("{}", optimize_ir(&file, &ir, optimization.level));
            } else {
                let Some(output) = output else {
                    eprintln!("error: xo build requires -o/--output unless --emit-ir is used");
                    std::process::exit(1);
                };

                build_binary(&file, mode, optimization.level, &output);
            }
        }
    }
}

fn build_binary(
    file: &PathBuf,
    mode: ModeOverride,
    optimization: OptimizationLevel,
    output: &PathBuf,
) {
    ensure_runtime_library();

    let ir = compile_ir(file, mode);
    let ir_path = write_temp_ir(file, &ir);
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

fn optimize_ir(file: &PathBuf, ir: &str, optimization: OptimizationLevel) -> String {
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

fn compile_ir(file: &PathBuf, mode: ModeOverride) -> String {
    let source = read_source_file(file, mode);

    let program = match echo_parser::parse_with_mode(&source.text, source.mode) {
        Ok(program) => program,
        Err(diagnostics) => {
            print_diagnostics(diagnostics);
            std::process::exit(1);
        }
    };

    match echo_codegen::compile_to_ir(&program) {
        Ok(ir) => ir,
        Err(diagnostics) => {
            print_diagnostics(diagnostics);
            std::process::exit(1);
        }
    }
}

fn write_temp_ir(file: &PathBuf, ir: &str) -> PathBuf {
    let path = create_temp_file(file, "ll", ir.as_bytes());

    path
}

fn create_temp_file(file: &PathBuf, extension: &str, contents: &[u8]) -> PathBuf {
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

fn temp_path(file: &PathBuf, extension: &str) -> PathBuf {
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

fn run_command(command: &mut ProcessCommand) {
    let status = command.status().unwrap_or_else(|err| {
        eprintln!("error: failed to run {command:?}: {err}");
        std::process::exit(1);
    });

    if !status.success() {
        eprintln!("error: {command:?} exited with {status}");
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn read_source(file: &PathBuf) -> String {
    fs::read_to_string(file).unwrap_or_else(|err| {
        eprintln!("error: failed to read {}: {err}", file.display());
        std::process::exit(1);
    })
}

fn read_source_file(file: &PathBuf, mode: ModeOverride) -> SourceFile {
    let mut source = SourceFile::new(file.clone(), read_source(file));
    source.mode = mode.apply(source.mode);
    source
}

fn print_diagnostics(diagnostics: Vec<echo_diagnostics::Diagnostic>) {
    for diagnostic in diagnostics {
        eprintln!(
            "error: {} at {}..{}",
            diagnostic.message, diagnostic.span.start, diagnostic.span.end
        );
    }
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
