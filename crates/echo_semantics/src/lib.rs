mod analysis;
mod analyzer;
mod index_facts;
mod types;

pub use analysis::{Analysis, VariableInfo};
pub use index_facts::{index_facts, index_facts_from_source};
pub use types::Type;

use echo_ast::Program;
use echo_diagnostics::Diagnostic;

pub(crate) use analysis::SpanKey;

pub fn analyze(program: &Program) -> Result<Analysis, Vec<Diagnostic>> {
    analyzer::analyze_program(program)
}

#[cfg(test)]
mod tests;
