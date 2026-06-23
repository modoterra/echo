use std::fs;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use clap::{Parser, Subcommand};

mod build;
mod repl;
mod source;
mod test_runner;
mod tools;

use build::{
    OptimizationLevel, build_ir_binary, optimize_ir, parse_optimization_level, run_command,
    temp_path, verify_ir,
};
use repl::run_repl;
use source::{
    ModeOverride, compile_ir, parse_source_program, print_diagnostics, read_source,
    read_source_file, run_jit,
};

#[derive(Debug, Parser)]
#[command(name = "xo")]
#[command(about = "Echo language toolchain")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Lsp,
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
        /// Execute with the in-process LLVM JIT instead of a temporary native binary.
        #[arg(long)]
        jit: bool,
        file: PathBuf,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Repl {
        #[command(flatten)]
        mode: ModeOverride,
    },
    Test {
        #[command(flatten)]
        mode: ModeOverride,
        path: PathBuf,
    },
    Tools {
        #[command(subcommand)]
        command: ToolsCommand,
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

#[derive(Debug, Subcommand)]
enum ToolsCommand {
    Grammar {
        #[command(subcommand)]
        command: GrammarCommand,
    },
}

#[derive(Debug, Subcommand)]
enum GrammarCommand {
    TreeSitter {
        /// Directory where the generated tree-sitter grammar package is written.
        #[arg(short, long)]
        output: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, clap::Args)]
struct OptimizationOptions {
    /// LLVM optimization level for build output.
    #[arg(short = 'O', default_value = "0", value_parser = parse_optimization_level)]
    level: OptimizationLevel,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Lsp => {
            let runtime = tokio::runtime::Runtime::new().unwrap_or_else(|err| {
                eprintln!("error: failed to initialize LSP runtime: {err}");
                std::process::exit(1);
            });
            runtime.block_on(async {
                if let Err(err) = echo_lsp::run_stdio().await {
                    eprintln!("error: Echo LSP server failed: {err}");
                    std::process::exit(1);
                }
            });
        }
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

            match parse_source_program(&source) {
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
        Command::Run {
            mode,
            jit,
            file,
            args,
        } => {
            if jit {
                if !args.is_empty() {
                    eprintln!("error: xo run --jit does not support program arguments yet");
                    std::process::exit(1);
                }
                run_jit(&file, mode);
            } else {
                let binary_path = temp_path(&file, "program");
                build_binary(&file, mode, OptimizationLevel::O0, &binary_path);
                run_command(ProcessCommand::new(&binary_path).args(args));
                let _ = fs::remove_file(binary_path);
            }
        }
        Command::Repl { mode } => {
            run_repl(mode);
        }
        Command::Test { mode, path } => {
            test_runner::run_tests(&path, mode);
        }
        Command::Tools { command } => match command {
            ToolsCommand::Grammar { command } => match command {
                GrammarCommand::TreeSitter { output } => {
                    if let Err(err) = tools::grammar::tree_sitter::generate(&output) {
                        eprintln!("error: failed to generate Tree-sitter grammar: {err}");
                        std::process::exit(1);
                    }
                }
            },
        },
        Command::Build {
            mode,
            optimization,
            file,
            output,
            emit_ir,
        } => {
            if emit_ir {
                let ir = compile_ir(&file, mode);
                verify_ir(&file, &ir);
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
    let ir = compile_ir(file, mode);
    verify_ir(file, &ir);
    build_ir_binary(file, &ir, optimization, output);
}
