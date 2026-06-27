use echo_ast::{Expr, NamespaceSource, Program, ReceiverConst, Stmt, TypedParam};
use echo_diagnostics::Diagnostic;
use echo_source::Span;
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
            for arg in &statement.args {
                validate_expr_mode(&arg.value, mode, diagnostics);
            }
        }
        Stmt::DynamicFunctionCall(statement) => {
            diagnostics.push(Diagnostic::new(
                "dynamic function calls are not allowed in strict mode",
                statement.span,
            ));
            for arg in &statement.args {
                validate_expr_mode(&arg.value, mode, diagnostics);
            }
        }
        Stmt::FunctionDecl(statement) => {
            validate_params_mode(&statement.params, statement.span, mode, diagnostics);
            for statement in &statement.body {
                validate_statement_mode(statement, mode, diagnostics);
            }
        }
        Stmt::Assign(statement) => {
            validate_receiver_assignment(&statement.name, statement.span, mode, diagnostics);
            validate_expr_mode(&statement.value, mode, diagnostics);
        }
        Stmt::CoalesceAssign(statement) => {
            validate_receiver_assignment(&statement.name, statement.span, mode, diagnostics);
            validate_expr_mode(&statement.value, mode, diagnostics);
        }
        Stmt::ListAssign(statement) => {
            for target in &statement.targets {
                validate_receiver_assignment(target, statement.span, mode, diagnostics);
            }
            validate_expr_mode(&statement.value, mode, diagnostics);
        }
        Stmt::Let(statement) => {
            validate_receiver_declaration(&statement.name, statement.span, mode, diagnostics);
            validate_expr_mode(&statement.value, mode, diagnostics);
        }
        Stmt::AssignRef(statement) => diagnostics.push(Diagnostic::new(
            "PHP references are not allowed in strict mode",
            statement.span,
        )),
        Stmt::Return(statement) => {
            if let Some(value) = &statement.value {
                validate_expr_mode(value, mode, diagnostics);
            }
        }
        Stmt::Throw(statement) => validate_expr_mode(&statement.value, mode, diagnostics),
        Stmt::Yield(statement) => validate_expr_mode(&statement.value, mode, diagnostics),
        Stmt::Expr(statement) => validate_expr_mode(&statement.expr, mode, diagnostics),
        Stmt::Loop(statement) => {
            for statement in &statement.body {
                validate_statement_mode(statement, mode, diagnostics);
            }
        }
        Stmt::While(statement) => {
            validate_expr_mode(&statement.condition, mode, diagnostics);
            for statement in &statement.body {
                validate_statement_mode(statement, mode, diagnostics);
            }
        }
        Stmt::Foreach(statement) => {
            validate_expr_mode(&statement.iterable, mode, diagnostics);
            for statement in &statement.body {
                validate_statement_mode(statement, mode, diagnostics);
            }
        }
        Stmt::Try(statement) => {
            for statement in &statement.body {
                validate_statement_mode(statement, mode, diagnostics);
            }
            for catch in &statement.catches {
                for statement in &catch.body {
                    validate_statement_mode(statement, mode, diagnostics);
                }
            }
            for statement in &statement.finally_body {
                validate_statement_mode(statement, mode, diagnostics);
            }
        }
        Stmt::If(statement) => {
            validate_expr_mode(&statement.condition, mode, diagnostics);
            for statement in &statement.body {
                validate_statement_mode(statement, mode, diagnostics);
            }
            for clause in &statement.elseif_clauses {
                validate_expr_mode(&clause.condition, mode, diagnostics);
                for statement in &clause.body {
                    validate_statement_mode(statement, mode, diagnostics);
                }
            }
            for statement in &statement.else_body {
                validate_statement_mode(statement, mode, diagnostics);
            }
        }
        Stmt::Break(statement) => {
            if let Some(value) = &statement.value {
                validate_expr_mode(value, mode, diagnostics);
            }
        }
        Stmt::Continue(statement) => {
            if let Some(value) = &statement.value {
                validate_expr_mode(value, mode, diagnostics);
            }
        }
        Stmt::Append(statement) => {
            validate_expr_mode(&statement.target, mode, diagnostics);
            validate_expr_mode(&statement.value, mode, diagnostics);
        }
        Stmt::UnnamedExport(statement) => validate_expr_mode(&statement.value, mode, diagnostics),
        Stmt::Namespace(statement) => {
            if statement.source == NamespaceSource::Std && !mode.allows_std_namespace() {
                diagnostics.push(Diagnostic::new(
                    "std namespace declarations are only allowed in trusted stdlib source",
                    statement.span,
                ));
            }
        }
        Stmt::ClassDecl(statement) => {
            for member in &statement.members {
                match member {
                    echo_ast::ClassMember::Method(method) => {
                        validate_params_mode(&method.params, method.span, mode, diagnostics);
                        for statement in &method.body {
                            validate_statement_mode(statement, mode, diagnostics);
                        }
                    }
                    echo_ast::ClassMember::Property(property) => {
                        if let Some(value) = &property.value {
                            validate_expr_mode(value, mode, diagnostics);
                        }
                    }
                    echo_ast::ClassMember::Const(constant) => {
                        validate_expr_mode(&constant.value, mode, diagnostics);
                    }
                    echo_ast::ClassMember::TraitUse(_) => {}
                }
            }
        }
        Stmt::TraitDecl(statement) => {
            for member in &statement.members {
                match member {
                    echo_ast::ClassMember::Method(method) => {
                        validate_params_mode(&method.params, method.span, mode, diagnostics);
                        for statement in &method.body {
                            validate_statement_mode(statement, mode, diagnostics);
                        }
                    }
                    echo_ast::ClassMember::Property(property) => {
                        if let Some(value) = &property.value {
                            validate_expr_mode(value, mode, diagnostics);
                        }
                    }
                    echo_ast::ClassMember::Const(constant) => {
                        validate_expr_mode(&constant.value, mode, diagnostics);
                    }
                    echo_ast::ClassMember::TraitUse(_) => {}
                }
            }
        }
        Stmt::ExtendDecl(statement) => {
            for member in &statement.members {
                match member {
                    echo_ast::ClassMember::Method(method) => {
                        validate_params_mode(&method.params, method.span, mode, diagnostics);
                        for statement in &method.body {
                            validate_statement_mode(statement, mode, diagnostics);
                        }
                    }
                    echo_ast::ClassMember::Property(property) => {
                        if let Some(value) = &property.value {
                            validate_expr_mode(value, mode, diagnostics);
                        }
                    }
                    echo_ast::ClassMember::Const(constant) => {
                        validate_expr_mode(&constant.value, mode, diagnostics);
                    }
                    echo_ast::ClassMember::TraitUse(_) => {}
                }
            }
        }
        Stmt::Use(_) | Stmt::Import(_) | Stmt::TypeDecl(_) => {}
    }
}

fn validate_params_mode(
    params: &[TypedParam],
    span: Span,
    mode: ValidationMode,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for param in params {
        validate_receiver_declaration(&param.name, span, mode, diagnostics);
        if let Some(value) = &param.default_value {
            validate_expr_mode(value, mode, diagnostics);
        }
    }
}

fn validate_receiver_declaration(
    name: &str,
    span: Span,
    mode: ValidationMode,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if mode.validates_strict() && ReceiverConst::from_variable_name(name).is_some() {
        diagnostics.push(Diagnostic::new(
            format!("${name} is a compiler-provided receiver constant and cannot be declared."),
            span,
        ));
    }
}

fn validate_receiver_assignment(
    name: &str,
    span: Span,
    mode: ValidationMode,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if mode.validates_strict() && ReceiverConst::from_variable_name(name).is_some() {
        diagnostics.push(Diagnostic::new(
            format!("${name} is a compiler-provided receiver constant and cannot be assigned."),
            span,
        ));
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
            for arg in &expr.args {
                validate_expr_mode(&arg.value, mode, diagnostics);
            }
        }
        Expr::DynamicFunctionCall(expr) => {
            for arg in &expr.args {
                validate_expr_mode(&arg.value, mode, diagnostics);
            }
        }
        Expr::DynamicCall(expr) => {
            validate_expr_mode(&expr.callee, mode, diagnostics);
            for arg in &expr.args {
                validate_expr_mode(&arg.value, mode, diagnostics);
            }
        }
        Expr::MethodCall(expr) => {
            validate_expr_mode(&expr.object, mode, diagnostics);
            for arg in &expr.args {
                validate_expr_mode(&arg.value, mode, diagnostics);
            }
        }
        Expr::StaticCall(expr) => {
            for arg in &expr.args {
                validate_expr_mode(&arg.value, mode, diagnostics);
            }
        }
        Expr::New(expr) => {
            if let echo_ast::NewTarget::Expr(target) = &expr.target {
                validate_expr_mode(target, mode, diagnostics);
            }
            for arg in &expr.args {
                validate_expr_mode(&arg.value, mode, diagnostics);
            }
        }
        Expr::Closure(expr) => {
            validate_params_mode(&expr.params, expr.span, mode, diagnostics);
            for statement in &expr.body {
                validate_statement_mode(statement, mode, diagnostics);
            }
        }
        Expr::ArrowFunction(expr) => {
            validate_params_mode(&expr.params, expr.span, mode, diagnostics);
            validate_expr_mode(&expr.body, mode, diagnostics);
        }
        Expr::Assign(expr) => {
            validate_receiver_assignment(&expr.name, expr.span, mode, diagnostics);
            validate_expr_mode(&expr.value, mode, diagnostics);
        }
        Expr::StaticPropertyAssign(expr) | Expr::StaticPropertyCoalesceAssign(expr) => {
            validate_expr_mode(&expr.value, mode, diagnostics)
        }
        Expr::Include(expr) => validate_expr_mode(&expr.path, mode, diagnostics),
        Expr::Binary(expr) => {
            validate_expr_mode(&expr.left, mode, diagnostics);
            validate_expr_mode(&expr.right, mode, diagnostics);
        }
        Expr::Ternary(expr) => {
            validate_expr_mode(&expr.condition, mode, diagnostics);
            validate_expr_mode(&expr.if_true, mode, diagnostics);
            validate_expr_mode(&expr.if_false, mode, diagnostics);
        }
        Expr::Match(expr) => {
            validate_expr_mode(&expr.subject, mode, diagnostics);
            for arm in &expr.arms {
                for condition in &arm.conditions {
                    validate_expr_mode(condition, mode, diagnostics);
                }
                validate_expr_mode(&arm.value, mode, diagnostics);
            }
        }
        Expr::Unary(expr) => validate_expr_mode(&expr.expr, mode, diagnostics),
        Expr::Cast(expr) => validate_expr_mode(&expr.expr, mode, diagnostics),
        Expr::TypeAscription(expr) => validate_expr_mode(&expr.expr, mode, diagnostics),
        Expr::Field(expr) => validate_expr_mode(&expr.object, mode, diagnostics),
        Expr::Index(expr) => {
            validate_expr_mode(&expr.collection, mode, diagnostics);
            validate_expr_mode(&expr.index, mode, diagnostics);
        }
        Expr::TargetAssign(expr) => {
            validate_expr_mode(&expr.target, mode, diagnostics);
            validate_expr_mode(&expr.value, mode, diagnostics);
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
        | Expr::Constant(_)
        | Expr::ReceiverConst(_)
        | Expr::StaticPropertyFetch(_)
        | Expr::ClassConstantFetch(_)
        | Expr::MagicConstant(_) => {}
    }
}
