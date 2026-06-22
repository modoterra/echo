use echo_ast::{Expr, NamespaceSource, Program, Stmt};
use echo_diagnostics::Diagnostic;
use echo_source::SourceMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValidationMode {
    Echo,
    Strict,
    TrustedStd,
}

impl ValidationMode {
    pub(crate) const fn from_source_mode(mode: SourceMode) -> Self {
        match mode {
            SourceMode::Echo => Self::Echo,
            SourceMode::Strict => Self::Strict,
        }
    }

    const fn validates_strict(self) -> bool {
        matches!(self, Self::Strict | Self::TrustedStd)
    }

    const fn allows_std_namespace(self) -> bool {
        matches!(self, Self::TrustedStd)
    }
}

pub(crate) fn validate_mode(
    program: &Program,
    mode: ValidationMode,
) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    if mode.validates_strict() {
        for statement in &program.statements {
            validate_statement_mode(statement, mode, &mut diagnostics);
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn validate_statement_mode(
    statement: &Stmt,
    mode: ValidationMode,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match statement {
        Stmt::Echo(statement) => {
            for expr in &statement.exprs {
                validate_expr_mode(expr, mode, diagnostics);
            }
        }
        Stmt::FunctionCall(statement) => {
            for expr in &statement.args {
                validate_expr_mode(expr, mode, diagnostics);
            }
        }
        Stmt::DynamicFunctionCall(statement) => {
            diagnostics.push(Diagnostic::new(
                "dynamic function calls are not allowed in strict mode",
                statement.span,
            ));
            for expr in &statement.args {
                validate_expr_mode(expr, mode, diagnostics);
            }
        }
        Stmt::FunctionDecl(statement) => {
            for statement in &statement.body {
                validate_statement_mode(statement, mode, diagnostics);
            }
        }
        Stmt::Assign(statement) => validate_expr_mode(&statement.value, mode, diagnostics),
        Stmt::Let(statement) => validate_expr_mode(&statement.value, mode, diagnostics),
        Stmt::AssignRef(statement) => diagnostics.push(Diagnostic::new(
            "PHP references are not allowed in strict mode",
            statement.span,
        )),
        Stmt::Return(statement) => validate_expr_mode(&statement.value, mode, diagnostics),
        Stmt::Yield(statement) => validate_expr_mode(&statement.value, mode, diagnostics),
        Stmt::Expr(statement) => validate_expr_mode(&statement.expr, mode, diagnostics),
        Stmt::Loop(statement) => {
            for statement in &statement.body {
                validate_statement_mode(statement, mode, diagnostics);
            }
        }
        Stmt::If(statement) => {
            validate_expr_mode(&statement.condition, mode, diagnostics);
            for statement in &statement.body {
                validate_statement_mode(statement, mode, diagnostics);
            }
        }
        Stmt::Break(statement) => {
            if let Some(value) = &statement.value {
                validate_expr_mode(value, mode, diagnostics);
            }
        }
        Stmt::Append(statement) => validate_expr_mode(&statement.value, mode, diagnostics),
        Stmt::Namespace(statement) => {
            if statement.source == NamespaceSource::Std && !mode.allows_std_namespace() {
                diagnostics.push(Diagnostic::new(
                    "std namespace declarations are only allowed in trusted stdlib source",
                    statement.span,
                ));
            }
        }
        Stmt::Use(_) | Stmt::Import(_) | Stmt::ClassDecl(_) | Stmt::TypeDecl(_) => {}
    }
}

fn validate_expr_mode(expr: &Expr, mode: ValidationMode, diagnostics: &mut Vec<Diagnostic>) {
    match expr {
        Expr::Defer(_) | Expr::Run(_) | Expr::Fork(_) | Expr::Spawn(_) | Expr::Join(_) => {}
        Expr::Loop(expr) => {
            for statement in &expr.body {
                validate_statement_mode(statement, mode, diagnostics);
            }
        }
        Expr::FunctionCall(expr) => {
            for expr in &expr.args {
                validate_expr_mode(expr, mode, diagnostics);
            }
        }
        Expr::MethodCall(expr) => {
            validate_expr_mode(&expr.object, mode, diagnostics);
            for expr in &expr.args {
                validate_expr_mode(expr, mode, diagnostics);
            }
        }
        Expr::StaticCall(expr) => {
            for expr in &expr.args {
                validate_expr_mode(expr, mode, diagnostics);
            }
        }
        Expr::Assign(expr) => validate_expr_mode(&expr.value, mode, diagnostics),
        Expr::Require(expr) => validate_expr_mode(&expr.path, mode, diagnostics),
        Expr::Binary(expr) => {
            validate_expr_mode(&expr.left, mode, diagnostics);
            validate_expr_mode(&expr.right, mode, diagnostics);
        }
        Expr::Unary(expr) => validate_expr_mode(&expr.expr, mode, diagnostics),
        Expr::TypeAscription(expr) => validate_expr_mode(&expr.expr, mode, diagnostics),
        Expr::Field(expr) => validate_expr_mode(&expr.object, mode, diagnostics),
        Expr::Index(expr) => {
            validate_expr_mode(&expr.collection, mode, diagnostics);
            validate_expr_mode(&expr.index, mode, diagnostics);
        }
        Expr::Object(expr) => {
            for field in &expr.fields {
                validate_expr_mode(&field.value, mode, diagnostics);
            }
        }
        Expr::List(expr) => {
            for value in &expr.values {
                validate_expr_mode(value, mode, diagnostics);
            }
        }
        Expr::Array(expr) => {
            for element in &expr.elements {
                if let Some(key) = &element.key {
                    validate_expr_mode(key, mode, diagnostics);
                    diagnostics.push(Diagnostic::new(
                        "keyed array elements are not allowed in strict mode",
                        element.span,
                    ));
                }
                validate_expr_mode(&element.value, mode, diagnostics);
            }
        }
        Expr::Null(_)
        | Expr::Bool(_)
        | Expr::String(_)
        | Expr::Number(_)
        | Expr::Variable(_)
        | Expr::MagicConstant(_) => {}
    }
}
