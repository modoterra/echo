use std::fs;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use clap::{Parser, Subcommand};

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
        file: PathBuf,
    },
    Ir {
        file: PathBuf,
    },
    Run {
        file: PathBuf,
    },
    Build {
        file: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
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
        Command::Ast { file } => {
            let source = read_source(&file);

            match echo_parser::parse(&source) {
                Ok(program) => {
                    println!("{program:#?}");
                }
                Err(diagnostics) => {
                    print_diagnostics(diagnostics);
                    std::process::exit(1);
                }
            }
        }
        Command::Ir { file } => {
            let ir = compile_ir(&file);
            print!("{ir}");
        }
        Command::Run { file } => {
            let binary_path = temp_path(&file, "program");
            build_binary(&file, &binary_path);
            run_command(&mut ProcessCommand::new(&binary_path));
            let _ = fs::remove_file(binary_path);
        }
        Command::Build { file, output } => {
            build_binary(&file, &output);
        }
    }
}

fn build_binary(file: &PathBuf, output: &PathBuf) {
    ensure_runtime_library();

    let ir = compile_ir(file);
    let ir_path = write_temp_ir(file, &ir);
    run_command(
        ProcessCommand::new("clang")
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

fn compile_ir(file: &PathBuf) -> String {
    let source = read_source(file);

    let program = match echo_parser::parse(&source) {
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
    let path = temp_path(file, "ll");

    fs::write(&path, ir).unwrap_or_else(|err| {
        eprintln!("error: failed to write {}: {err}", path.display());
        std::process::exit(1);
    });

    path
}

fn temp_path(file: &PathBuf, extension: &str) -> PathBuf {
    let stem = file
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("program");

    std::env::temp_dir().join(format!("echo-{stem}-{}.{}", std::process::id(), extension))
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

fn print_diagnostics(diagnostics: Vec<echo_diagnostics::Diagnostic>) {
    for diagnostic in diagnostics {
        eprintln!(
            "error: {} at {}..{}",
            diagnostic.message, diagnostic.span.start, diagnostic.span.end
        );
    }
}
