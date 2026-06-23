use std::collections::HashMap;

use echo_ast::{BinaryOp, Expr, FunctionDeclStmt, Program, Stmt, UnaryOp};
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
                for expr in &statement.args {
                    self.analyze_expr(expr);
                }
            }
            Stmt::DynamicFunctionCall(statement) => {
                self.resolve_variable(&statement.name, statement.span);
                for expr in &statement.args {
                    self.analyze_expr(expr);
                }
            }
            Stmt::FunctionDecl(statement) => self.analyze_function_decl(statement),
            Stmt::Assign(statement) => {
                let ty = self.analyze_expr(&statement.value);
                self.bind_variable(&statement.name, ty, statement.span);
            }
            Stmt::Let(statement) => {
                let value_ty = self.analyze_expr(&statement.value);
                let ty = statement
                    .ty
                    .as_ref()
                    .map(|ty| Type::Named(ty.clone()))
                    .unwrap_or(value_ty);
                self.bind_variable(&statement.name, ty, statement.span);
            }
            Stmt::AssignRef(statement) => {
                let ty = self
                    .resolve_variable(&statement.target, statement.span)
                    .unwrap_or(Type::Unknown);
                self.bind_variable(&statement.name, ty, statement.span);
            }
            Stmt::Return(statement) => {
                self.analyze_expr(&statement.value);
            }
            Stmt::Yield(statement) => {
                self.analyze_expr(&statement.value);
            }
            Stmt::Expr(statement) => {
                self.analyze_expr(&statement.expr);
            }
            Stmt::Namespace(_)
            | Stmt::Use(_)
            | Stmt::Import(_)
            | Stmt::ClassDecl(_)
            | Stmt::TypeDecl(_)
            | Stmt::Break(_) => {}
            Stmt::Loop(statement) => self.analyze_statements(&statement.body),
            Stmt::If(statement) => {
                self.analyze_expr(&statement.condition);
                self.analyze_statements(&statement.body);
            }
            Stmt::Append(statement) => {
                let target_ty = self.resolve_variable(&statement.target, statement.span);
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
                self.analyze_expr(&statement.value);
            }
        }
    }

    fn analyze_function_decl(&mut self, statement: &FunctionDeclStmt) {
        let saved_variables = self.variables.clone();
        for param in &statement.params {
            let ty = param
                .ty
                .as_ref()
                .map(|ty| Type::Named(ty.clone()))
                .unwrap_or(Type::Unknown);
            self.bind_variable(&param.name, ty, statement.span);
        }
        self.analyze_statements(&statement.body);
        self.variables = saved_variables;
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
            Expr::FunctionCall(expr) => {
                for arg in &expr.args {
                    self.analyze_expr(arg);
                }
                echo_reflection::function(&expr.name)
                    .and_then(|function| function.return_type.clone())
                    .map(Type::Named)
                    .unwrap_or(Type::Unknown)
            }
            Expr::MethodCall(expr) => {
                self.analyze_expr(&expr.object);
                for arg in &expr.args {
                    self.analyze_expr(arg);
                }
                Type::Unknown
            }
            Expr::StaticCall(expr) => {
                for arg in &expr.args {
                    self.analyze_expr(arg);
                }
                Type::Unknown
            }
            Expr::Assign(expr) => {
                let ty = self.analyze_expr(&expr.value);
                self.variables.insert(
                    expr.name.clone(),
                    VariableInfo {
                        name: expr.name.clone(),
                        ty: ty.clone(),
                        span: expr.span,
                    },
                );
                ty
            }
            Expr::MagicConstant(_) => Type::String,
            Expr::Require(expr) => {
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
                    BinaryOp::Identical | BinaryOp::Is | BinaryOp::IsNot => Type::Bool,
                }
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

    fn resolve_variable(&mut self, name: &str, span: Span) -> Option<Type> {
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

impl Type {
    fn allows_php_append_syntax(&self) -> bool {
        match self {
            Self::Array => true,
            Self::Named(name) => !name.contains('[') && name.starts_with("array"),
            _ => false,
        }
    }

    fn allows_index_access(&self) -> bool {
        match self {
            Self::Array | Self::List => true,
            Self::Named(name) => name.starts_with("array") || name.starts_with("list"),
            Self::Unknown => true,
            _ => false,
        }
    }
}
