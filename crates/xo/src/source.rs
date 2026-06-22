use std::fs;
use std::path::{Path, PathBuf};

use echo_ast::Program;
use echo_source::{SourceFile, SourceMode};

#[derive(Debug, Clone, Copy, clap::Args)]
pub struct ModeOverride {
    /// Force strict mode, rejecting unsafe PHP compatibility patterns.
    #[arg(long, conflicts_with = "unsafe_mode")]
    pub strict: bool,

    /// Force Echo unsafe/superset mode, allowing PHP compatibility patterns.
    #[arg(long = "unsafe", conflicts_with = "strict")]
    pub unsafe_mode: bool,
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

pub fn read_source(file: &Path) -> String {
    fs::read_to_string(file).unwrap_or_else(|err| {
        eprintln!("error: failed to read {}: {err}", file.display());
        std::process::exit(1);
    })
}

pub fn read_source_file(file: &Path, mode: ModeOverride) -> SourceFile {
    source_file_from_text(file.to_path_buf(), read_source(file), mode)
}

pub fn source_file_from_text(file: PathBuf, text: String, mode: ModeOverride) -> SourceFile {
    let mut source = SourceFile::new(file, text);
    source.mode = mode.apply(source.mode);
    source
}

pub fn compile_ir(file: &Path, mode: ModeOverride) -> String {
    let source = read_source_file(file, mode);

    match try_compile_ir(&source) {
        Ok(ir) => ir,
        Err(diagnostics) => {
            print_diagnostics(diagnostics);
            std::process::exit(1);
        }
    }
}

pub fn run_jit(file: &Path, mode: ModeOverride) {
    let source = read_source_file(file, mode);

    match try_run_jit(&source) {
        Ok(status) => {
            if status != 0 {
                std::process::exit(status);
            }
        }
        Err(diagnostics) => {
            print_diagnostics(diagnostics);
            std::process::exit(1);
        }
    }
}

pub fn try_run_jit(source: &SourceFile) -> Result<i32, Vec<echo_diagnostics::Diagnostic>> {
    let program = parse_source_program(source)?;

    echo_codegen::run_program_jit(&program)
}

pub fn try_compile_ir(source: &SourceFile) -> Result<String, Vec<echo_diagnostics::Diagnostic>> {
    let program = parse_source_program(source)?;

    echo_codegen::compile_to_ir(&program)
}

pub fn parse_source_program(
    source: &SourceFile,
) -> Result<Program, Vec<echo_diagnostics::Diagnostic>> {
    let mut program = echo_parser::parse_with_mode(&source.text, source.mode)?;
    program.source_dir = source_dir_for(&source.path);
    Ok(program)
}

pub fn print_diagnostics(diagnostics: Vec<echo_diagnostics::Diagnostic>) {
    for diagnostic in diagnostics {
        eprintln!(
            "error: {} at {}..{}",
            diagnostic.message, diagnostic.span.start, diagnostic.span.end
        );
    }
}

fn source_dir_for(path: &Path) -> Option<String> {
    let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    path.parent().map(|path| path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repl_source_defaults_to_strict_mode() {
        let source = source_file_from_text(
            PathBuf::from("repl.echo"),
            "echo \"hello\";".to_string(),
            ModeOverride {
                strict: false,
                unsafe_mode: false,
            },
        );

        assert_eq!(source.mode, SourceMode::Strict);
    }

    #[test]
    fn repl_source_can_use_unsafe_mode() {
        let source = source_file_from_text(
            PathBuf::from("repl.echo"),
            "<?php echo \"hello\";".to_string(),
            ModeOverride {
                strict: false,
                unsafe_mode: true,
            },
        );

        assert_eq!(source.mode, SourceMode::Echo);
    }
}
