use echo_ast::Program;
use echo_diagnostics::Diagnostic;

pub(crate) fn validate_program(_program: &Program) -> Result<(), Vec<Diagnostic>> {
    Ok(())
}
