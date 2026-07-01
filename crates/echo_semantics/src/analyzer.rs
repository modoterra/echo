use std::collections::HashMap;

use echo_ast::{
    BinaryOp, ClassMember, EnumMember, Expr, FunctionDeclStmt, InterfaceMember, MethodDecl,
    Program, ReceiverConst, Stmt, UnaryOp,
};
use echo_diagnostics::Diagnostic;
use echo_source::Span;

use crate::{Analysis, SpanKey, Type, VariableInfo};

pub(super) fn analyze_program(program: &Program) -> Result<Analysis, Vec<Diagnostic>> {
    let mut analyzer = Analyzer::default();
    analyzer.analyze_statements(&program.statements);
    if analyzer.diagnostics.is_empty() {
        Ok(Analysis {
            expression_types: analyzer.expression_types,
            variables: analyzer.variables,
        })
    } else {
        Err(analyzer.diagnostics)
    }
}

#[derive(Default)]
struct Analyzer {
    expression_types: HashMap<SpanKey, Type>,
    variables: HashMap<String, VariableInfo>,
    diagnostics: Vec<Diagnostic>,
    receiver_context: ReceiverContext,
    unnamed_export_span: Option<Span>,
}

#[derive(Debug, Clone, Copy, Default)]
struct ReceiverContext {
    has_instance: bool,
    has_self_type: bool,
    has_parent: bool,
}

impl Analyzer {
    fn analyze_statements(&mut self, statements: &[Stmt]) {
        for statement in statements {
            self.analyze_statement(statement);
        }
    }

    fn analyze_statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Echo(statement) => {
                for expr in &statement.exprs {
                    self.analyze_expr(expr);
                }
            }
            Stmt::FunctionCall(statement) => {
                if statement.name.eq_ignore_ascii_case("unset") {
                    for arg in &statement.args {
                        self.analyze_unset_target(&arg.value);
                    }
                } else {
                    for arg in &statement.args {
                        self.analyze_call_arg(&arg.value);
                    }
                }
            }
            Stmt::DynamicFunctionCall(statement) => {
                self.resolve_variable(&statement.name, statement.span);
                for arg in &statement.args {
                    self.analyze_call_arg(&arg.value);
                }
            }
            Stmt::FunctionDecl(statement) => self.analyze_function_decl(statement),
            Stmt::Assign(statement) => {
                let ty = self.analyze_expr(&statement.value);
                if !self.reject_receiver_assignment(&statement.name, statement.span) {
                    self.bind_variable(&statement.name, ty, statement.span);
                }
            }
            Stmt::CoalesceAssign(statement) => {
                let ty = self.analyze_expr(&statement.value);
                if !self.reject_receiver_assignment(&statement.name, statement.span) {
                    self.bind_variable(&statement.name, ty, statement.span);
                }
            }
            Stmt::ListAssign(statement) => {
                self.analyze_expr(&statement.value);
                for target in &statement.targets {
                    if !self.reject_receiver_assignment(target, statement.span) {
                        self.bind_variable(target, Type::Unknown, statement.span);
                    }
                }
            }
            Stmt::Let(statement) => {
                let value_ty = self.analyze_expr(&statement.value);
                let ty = statement
                    .ty
                    .as_ref()
                    .map(|ty| Type::Named(ty.clone()))
                    .unwrap_or(value_ty);
                if !self.reject_receiver_binding(&statement.name, statement.span) {
                    self.bind_variable(&statement.name, ty, statement.span);
                }
            }
            Stmt::AssignRef(statement) => {
                let ty = self
                    .resolve_variable(&statement.target, statement.span)
                    .unwrap_or(Type::Unknown);
                if !self.reject_receiver_binding(&statement.name, statement.span) {
                    self.bind_variable(&statement.name, ty, statement.span);
                }
            }
            Stmt::Return(statement) => {
                if let Some(value) = &statement.value {
                    self.analyze_expr(value);
                }
            }
            Stmt::Throw(statement) => {
                self.analyze_expr(&statement.value);
            }
            Stmt::Yield(statement) => {
                self.analyze_expr(&statement.value);
            }
            Stmt::Expr(statement) => {
                self.analyze_expr(&statement.expr);
            }
            Stmt::UnnamedExport(statement) => {
                if self.unnamed_export_span.replace(statement.span).is_some() {
                    self.diagnostics.push(Diagnostic::new(
                        "only one unnamed export is allowed per module.",
                        statement.span,
                    ));
                }
                self.analyze_expr(&statement.value);
            }
            Stmt::ClassDecl(statement) => {
                for member in &statement.members {
                    match member {
                        ClassMember::Method(method) => self.analyze_method_decl(
                            method,
                            ReceiverContext {
                                has_instance: !method.is_static,
                                has_self_type: true,
                                has_parent: statement.parent.is_some(),
                            },
                        ),
                        ClassMember::Property(property) => {
                            if let Some(value) = &property.value {
                                self.analyze_expr(value);
                            }
                        }
                        ClassMember::Const(constant) => {
                            self.analyze_expr(&constant.value);
                        }
                        ClassMember::TraitUse(_) => {}
                    }
                }
            }
            Stmt::InterfaceDecl(statement) => {
                for member in &statement.members {
                    match member {
                        InterfaceMember::Method(method) => self.analyze_method_decl(
                            method,
                            ReceiverContext {
                                has_instance: false,
                                has_self_type: true,
                                has_parent: !statement.parents.is_empty(),
                            },
                        ),
                        InterfaceMember::Const(constant) => {
                            self.analyze_expr(&constant.value);
                        }
                    }
                }
            }
            Stmt::TraitDecl(statement) => {
                for member in &statement.members {
                    match member {
                        ClassMember::Method(method) => self.analyze_method_decl(
                            method,
                            ReceiverContext {
                                has_instance: !method.is_static,
                                has_self_type: true,
                                has_parent: false,
                            },
                        ),
                        ClassMember::Property(property) => {
                            if let Some(value) = &property.value {
                                self.analyze_expr(value);
                            }
                        }
                        ClassMember::Const(constant) => {
                            self.analyze_expr(&constant.value);
                        }
                        ClassMember::TraitUse(_) => {}
                    }
                }
            }
            Stmt::EnumDecl(statement) => {
                for member in &statement.members {
                    match member {
                        EnumMember::Case(case) => {
                            if let Some(value) = &case.value {
                                self.analyze_expr(value);
                            }
                        }
                        EnumMember::Method(method) => self.analyze_method_decl(
                            method,
                            ReceiverContext {
                                has_instance: !method.is_static,
                                has_self_type: true,
                                has_parent: false,
                            },
                        ),
                        EnumMember::TraitUse(_) => {}
                    }
                }
            }
            Stmt::FacetDecl(statement) => {
                for member in &statement.members {
                    match member {
                        ClassMember::Method(method) => self.analyze_method_decl(
                            method,
                            ReceiverContext {
                                has_instance: !method.is_static,
                                has_self_type: true,
                                has_parent: false,
                            },
                        ),
                        ClassMember::Property(property) => {
                            if let Some(value) = &property.value {
                                self.analyze_expr(value);
                            }
                        }
                        ClassMember::Const(constant) => {
                            self.analyze_expr(&constant.value);
                        }
                        ClassMember::TraitUse(_) => {}
                    }
                }
            }
            Stmt::Namespace(_)
            | Stmt::Compile(_)
            | Stmt::Use(_)
            | Stmt::Import(_)
            | Stmt::TypeDecl(_)
            | Stmt::Break(_)
            | Stmt::Continue(_) => {}
            Stmt::Loop(statement) => self.analyze_statements(&statement.body),
            Stmt::While(statement) => {
                self.analyze_expr(&statement.condition);
                self.analyze_statements(&statement.body);
            }
            Stmt::Foreach(statement) => {
                self.analyze_expr(&statement.iterable);
                if let Some(key) = &statement.key {
                    self.bind_variable(key, Type::Unknown, statement.span);
                }
                self.bind_variable(&statement.value, Type::Unknown, statement.span);
                self.analyze_statements(&statement.body);
            }
            Stmt::If(statement) => {
                self.analyze_expr(&statement.condition);
                self.analyze_statements(&statement.body);
                for clause in &statement.elseif_clauses {
                    self.analyze_expr(&clause.condition);
                    self.analyze_statements(&clause.body);
                }
                self.analyze_statements(&statement.else_body);
            }
            Stmt::Try(statement) => {
                self.analyze_statements(&statement.body);
                for catch in &statement.catches {
                    let saved_variables = self.variables.clone();
                    if let Some(variable) = &catch.variable {
                        self.bind_variable(variable, Type::Unknown, catch.span);
                    }
                    self.analyze_statements(&catch.body);
                    self.variables = saved_variables;
                }
                self.analyze_statements(&statement.finally_body);
            }
            Stmt::Append(statement) => {
                if let Expr::Variable(target) = &statement.target {
                    let target_ty = self.resolve_variable(&target.name, statement.span);
                    if let Some(ty) = target_ty {
                        if !ty.allows_php_append_syntax() {
                            self.diagnostics.push(Diagnostic::new(
                                format!(
                                    "PHP array append syntax requires array target, found {}",
                                    ty.display_name()
                                ),
                                statement.span,
                            ));
                        }
                    }
                } else {
                    self.analyze_expr(&statement.target);
                }
                self.analyze_expr(&statement.value);
            }
        }
    }

    fn analyze_function_decl(&mut self, statement: &FunctionDeclStmt) {
        let saved_variables = self.variables.clone();
        for param in &statement.params {
            if let Some(default_value) = &param.default_value {
                self.analyze_expr(default_value);
            }
            let ty = param
                .ty
                .as_ref()
                .map(|ty| Type::Named(ty.clone()))
                .unwrap_or(Type::Unknown);
            if !self.reject_receiver_binding(&param.name, statement.span) {
                self.bind_variable(&param.name, ty, statement.span);
            }
        }
        self.analyze_statements(&statement.body);
        self.variables = saved_variables;
    }

    fn analyze_unset_target(&mut self, target: &Expr) {
        match target {
            Expr::Variable(_) => {}
            Expr::Field(expr) => {
                self.analyze_expr(&expr.object);
            }
            Expr::Index(expr) => {
                self.analyze_unset_target(&expr.collection);
                self.analyze_expr(&expr.index);
            }
            _ => {
                self.analyze_expr(target);
                ()
            }
        }
    }

    fn analyze_method_decl(&mut self, method: &MethodDecl, context: ReceiverContext) {
        let saved_variables = self.variables.clone();
        let saved_context = self.receiver_context;
        self.receiver_context = context;
        for param in &method.params {
            if let Some(default_value) = &param.default_value {
                self.analyze_expr(default_value);
            }
            let ty = param
                .ty
                .as_ref()
                .map(|ty| Type::Named(ty.clone()))
                .unwrap_or(Type::Unknown);
            if !self.reject_receiver_binding(&param.name, method.span) {
                self.bind_variable(&param.name, ty, method.span);
            }
        }
        self.analyze_statements(&method.body);
        self.receiver_context = saved_context;
        self.variables = saved_variables;
    }

    fn analyze_class_member(&mut self, member: &ClassMember, has_parent: bool) {
        match member {
            ClassMember::Method(method) => self.analyze_method_decl(
                method,
                ReceiverContext {
                    has_instance: !method.is_static,
                    has_self_type: true,
                    has_parent,
                },
            ),
            ClassMember::Property(property) => {
                if let Some(value) = &property.value {
                    self.analyze_expr(value);
                }
            }
            ClassMember::Const(constant) => {
                self.analyze_expr(&constant.value);
            }
            ClassMember::TraitUse(_) => {}
        }
    }

    fn analyze_expr(&mut self, expr: &Expr) -> Type {
        let ty = match expr {
            Expr::Null(_) => Type::Null,
            Expr::Bool(_) => Type::Bool,
            Expr::String(_) => Type::String,
            Expr::Number(expr) if expr.value.contains(['.', 'e', 'E']) => Type::Float,
            Expr::Number(_) => Type::Int,
            Expr::Variable(expr) => self
                .resolve_variable(&expr.name, expr.span)
                .unwrap_or(Type::Unknown),
            Expr::Constant(expr) if expr.name == "PHP_VERSION_ID" => Type::Int,
            Expr::Constant(expr)
                if matches!(
                    expr.name.as_str(),
                    "PHP_VERSION" | "PHP_SAPI" | "PHP_EOL" | "STDERR"
                ) =>
            {
                Type::String
            }
            Expr::Constant(_) => Type::Unknown,
            Expr::ReceiverConst(expr) => self.analyze_receiver_const(expr.kind, expr.span),
            Expr::StaticPropertyFetch(_) => Type::Unknown,
            Expr::StaticPropertyAssign(expr) => {
                self.analyze_expr(&expr.value);
                Type::Unknown
            }
            Expr::StaticPropertyCoalesceAssign(expr) => {
                self.analyze_expr(&expr.value);
                Type::Unknown
            }
            Expr::ClassConstantFetch(_) => Type::Unknown,
            Expr::FunctionCall(expr) => {
                for arg in &expr.args {
                    self.analyze_call_arg(&arg.value);
                }
                echo_reflection::function(&expr.name)
                    .and_then(|function| function.return_type.clone())
                    .map(Type::Named)
                    .unwrap_or(Type::Unknown)
            }
            Expr::Print(expr) => {
                self.analyze_expr(&expr.value);
                Type::Int
            }
            Expr::DynamicFunctionCall(expr) => {
                self.resolve_variable(&expr.name, expr.span);
                for arg in &expr.args {
                    self.analyze_call_arg(&arg.value);
                }
                Type::Unknown
            }
            Expr::DynamicCall(expr) => {
                self.analyze_expr(&expr.callee);
                for arg in &expr.args {
                    self.analyze_call_arg(&arg.value);
                }
                Type::Unknown
            }
            Expr::MethodCall(expr) => {
                self.analyze_expr(&expr.object);
                for arg in &expr.args {
                    self.analyze_call_arg(&arg.value);
                }
                Type::Unknown
            }
            Expr::StaticCall(expr) => {
                for arg in &expr.args {
                    self.analyze_call_arg(&arg.value);
                }
                Type::Unknown
            }
            Expr::New(expr) => {
                let ty = match &expr.target {
                    echo_ast::NewTarget::Class(class_name) => Some(class_name.as_string()),
                    echo_ast::NewTarget::Expr(target) => {
                        self.analyze_expr(target);
                        None
                    }
                    echo_ast::NewTarget::AnonymousClass(class) => {
                        for member in &class.members {
                            self.analyze_class_member(member, class.parent.is_some());
                        }
                        None
                    }
                };
                for arg in &expr.args {
                    self.analyze_call_arg(&arg.value);
                }
                Type::Object(ty)
            }
            Expr::Closure(expr) => {
                let saved_variables = self.variables.clone();
                for param in &expr.params {
                    if let Some(default_value) = &param.default_value {
                        self.analyze_expr(default_value);
                    }
                    let ty = param
                        .ty
                        .as_ref()
                        .map(|ty| Type::Named(ty.clone()))
                        .unwrap_or(Type::Unknown);
                    if !self.reject_receiver_binding(&param.name, expr.span) {
                        self.bind_variable(&param.name, ty, expr.span);
                    }
                }
                self.analyze_statements(&expr.body);
                self.variables = saved_variables;
                Type::Unknown
            }
            Expr::ArrowFunction(expr) => {
                let saved_variables = self.variables.clone();
                for param in &expr.params {
                    if let Some(default_value) = &param.default_value {
                        self.analyze_expr(default_value);
                    }
                    let ty = param
                        .ty
                        .as_ref()
                        .map(|ty| Type::Named(ty.clone()))
                        .unwrap_or(Type::Unknown);
                    if !self.reject_receiver_binding(&param.name, expr.span) {
                        self.bind_variable(&param.name, ty, expr.span);
                    }
                }
                self.analyze_expr(&expr.body);
                self.variables = saved_variables;
                Type::Unknown
            }
            Expr::Assign(expr) => {
                let ty = self.analyze_expr(&expr.value);
                if !self.reject_receiver_assignment(&expr.name, expr.span) {
                    self.variables.insert(
                        expr.name.clone(),
                        VariableInfo {
                            name: expr.name.clone(),
                            ty: ty.clone(),
                            span: expr.span,
                        },
                    );
                }
                ty
            }
            Expr::MagicConstant(_) => Type::String,
            Expr::Include(expr) => {
                self.analyze_expr(&expr.path);
                Type::Bool
            }
            Expr::Defer(expr) => {
                self.analyze_statements(&expr.body);
                Type::Task
            }
            Expr::Run(expr) => {
                match expr {
                    echo_ast::RunExpr::Block { body, .. } => self.analyze_statements(body),
                    echo_ast::RunExpr::Task { expr, .. } => {
                        self.analyze_expr(expr);
                    }
                    echo_ast::RunExpr::Group { entries, .. } => {
                        for entry in entries {
                            self.analyze_statements(entry);
                        }
                        return Type::List;
                    }
                }
                Type::Task
            }
            Expr::Fork(expr) => {
                match expr {
                    echo_ast::ForkExpr::Block { body, .. } => self.analyze_statements(body),
                    echo_ast::ForkExpr::Task { expr, .. } => {
                        self.analyze_expr(expr);
                    }
                }
                Type::Thread
            }
            Expr::Spawn(expr) => {
                self.analyze_expr(&expr.command);
                Type::Process
            }
            Expr::Join(expr) => match self.analyze_expr(&expr.handle) {
                Type::Process => Type::Int,
                Type::Task | Type::Thread | Type::Unknown => Type::Unknown,
                _ => Type::Unknown,
            },
            Expr::Loop(expr) => {
                self.analyze_statements(&expr.body);
                Type::Unknown
            }
            Expr::Unary(expr) => {
                self.analyze_expr(&expr.expr);
                match expr.op {
                    UnaryOp::Plus | UnaryOp::Minus => Type::Number,
                    UnaryOp::Not => Type::Bool,
                    UnaryOp::Clone => Type::Object(None),
                }
            }
            Expr::Cast(expr) => {
                self.analyze_expr(&expr.expr);
                match expr.ty.as_str() {
                    "array" => Type::Array,
                    "string" => Type::String,
                    "int" | "integer" | "float" | "double" => Type::Number,
                    "bool" | "boolean" => Type::Bool,
                    _ => Type::Unknown,
                }
            }
            Expr::Binary(expr) => {
                self.analyze_expr(&expr.left);
                self.analyze_expr(&expr.right);
                match expr.op {
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::Mod
                    | BinaryOp::Pow => Type::Number,
                    BinaryOp::Concat => Type::String,
                    BinaryOp::Coalesce => Type::Unknown,
                    BinaryOp::LessThan
                    | BinaryOp::GreaterThanOrEqual
                    | BinaryOp::Identical
                    | BinaryOp::NotIdentical
                    | BinaryOp::Equal
                    | BinaryOp::NotEqual
                    | BinaryOp::InstanceOf
                    | BinaryOp::And
                    | BinaryOp::Or
                    | BinaryOp::Is
                    | BinaryOp::IsNot => Type::Bool,
                }
            }
            Expr::Ternary(expr) => {
                self.analyze_expr(&expr.condition);
                self.analyze_expr(&expr.if_true);
                self.analyze_expr(&expr.if_false);
                Type::Unknown
            }
            Expr::Match(expr) => {
                self.analyze_expr(&expr.subject);
                for arm in &expr.arms {
                    for condition in &arm.conditions {
                        self.analyze_expr(condition);
                    }
                    self.analyze_expr(&arm.value);
                }
                Type::Unknown
            }
            Expr::Field(expr) => {
                self.analyze_expr(&expr.object);
                Type::Unknown
            }
            Expr::Index(expr) => {
                let collection_ty = self.analyze_expr(&expr.collection);
                self.analyze_expr(&expr.index);
                if !collection_ty.allows_index_access() {
                    self.diagnostics.push(Diagnostic::new(
                        format!(
                            "index access requires array or list target, found {}",
                            collection_ty.display_name()
                        ),
                        expr.span,
                    ));
                }
                Type::Unknown
            }
            Expr::TargetAssign(expr) => {
                self.analyze_expr(&expr.target);
                self.analyze_expr(&expr.value);
                Type::Unknown
            }
            Expr::TypeAscription(expr) => {
                self.analyze_expr(&expr.expr);
                Type::Named(expr.ty.clone())
            }
            Expr::Object(expr) => {
                for field in &expr.fields {
                    self.analyze_expr(&field.value);
                }
                Type::Object(Some(expr.name.clone()))
            }
            Expr::List(expr) => {
                for value in &expr.values {
                    self.analyze_expr(value);
                }
                Type::List
            }
            Expr::Array(expr) => {
                for element in &expr.elements {
                    if let Some(key) = &element.key {
                        self.analyze_expr(key);
                    }
                    self.analyze_expr(&element.value);
                }
                Type::Array
            }
        };
        self.expression_types
            .insert(SpanKey::from(expr.span()), ty.clone());
        ty
    }

    fn bind_variable(&mut self, name: &str, ty: Type, span: Span) {
        self.variables.insert(
            name.to_string(),
            VariableInfo {
                name: name.to_string(),
                ty,
                span,
            },
        );
    }

    fn analyze_call_arg(&mut self, expr: &Expr) {
        if let Expr::Variable(variable) = expr {
            if !self.variables.contains_key(&variable.name) {
                self.bind_variable(&variable.name, Type::Unknown, variable.span);
                return;
            }
        }
        self.analyze_expr(expr);
    }

    fn analyze_receiver_const(&mut self, kind: ReceiverConst, span: Span) -> Type {
        match kind {
            ReceiverConst::This if !self.receiver_context.has_instance => {
                self.diagnostics.push(Diagnostic::new(
                    "$this is only available inside instance receiver contexts.",
                    span,
                ));
            }
            ReceiverConst::SelfType if !self.receiver_context.has_self_type => {
                self.diagnostics.push(Diagnostic::new(
                    "$self is not a strict Echo receiver; facet methods use the explicit receiver alias.",
                    span,
                ));
            }
            ReceiverConst::Parent if !self.receiver_context.has_parent => {
                self.diagnostics.push(Diagnostic::new(
                    "$parent is only available when the lexical type has a parent.",
                    span,
                ));
            }
            ReceiverConst::Static => {
                self.diagnostics.push(Diagnostic::new(
                    "$static is reserved for late static binding and is not implemented yet.",
                    span,
                ));
            }
            _ => {}
        }
        Type::Unknown
    }

    fn reject_receiver_binding(&mut self, name: &str, span: Span) -> bool {
        if name != ReceiverConst::This.variable_name() {
            return false;
        }
        self.diagnostics.push(Diagnostic::new(
            format!("${name} is a compiler-provided receiver constant and cannot be declared."),
            span,
        ));
        true
    }

    fn reject_receiver_assignment(&mut self, name: &str, span: Span) -> bool {
        if name != ReceiverConst::This.variable_name() {
            return false;
        }
        self.diagnostics.push(Diagnostic::new(
            format!("${name} is a compiler-provided receiver constant and cannot be assigned."),
            span,
        ));
        true
    }

    fn resolve_variable(&mut self, name: &str, span: Span) -> Option<Type> {
        if is_php_superglobal(name) {
            return Some(Type::Array);
        }
        let ty = self.variables.get(name).map(|variable| variable.ty.clone());
        if ty.is_none() {
            self.diagnostics.push(Diagnostic::new(
                format!("undefined variable `${name}`"),
                span,
            ));
        }
        ty
    }
}

fn is_php_superglobal(name: &str) -> bool {
    matches!(
        name,
        "GLOBALS"
            | "_SERVER"
            | "_GET"
            | "_POST"
            | "_FILES"
            | "_COOKIE"
            | "_SESSION"
            | "_REQUEST"
            | "_ENV"
    )
}

impl Type {
    fn allows_php_append_syntax(&self) -> bool {
        match self {
            Self::Array | Self::Unknown => true,
            Self::Named(name) => !name.contains('[') && name.starts_with("array"),
            _ => false,
        }
    }

    fn allows_index_access(&self) -> bool {
        match self {
            Self::Array | Self::List | Self::String => true,
            Self::Named(name) => {
                name == "string" || name.starts_with("array") || name.starts_with("list")
            }
            Self::Unknown => true,
            _ => false,
        }
    }
}
