use std::collections::HashMap;

use echo_ast::{Expr, Program};
use echo_diagnostics::Diagnostic;
use echo_source::Span;

mod analyzer;
mod index_facts;

pub use index_facts::{index_facts, index_facts_from_source};

#[derive(Debug, Clone)]
pub struct Analysis {
    expression_types: HashMap<SpanKey, Type>,
    variables: HashMap<String, VariableInfo>,
}

impl Analysis {
    pub fn expression_type(&self, expr: &Expr) -> Type {
        self.expression_type_at(expr.span())
    }

    pub fn expression_type_at(&self, span: Span) -> Type {
        self.expression_types
            .get(&SpanKey::from(span))
            .cloned()
            .unwrap_or(Type::Unknown)
    }

    pub fn variable(&self, name: &str) -> Option<&VariableInfo> {
        self.variables.get(name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableInfo {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Null,
    Bool,
    Int,
    Float,
    Number,
    String,
    Array,
    List,
    Object(Option<String>),
    Task,
    Thread,
    Process,
    Never,
    Unknown,
    Named(String),
}

impl Type {
    pub fn display_name(&self) -> String {
        match self {
            Self::Null => "null".to_string(),
            Self::Bool => "bool".to_string(),
            Self::Int => "int".to_string(),
            Self::Float => "float".to_string(),
            Self::Number => "number".to_string(),
            Self::String => "string".to_string(),
            Self::Array => "array".to_string(),
            Self::List => "list".to_string(),
            Self::Object(Some(name)) if !name.is_empty() => name.clone(),
            Self::Object(Some(_)) => "object".to_string(),
            Self::Object(None) => "object".to_string(),
            Self::Task => "task".to_string(),
            Self::Thread => "thread".to_string(),
            Self::Process => "process".to_string(),
            Self::Never => "never".to_string(),
            Self::Unknown => "unknown".to_string(),
            Self::Named(name) => name.clone(),
        }
    }
}

pub fn analyze(program: &Program) -> Result<Analysis, Vec<Diagnostic>> {
    analyzer::analyze_program(program)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SpanKey {
    start: usize,
    end: usize,
}

impl From<Span> for SpanKey {
    fn from(span: Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

#[cfg(test)]
mod tests {
    use echo_ast::{
        AppendStmt, ArrayExpr, AssignStmt, ExprStmt, ForkExpr, IndexExpr, JoinExpr, LetStmt,
        ListExpr, NumberLiteral, ObjectExpr, ObjectField, RunExpr, SpawnExpr, Stmt, StringLiteral,
        VariableExpr,
    };

    use super::*;

    fn program(statements: Vec<Stmt>) -> Program {
        Program {
            open_tag: None,
            statements,
            source_dir: None,
            span: Span::new(0, 0),
        }
    }

    #[test]
    fn tracks_variable_type_from_let_initializer() {
        let variable = Expr::Variable(VariableExpr {
            name: "a".to_string(),
            span: Span::new(12, 14),
        });
        let analysis = analyze(&program(vec![
            Stmt::Let(LetStmt {
                name: "a".to_string(),
                ty: None,
                value: Expr::Array(ArrayExpr {
                    elements: vec![],
                    span: Span::new(9, 11),
                }),
                span: Span::new(0, 11),
            }),
            Stmt::Expr(ExprStmt {
                expr: variable.clone(),
                span: Span::new(12, 14),
            }),
        ]))
        .expect("program should analyze");

        assert_eq!(
            analysis.variable("a").map(|info| &info.ty),
            Some(&Type::Array)
        );
        assert_eq!(analysis.expression_type(&variable), Type::Array);
    }

    #[test]
    fn reports_undefined_variable() {
        let diagnostics = analyze(&program(vec![Stmt::Expr(ExprStmt {
            expr: Expr::Variable(VariableExpr {
                name: "missing".to_string(),
                span: Span::new(0, 8),
            }),
            span: Span::new(0, 8),
        })]))
        .expect_err("undefined variable should be diagnostic");

        assert_eq!(diagnostics[0].message, "undefined variable `$missing`");
        assert_eq!(diagnostics[0].span, Span::new(0, 8));
    }

    #[test]
    fn infers_literal_and_operator_types() {
        let analysis = analyze(&program(vec![Stmt::Expr(ExprStmt {
            expr: Expr::String(StringLiteral {
                value: "x".to_string(),
                span: Span::new(0, 3),
            }),
            span: Span::new(0, 3),
        })]))
        .expect("program should analyze");

        assert_eq!(analysis.expression_type_at(Span::new(0, 3)), Type::String);

        let analysis = analyze(&program(vec![Stmt::Expr(ExprStmt {
            expr: Expr::Number(NumberLiteral {
                value: "1.5".to_string(),
                span: Span::new(0, 3),
            }),
            span: Span::new(0, 3),
        })]))
        .expect("program should analyze");

        assert_eq!(analysis.expression_type_at(Span::new(0, 3)), Type::Float);
    }

    #[test]
    fn infers_concurrency_handle_types() {
        let analysis = analyze(&program(vec![
            Stmt::Assign(AssignStmt {
                name: "task".to_string(),
                value: Expr::Run(RunExpr::Block {
                    body: vec![],
                    span: Span::new(0, 8),
                }),
                span: Span::new(0, 8),
            }),
            Stmt::Assign(AssignStmt {
                name: "thread".to_string(),
                value: Expr::Fork(ForkExpr::Block {
                    body: vec![],
                    span: Span::new(9, 18),
                }),
                span: Span::new(9, 18),
            }),
            Stmt::Assign(AssignStmt {
                name: "process".to_string(),
                value: Expr::Spawn(SpawnExpr {
                    command: Box::new(Expr::String(StringLiteral {
                        value: "true".to_string(),
                        span: Span::new(25, 31),
                    })),
                    span: Span::new(19, 31),
                }),
                span: Span::new(19, 31),
            }),
        ]))
        .expect("program should analyze");

        assert_eq!(
            analysis.variable("task").map(|info| &info.ty),
            Some(&Type::Task)
        );
        assert_eq!(
            analysis.variable("thread").map(|info| &info.ty),
            Some(&Type::Thread)
        );
        assert_eq!(
            analysis.variable("process").map(|info| &info.ty),
            Some(&Type::Process)
        );
    }

    #[test]
    fn infers_join_type_from_handle_variable_type() {
        let task_join = Expr::Join(JoinExpr {
            handle: Box::new(Expr::Variable(VariableExpr {
                name: "task".to_string(),
                span: Span::new(32, 37),
            })),
            span: Span::new(27, 37),
        });
        let thread_join = Expr::Join(JoinExpr {
            handle: Box::new(Expr::Variable(VariableExpr {
                name: "thread".to_string(),
                span: Span::new(48, 55),
            })),
            span: Span::new(43, 55),
        });
        let process_join = Expr::Join(JoinExpr {
            handle: Box::new(Expr::Variable(VariableExpr {
                name: "process".to_string(),
                span: Span::new(66, 75),
            })),
            span: Span::new(61, 75),
        });
        let analysis = analyze(&program(vec![
            Stmt::Assign(AssignStmt {
                name: "task".to_string(),
                value: Expr::Run(RunExpr::Block {
                    body: vec![],
                    span: Span::new(0, 8),
                }),
                span: Span::new(0, 8),
            }),
            Stmt::Assign(AssignStmt {
                name: "thread".to_string(),
                value: Expr::Fork(ForkExpr::Block {
                    body: vec![],
                    span: Span::new(9, 18),
                }),
                span: Span::new(9, 18),
            }),
            Stmt::Assign(AssignStmt {
                name: "process".to_string(),
                value: Expr::Spawn(SpawnExpr {
                    command: Box::new(Expr::String(StringLiteral {
                        value: "true".to_string(),
                        span: Span::new(25, 31),
                    })),
                    span: Span::new(19, 31),
                }),
                span: Span::new(19, 31),
            }),
            Stmt::Expr(ExprStmt {
                expr: task_join.clone(),
                span: Span::new(27, 37),
            }),
            Stmt::Expr(ExprStmt {
                expr: thread_join.clone(),
                span: Span::new(43, 55),
            }),
            Stmt::Expr(ExprStmt {
                expr: process_join.clone(),
                span: Span::new(61, 75),
            }),
        ]))
        .expect("program should analyze");

        assert_eq!(analysis.expression_type(&task_join), Type::Unknown);
        assert_eq!(analysis.expression_type(&thread_join), Type::Unknown);
        assert_eq!(analysis.expression_type(&process_join), Type::Int);
    }

    #[test]
    fn allows_php_append_syntax_for_arrays() {
        analyze(&program(vec![
            Stmt::Let(LetStmt {
                name: "a".to_string(),
                ty: None,
                value: Expr::Array(ArrayExpr {
                    elements: vec![],
                    span: Span::new(9, 11),
                }),
                span: Span::new(0, 11),
            }),
            Stmt::Append(AppendStmt {
                target: "a".to_string(),
                value: Expr::Number(NumberLiteral {
                    value: "1".to_string(),
                    span: Span::new(19, 20),
                }),
                span: Span::new(12, 21),
            }),
        ]))
        .expect("array append should analyze");
    }

    #[test]
    fn rejects_php_append_syntax_for_lists() {
        let diagnostics = analyze(&program(vec![
            Stmt::Let(LetStmt {
                name: "a".to_string(),
                ty: None,
                value: Expr::List(ListExpr {
                    values: vec![],
                    span: Span::new(9, 11),
                }),
                span: Span::new(0, 11),
            }),
            Stmt::Append(AppendStmt {
                target: "a".to_string(),
                value: Expr::Number(NumberLiteral {
                    value: "1".to_string(),
                    span: Span::new(19, 20),
                }),
                span: Span::new(12, 21),
            }),
        ]))
        .expect_err("list append should be rejected");

        assert_eq!(
            diagnostics[0].message,
            "PHP array append syntax requires array target, found list"
        );
    }

    #[test]
    fn rejects_php_append_syntax_for_fixed_size_arrays() {
        let diagnostics = analyze(&program(vec![
            Stmt::Let(LetStmt {
                name: "a".to_string(),
                ty: Some("array<int>[3]".to_string()),
                value: Expr::Array(ArrayExpr {
                    elements: vec![],
                    span: Span::new(22, 24),
                }),
                span: Span::new(0, 24),
            }),
            Stmt::Append(AppendStmt {
                target: "a".to_string(),
                value: Expr::Number(NumberLiteral {
                    value: "1".to_string(),
                    span: Span::new(32, 33),
                }),
                span: Span::new(25, 34),
            }),
        ]))
        .expect_err("fixed-size array append should be rejected");

        assert_eq!(
            diagnostics[0].message,
            "PHP array append syntax requires array target, found array<int>[3]"
        );
    }

    #[test]
    fn allows_index_access_for_arrays_and_lists() {
        analyze(&program(vec![
            Stmt::Expr(ExprStmt {
                expr: Expr::Index(Box::new(IndexExpr {
                    collection: Expr::Array(ArrayExpr {
                        elements: vec![],
                        span: Span::new(0, 2),
                    }),
                    index: Expr::Number(NumberLiteral {
                        value: "0".to_string(),
                        span: Span::new(3, 4),
                    }),
                    span: Span::new(0, 5),
                })),
                span: Span::new(0, 5),
            }),
            Stmt::Expr(ExprStmt {
                expr: Expr::Index(Box::new(IndexExpr {
                    collection: Expr::List(ListExpr {
                        values: vec![],
                        span: Span::new(6, 8),
                    }),
                    index: Expr::Number(NumberLiteral {
                        value: "0".to_string(),
                        span: Span::new(9, 10),
                    }),
                    span: Span::new(6, 11),
                })),
                span: Span::new(6, 11),
            }),
        ]))
        .expect("array and list index access should analyze");
    }

    #[test]
    fn rejects_index_access_for_objects() {
        let diagnostics = analyze(&program(vec![Stmt::Expr(ExprStmt {
            expr: Expr::Index(Box::new(IndexExpr {
                collection: Expr::Object(ObjectExpr {
                    name: String::new(),
                    fields: vec![ObjectField {
                        name: "a".to_string(),
                        value: Expr::Number(NumberLiteral {
                            value: "1".to_string(),
                            span: Span::new(5, 6),
                        }),
                    }],
                    span: Span::new(0, 7),
                }),
                index: Expr::String(StringLiteral {
                    value: "a".to_string(),
                    span: Span::new(8, 11),
                }),
                span: Span::new(0, 12),
            })),
            span: Span::new(0, 12),
        })]))
        .expect_err("object index access should be rejected");

        assert_eq!(
            diagnostics[0].message,
            "index access requires array or list target, found object"
        );
    }
}
