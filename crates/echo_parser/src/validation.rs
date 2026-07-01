use echo_ast::{NamespaceSource, Program, Stmt};
use echo_diagnostics::Diagnostic;

pub(crate) fn validate_program(program: &Program) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    validate_inline_html(program, &mut diagnostics);
    validate_namespace_prelude(program, &mut diagnostics);
    validate_compile_prelude(program, &mut diagnostics);

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn validate_inline_html(program: &Program, diagnostics: &mut Vec<Diagnostic>) {
    let has_php_context = program.open_tag.is_some()
        || program
            .statements
            .iter()
            .any(|statement| !matches!(statement, Stmt::PhpInlineHtml(_)));

    for statement in &program.statements {
        let Stmt::PhpInlineHtml(statement) = statement else {
            continue;
        };

        if has_php_context || statement.text.trim_start().starts_with('<') {
            continue;
        }

        diagnostics.push(Diagnostic::new(
            "expected PHP code, Echo code, or inline HTML markup",
            statement.span,
        ));
    }
}

fn validate_namespace_prelude(program: &Program, diagnostics: &mut Vec<Diagnostic>) {
    let mut saw_non_module = false;

    for statement in &program.statements {
        match statement {
            Stmt::Namespace(statement)
                if statement.source == NamespaceSource::Echo && saw_non_module =>
            {
                diagnostics.push(Diagnostic::new(
                    "module and namespace declarations must appear before imports, compile declarations, and executable statements",
                    statement.span,
                ));
            }
            Stmt::Namespace(statement) if statement.source == NamespaceSource::Echo => {}
            _ => saw_non_module = true,
        }
    }
}

fn validate_compile_prelude(program: &Program, diagnostics: &mut Vec<Diagnostic>) {
    let mut saw_executable = false;

    for statement in &program.statements {
        match statement {
            Stmt::Namespace(_) | Stmt::Use(_) | Stmt::Import(_) | Stmt::Compile(_) => {
                if saw_executable && let Stmt::Compile(statement) = statement {
                    diagnostics.push(Diagnostic::new(
                        "`compile` declarations must appear before executable statements",
                        statement.span,
                    ));
                }
            }
            _ => saw_executable = true,
        }
    }
}
