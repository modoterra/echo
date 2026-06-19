use echo_ast::{BinaryOp, Expr, FunctionDeclStmt, ImportStmt, Stmt, TypedParam, UnaryOp};
use echo_diagnostics::Diagnostic;
use echo_hir::{HirProgram, HirStmt};
use echo_source::Span;

#[derive(Debug, Clone)]
pub struct MirProgram {
    source_dir: Option<String>,
    imports: Vec<ImportStmt>,
    functions: Vec<MirFunction>,
    statements: Vec<MirStmt>,
}

impl MirProgram {
    pub fn source_dir(&self) -> Option<&str> {
        self.source_dir.as_deref()
    }

    pub fn imports(&self) -> &[ImportStmt] {
        &self.imports
    }

    pub fn functions(&self) -> &[MirFunction] {
        &self.functions
    }

    pub fn statements(&self) -> &[MirStmt] {
        &self.statements
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirFunction {
    pub source: FunctionDeclStmt,
    pub name: String,
    pub params: Vec<TypedParam>,
    pub return_type: Option<String>,
    pub body: Vec<MirStmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirStmt {
    Echo {
        source: Stmt,
        exprs: Vec<MirExpr>,
    },
    FunctionCall {
        source: Stmt,
        call: MirFunctionCall,
    },
    DynamicFunctionCall {
        source: Stmt,
        name: String,
        args: Vec<MirExpr>,
    },
    Assign {
        source: Stmt,
        name: String,
        value: MirExpr,
    },
    Let {
        source: Stmt,
        name: String,
        value: MirExpr,
    },
    Return {
        source: Stmt,
        value: MirExpr,
    },
    Expr {
        source: Stmt,
        expr: MirExpr,
    },
    Loop {
        source: Stmt,
        body: Vec<MirStmt>,
    },
    If {
        source: Stmt,
        condition: MirExpr,
        body: Vec<MirStmt>,
    },
    Break {
        source: Stmt,
        value: Option<MirExpr>,
    },
    Append {
        source: Stmt,
        target: String,
        value: MirExpr,
    },
    AssignRef {
        source: Stmt,
        name: String,
        target: String,
    },
    Yield {
        source: Stmt,
        value: MirExpr,
    },
    Noop {
        source: Stmt,
    },
}

impl MirStmt {
    pub fn syntax(&self) -> &Stmt {
        match self {
            Self::Echo { source, .. }
            | Self::FunctionCall { source, .. }
            | Self::DynamicFunctionCall { source, .. }
            | Self::Assign { source, .. }
            | Self::Let { source, .. }
            | Self::Return { source, .. }
            | Self::Expr { source, .. }
            | Self::Loop { source, .. }
            | Self::If { source, .. }
            | Self::Break { source, .. }
            | Self::Append { source, .. }
            | Self::AssignRef { source, .. }
            | Self::Yield { source, .. }
            | Self::Noop { source } => source,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirExpr {
    Null {
        source: Expr,
    },
    Bool {
        source: Expr,
        value: bool,
    },
    String {
        source: Expr,
        value: String,
    },
    Number {
        source: Expr,
        value: String,
    },
    Variable {
        source: Expr,
        name: String,
    },
    FunctionCall {
        source: Expr,
        call: MirFunctionCall,
    },
    MethodCall {
        source: Expr,
        object: Box<MirExpr>,
        method: String,
        args: Vec<MirExpr>,
    },
    StaticCall {
        source: Expr,
        class_name: echo_ast::QualifiedName,
        method: String,
        args: Vec<MirExpr>,
    },
    Assign {
        source: Expr,
        name: String,
        value: Box<MirExpr>,
    },
    MagicDir {
        source: Expr,
    },
    Require {
        source: Expr,
        once: bool,
        path: Box<MirExpr>,
    },
    Defer {
        source: Expr,
        body: Vec<MirStmt>,
    },
    Run {
        source: Expr,
        expr: MirRunExpr,
    },
    Fork {
        source: Expr,
        expr: MirForkExpr,
    },
    Spawn {
        source: Expr,
        command: Box<MirExpr>,
    },
    Join {
        source: Expr,
        handle: Box<MirExpr>,
    },
    Loop {
        source: Expr,
        body: Vec<MirStmt>,
    },
    Unary {
        source: Expr,
        op: UnaryOp,
        expr: Box<MirExpr>,
    },
    Binary {
        source: Expr,
        left: Box<MirExpr>,
        op: BinaryOp,
        right: Box<MirExpr>,
    },
    Field {
        source: Expr,
        object: Box<MirExpr>,
        field: String,
    },
    Index {
        source: Expr,
        collection: Box<MirExpr>,
        index: Box<MirExpr>,
    },
    Object {
        source: Expr,
        name: String,
        fields: Vec<MirObjectField>,
    },
    List {
        source: Expr,
        values: Vec<MirExpr>,
    },
    Array {
        source: Expr,
        elements: Vec<MirArrayElement>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirObjectField {
    pub name: String,
    pub value: MirExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirArrayElement {
    pub key: Option<MirExpr>,
    pub value: MirExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirFunctionCall {
    pub name: String,
    pub args: Vec<MirExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirRunExpr {
    Block { body: Vec<MirStmt> },
    Task { expr: Box<MirExpr> },
    Group { entries: Vec<Vec<MirStmt>> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirForkExpr {
    Block { body: Vec<MirStmt> },
    Task { expr: Box<MirExpr> },
}

impl MirExpr {
    pub fn syntax(&self) -> &Expr {
        match self {
            Self::Null { source }
            | Self::Bool { source, .. }
            | Self::String { source, .. }
            | Self::Number { source, .. }
            | Self::Variable { source, .. }
            | Self::FunctionCall { source, .. }
            | Self::MethodCall { source, .. }
            | Self::StaticCall { source, .. }
            | Self::Assign { source, .. }
            | Self::MagicDir { source }
            | Self::Require { source, .. }
            | Self::Defer { source, .. }
            | Self::Run { source, .. }
            | Self::Fork { source, .. }
            | Self::Spawn { source, .. }
            | Self::Join { source, .. }
            | Self::Loop { source, .. }
            | Self::Unary { source, .. }
            | Self::Binary { source, .. }
            | Self::Field { source, .. }
            | Self::Index { source, .. }
            | Self::Object { source, .. }
            | Self::List { source, .. }
            | Self::Array { source, .. } => source,
        }
    }

    pub fn to_syntax(&self) -> Expr {
        match self {
            Self::Null { source }
            | Self::Bool { source, .. }
            | Self::String { source, .. }
            | Self::Number { source, .. }
            | Self::Variable { source, .. }
            | Self::FunctionCall { source, .. }
            | Self::MethodCall { source, .. }
            | Self::StaticCall { source, .. }
            | Self::Assign { source, .. }
            | Self::MagicDir { source }
            | Self::Require { source, .. }
            | Self::Defer { source, .. }
            | Self::Run { source, .. }
            | Self::Fork { source, .. }
            | Self::Spawn { source, .. }
            | Self::Join { source, .. }
            | Self::Loop { source, .. }
            | Self::Unary { source, .. }
            | Self::Binary { source, .. }
            | Self::Field { source, .. }
            | Self::Index { source, .. }
            | Self::Object { source, .. }
            | Self::List { source, .. }
            | Self::Array { source, .. } => source.clone(),
        }
    }
}

pub fn lower_program(program: &HirProgram) -> Result<MirProgram, Vec<Diagnostic>> {
    let mut imports = Vec::new();
    let mut functions = Vec::new();
    let statements = program
        .statements()
        .iter()
        .map(|statement| lower_statement(statement, &mut imports, &mut functions))
        .collect::<Vec<_>>();

    Ok(MirProgram {
        source_dir: program.source().source_dir.clone(),
        imports,
        functions,
        statements,
    })
}

fn lower_statement(
    statement: &HirStmt,
    imports: &mut Vec<ImportStmt>,
    functions: &mut Vec<MirFunction>,
) -> MirStmt {
    match statement {
        HirStmt::Syntax(statement) => lower_syntax_statement(statement, imports, functions),
    }
}

fn lower_syntax_statement(
    statement: &Stmt,
    imports: &mut Vec<ImportStmt>,
    functions: &mut Vec<MirFunction>,
) -> MirStmt {
    match statement {
        Stmt::Import(statement) => imports.push(statement.clone()),
        Stmt::FunctionDecl(statement) if !statement.is_intrinsic && !statement.is_generator => {
            functions.push(lower_function(statement));
        }
        _ => {}
    }

    match statement {
        Stmt::Echo(statement) => MirStmt::Echo {
            source: Stmt::Echo(statement.clone()),
            exprs: statement.exprs.iter().map(lower_expr).collect(),
        },
        Stmt::FunctionCall(statement) => MirStmt::FunctionCall {
            source: Stmt::FunctionCall(statement.clone()),
            call: MirFunctionCall {
                name: statement.name.clone(),
                args: statement.args.iter().map(lower_expr).collect(),
                span: statement.span,
            },
        },
        Stmt::DynamicFunctionCall(statement) => MirStmt::DynamicFunctionCall {
            source: Stmt::DynamicFunctionCall(statement.clone()),
            name: statement.name.clone(),
            args: statement.args.iter().map(lower_expr).collect(),
        },
        Stmt::Assign(statement) => MirStmt::Assign {
            source: Stmt::Assign(statement.clone()),
            name: statement.name.clone(),
            value: lower_expr(&statement.value),
        },
        Stmt::Let(statement) => MirStmt::Let {
            source: Stmt::Let(statement.clone()),
            name: statement.name.clone(),
            value: lower_expr(&statement.value),
        },
        Stmt::Return(statement) => MirStmt::Return {
            source: Stmt::Return(statement.clone()),
            value: lower_expr(&statement.value),
        },
        Stmt::Expr(statement) => MirStmt::Expr {
            source: Stmt::Expr(statement.clone()),
            expr: lower_expr(&statement.expr),
        },
        Stmt::Loop(statement) => MirStmt::Loop {
            source: Stmt::Loop(statement.clone()),
            body: statement
                .body
                .iter()
                .map(|statement| lower_syntax_statement(statement, imports, functions))
                .collect(),
        },
        Stmt::If(statement) => MirStmt::If {
            source: Stmt::If(statement.clone()),
            condition: lower_expr(&statement.condition),
            body: statement
                .body
                .iter()
                .map(|statement| lower_syntax_statement(statement, imports, functions))
                .collect(),
        },
        Stmt::Break(statement) => MirStmt::Break {
            source: Stmt::Break(statement.clone()),
            value: statement.value.as_ref().map(lower_expr),
        },
        Stmt::Append(statement) => MirStmt::Append {
            source: Stmt::Append(statement.clone()),
            target: statement.target.clone(),
            value: lower_expr(&statement.value),
        },
        Stmt::AssignRef(statement) => MirStmt::AssignRef {
            source: Stmt::AssignRef(statement.clone()),
            name: statement.name.clone(),
            target: statement.target.clone(),
        },
        Stmt::Yield(statement) => MirStmt::Yield {
            source: Stmt::Yield(statement.clone()),
            value: lower_expr(&statement.value),
        },
        Stmt::FunctionDecl(_)
        | Stmt::Namespace(_)
        | Stmt::Use(_)
        | Stmt::Import(_)
        | Stmt::ClassDecl(_)
        | Stmt::TypeDecl(_) => MirStmt::Noop {
            source: statement.clone(),
        },
    }
}

fn lower_function(statement: &FunctionDeclStmt) -> MirFunction {
    MirFunction {
        source: statement.clone(),
        name: statement.name.clone(),
        params: statement.params.clone(),
        return_type: statement.return_type.clone(),
        body: statement
            .body
            .iter()
            .map(lower_function_body_statement)
            .collect(),
    }
}

fn lower_function_body_statement(statement: &Stmt) -> MirStmt {
    let mut imports = Vec::new();
    let mut functions = Vec::new();
    lower_syntax_statement(statement, &mut imports, &mut functions)
}

fn lower_expr(expr: &Expr) -> MirExpr {
    match expr {
        Expr::Null(_) => MirExpr::Null {
            source: expr.clone(),
        },
        Expr::Bool(value) => MirExpr::Bool {
            source: expr.clone(),
            value: value.value,
        },
        Expr::String(value) => MirExpr::String {
            source: expr.clone(),
            value: value.value.clone(),
        },
        Expr::Number(value) => MirExpr::Number {
            source: expr.clone(),
            value: value.value.clone(),
        },
        Expr::Variable(value) => MirExpr::Variable {
            source: expr.clone(),
            name: value.name.clone(),
        },
        Expr::FunctionCall(value) => MirExpr::FunctionCall {
            source: expr.clone(),
            call: MirFunctionCall {
                name: value.name.clone(),
                args: value.args.iter().map(lower_expr).collect(),
                span: value.span,
            },
        },
        Expr::MethodCall(value) => MirExpr::MethodCall {
            source: expr.clone(),
            object: Box::new(lower_expr(&value.object)),
            method: value.method.clone(),
            args: value.args.iter().map(lower_expr).collect(),
        },
        Expr::StaticCall(value) => MirExpr::StaticCall {
            source: expr.clone(),
            class_name: value.class_name.clone(),
            method: value.method.clone(),
            args: value.args.iter().map(lower_expr).collect(),
        },
        Expr::Assign(value) => MirExpr::Assign {
            source: expr.clone(),
            name: value.name.clone(),
            value: Box::new(lower_expr(&value.value)),
        },
        Expr::MagicConstant(value) => match value.kind {
            echo_ast::MagicConstantKind::Dir => MirExpr::MagicDir {
                source: expr.clone(),
            },
        },
        Expr::Require(value) => MirExpr::Require {
            source: expr.clone(),
            once: value.kind == echo_ast::RequireKind::RequireOnce,
            path: Box::new(lower_expr(&value.path)),
        },
        Expr::Defer(value) => MirExpr::Defer {
            source: expr.clone(),
            body: value
                .body
                .iter()
                .map(lower_function_body_statement)
                .collect(),
        },
        Expr::Run(value) => MirExpr::Run {
            source: expr.clone(),
            expr: match value {
                echo_ast::RunExpr::Block { body, .. } => MirRunExpr::Block {
                    body: body.iter().map(lower_function_body_statement).collect(),
                },
                echo_ast::RunExpr::Task { expr, .. } => MirRunExpr::Task {
                    expr: Box::new(lower_expr(expr)),
                },
                echo_ast::RunExpr::Group { entries, .. } => MirRunExpr::Group {
                    entries: entries
                        .iter()
                        .map(|entry| entry.iter().map(lower_function_body_statement).collect())
                        .collect(),
                },
            },
        },
        Expr::Fork(value) => MirExpr::Fork {
            source: expr.clone(),
            expr: match value {
                echo_ast::ForkExpr::Block { body, .. } => MirForkExpr::Block {
                    body: body.iter().map(lower_function_body_statement).collect(),
                },
                echo_ast::ForkExpr::Task { expr, .. } => MirForkExpr::Task {
                    expr: Box::new(lower_expr(expr)),
                },
            },
        },
        Expr::Spawn(value) => MirExpr::Spawn {
            source: expr.clone(),
            command: Box::new(lower_expr(&value.command)),
        },
        Expr::Join(value) => MirExpr::Join {
            source: expr.clone(),
            handle: Box::new(lower_expr(&value.handle)),
        },
        Expr::Loop(value) => MirExpr::Loop {
            source: expr.clone(),
            body: value
                .body
                .iter()
                .map(lower_function_body_statement)
                .collect(),
        },
        Expr::Unary(value) => MirExpr::Unary {
            source: expr.clone(),
            op: value.op,
            expr: Box::new(lower_expr(&value.expr)),
        },
        Expr::Binary(value) => MirExpr::Binary {
            source: expr.clone(),
            left: Box::new(lower_expr(&value.left)),
            op: value.op,
            right: Box::new(lower_expr(&value.right)),
        },
        Expr::Field(value) => MirExpr::Field {
            source: expr.clone(),
            object: Box::new(lower_expr(&value.object)),
            field: value.field.clone(),
        },
        Expr::Index(value) => MirExpr::Index {
            source: expr.clone(),
            collection: Box::new(lower_expr(&value.collection)),
            index: Box::new(lower_expr(&value.index)),
        },
        Expr::Object(value) => MirExpr::Object {
            source: expr.clone(),
            name: value.name.clone(),
            fields: value
                .fields
                .iter()
                .map(|field| MirObjectField {
                    name: field.name.clone(),
                    value: lower_expr(&field.value),
                })
                .collect(),
        },
        Expr::List(value) => MirExpr::List {
            source: expr.clone(),
            values: value.values.iter().map(lower_expr).collect(),
        },
        Expr::Array(value) => MirExpr::Array {
            source: expr.clone(),
            elements: value
                .elements
                .iter()
                .map(|element| MirArrayElement {
                    key: element.key.as_ref().map(lower_expr),
                    value: lower_expr(&element.value),
                })
                .collect(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use echo_ast::{
        AppendStmt, ArrayElement, ArrayExpr, AssignRefStmt, BinaryExpr, BinaryOp, BreakStmt,
        DynamicFunctionCallStmt, EchoStmt, Expr, FieldExpr, FunctionDeclStmt, IfStmt, ImportSource,
        ImportStmt, LetStmt, ListExpr, LoopStmt, NamespaceSource, NamespaceStmt, NullLiteral,
        NumberLiteral, ObjectExpr, ObjectField, Program, QualifiedName, Stmt, StringLiteral,
        TypeDeclStmt, VariableExpr, YieldStmt,
    };
    use echo_source::Span;

    fn program(statements: Vec<Stmt>) -> Program {
        Program {
            open_tag: None,
            statements,
            source_dir: None,
            span: Span::new(0, 0),
        }
    }

    #[test]
    fn lower_program_preserves_hir_syntax_for_codegen() {
        let program = program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::String(StringLiteral {
                value: "hello".to_string(),
                span: Span::new(5, 12),
            })],
            span: Span::new(0, 13),
        })]);
        let hir = echo_hir::lower_program(&program).expect("program should lower to HIR");
        let mir = lower_program(&hir).expect("HIR should lower to MIR");

        assert_eq!(mir.statements().len(), program.statements.len());
        assert_eq!(mir.statements()[0].syntax(), &program.statements[0]);
        assert!(matches!(mir.statements()[0], MirStmt::Echo { .. }));
        assert!(mir.imports().is_empty());
        assert!(mir.functions().is_empty());
    }

    #[test]
    fn lower_program_extracts_import_and_user_function_sections() {
        let program = program(vec![
            Stmt::Import(ImportStmt {
                source: ImportSource::Std,
                name: QualifiedName::new(vec!["net".to_string()]),
                alias: None,
                span: Span::new(0, 15),
            }),
            Stmt::FunctionDecl(FunctionDeclStmt {
                name: "greet".to_string(),
                params: Vec::new(),
                return_type: None,
                is_intrinsic: false,
                is_generator: false,
                body: vec![Stmt::Return(echo_ast::ReturnStmt {
                    value: Expr::String(StringLiteral {
                        value: "hi".to_string(),
                        span: Span::new(26, 30),
                    }),
                    span: Span::new(19, 31),
                })],
                span: Span::new(16, 29),
            }),
        ]);
        let hir = echo_hir::lower_program(&program).expect("program should lower to HIR");
        let mir = lower_program(&hir).expect("HIR should lower to MIR");

        assert_eq!(mir.imports().len(), 1);
        assert_eq!(mir.imports()[0].name.as_string(), "net");
        assert_eq!(mir.functions().len(), 1);
        assert_eq!(mir.functions()[0].name, "greet");
        assert!(matches!(
            mir.functions()[0].body.first(),
            Some(MirStmt::Return { .. })
        ));
        assert_eq!(mir.statements().len(), 2);
    }

    #[test]
    fn lower_program_lowers_common_expression_shapes() {
        let program = program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::Binary(Box::new(BinaryExpr {
                left: Expr::Number(NumberLiteral {
                    value: "2".to_string(),
                    span: Span::new(5, 6),
                }),
                op: BinaryOp::Add,
                right: Expr::Number(NumberLiteral {
                    value: "3".to_string(),
                    span: Span::new(9, 10),
                }),
                span: Span::new(5, 10),
            }))],
            span: Span::new(0, 11),
        })]);
        let hir = echo_hir::lower_program(&program).expect("program should lower to HIR");
        let mir = lower_program(&hir).expect("HIR should lower to MIR");

        let MirStmt::Echo { exprs, .. } = &mir.statements()[0] else {
            panic!("echo statement should lower to MIR echo");
        };
        assert!(matches!(
            &exprs[0],
            MirExpr::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
    }

    #[test]
    fn lower_program_lowers_collection_and_access_expression_shapes() {
        let program = program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![
                Expr::List(ListExpr {
                    values: vec![Expr::String(StringLiteral {
                        value: "a".to_string(),
                        span: Span::new(1, 4),
                    })],
                    span: Span::new(0, 5),
                }),
                Expr::Array(ArrayExpr {
                    elements: vec![ArrayElement {
                        key: Some(Expr::String(StringLiteral {
                            value: "k".to_string(),
                            span: Span::new(7, 10),
                        })),
                        value: Expr::Number(NumberLiteral {
                            value: "1".to_string(),
                            span: Span::new(14, 15),
                        }),
                        span: Span::new(7, 15),
                    }],
                    span: Span::new(6, 16),
                }),
                Expr::Field(Box::new(FieldExpr {
                    object: Expr::Object(ObjectExpr {
                        name: "object".to_string(),
                        fields: vec![ObjectField {
                            name: "name".to_string(),
                            value: Expr::String(StringLiteral {
                                value: "Echo".to_string(),
                                span: Span::new(26, 32),
                            }),
                        }],
                        span: Span::new(18, 33),
                    }),
                    field: "name".to_string(),
                    span: Span::new(18, 38),
                })),
            ],
            span: Span::new(0, 39),
        })]);
        let hir = echo_hir::lower_program(&program).expect("program should lower to HIR");
        let mir = lower_program(&hir).expect("HIR should lower to MIR");

        let MirStmt::Echo { exprs, .. } = &mir.statements()[0] else {
            panic!("echo statement should lower to MIR echo");
        };

        assert!(matches!(&exprs[0], MirExpr::List { .. }));
        assert!(matches!(&exprs[1], MirExpr::Array { .. }));
        assert!(matches!(
            &exprs[2],
            MirExpr::Field {
                object,
                field,
                ..
            } if field == "name" && matches!(object.as_ref(), MirExpr::Object { .. })
        ));
    }

    #[test]
    fn lower_program_lowers_control_statement_shapes() {
        let program = program(vec![
            Stmt::Let(LetStmt {
                name: "items".to_string(),
                ty: None,
                value: Expr::Array(ArrayExpr {
                    elements: Vec::new(),
                    span: Span::new(12, 14),
                }),
                span: Span::new(0, 15),
            }),
            Stmt::Let(LetStmt {
                name: "item".to_string(),
                ty: None,
                value: Expr::Null(NullLiteral {
                    span: Span::new(27, 31),
                }),
                span: Span::new(17, 32),
            }),
            Stmt::Loop(LoopStmt {
                body: vec![
                    Stmt::Append(AppendStmt {
                        target: "items".to_string(),
                        value: Expr::Variable(VariableExpr {
                            name: "item".to_string(),
                            span: Span::new(42, 47),
                        }),
                        span: Span::new(34, 48),
                    }),
                    Stmt::If(IfStmt {
                        condition: Expr::Binary(Box::new(BinaryExpr {
                            left: Expr::Variable(VariableExpr {
                                name: "item".to_string(),
                                span: Span::new(53, 58),
                            }),
                            op: BinaryOp::Is,
                            right: Expr::Null(NullLiteral {
                                span: Span::new(62, 66),
                            }),
                            span: Span::new(53, 66),
                        })),
                        body: vec![Stmt::Break(BreakStmt {
                            value: None,
                            span: Span::new(69, 75),
                        })],
                        span: Span::new(50, 77),
                    }),
                ],
                span: Span::new(34, 78),
            }),
        ]);
        let hir = echo_hir::lower_program(&program).expect("program should lower to HIR");
        let mir = lower_program(&hir).expect("HIR should lower to MIR");

        let MirStmt::Loop { body, .. } = &mir.statements()[2] else {
            panic!("loop statement should lower to MIR loop");
        };

        assert!(matches!(&body[0], MirStmt::Append { target, .. } if target == "items"));
        assert!(matches!(
            &body[1],
            MirStmt::If {
                condition: MirExpr::Binary {
                    op: BinaryOp::Is,
                    ..
                },
                body,
                ..
            } if matches!(body.first(), Some(MirStmt::Break { value: None, .. }))
        ));
    }

    #[test]
    fn lower_syntax_statement_lowers_remaining_statement_shapes() {
        let mut imports = Vec::new();
        let mut functions = Vec::new();

        assert!(matches!(
            lower_syntax_statement(
                &Stmt::DynamicFunctionCall(DynamicFunctionCallStmt {
                    name: "handler".to_string(),
                    args: Vec::new(),
                    span: Span::new(0, 10),
                }),
                &mut imports,
                &mut functions,
            ),
            MirStmt::DynamicFunctionCall { name, .. } if name == "handler"
        ));

        assert!(matches!(
            lower_syntax_statement(
                &Stmt::AssignRef(AssignRefStmt {
                    name: "alias".to_string(),
                    target: "target".to_string(),
                    span: Span::new(11, 22),
                }),
                &mut imports,
                &mut functions,
            ),
            MirStmt::AssignRef { name, target, .. } if name == "alias" && target == "target"
        ));

        assert!(matches!(
            lower_syntax_statement(
                &Stmt::Yield(YieldStmt {
                    value: Expr::Null(NullLiteral {
                        span: Span::new(29, 33),
                    }),
                    span: Span::new(23, 34),
                }),
                &mut imports,
                &mut functions,
            ),
            MirStmt::Yield { .. }
        ));

        assert!(matches!(
            lower_syntax_statement(
                &Stmt::Namespace(NamespaceStmt {
                    source: NamespaceSource::Php,
                    name: QualifiedName::new(vec!["App".to_string()]),
                    span: Span::new(35, 49),
                }),
                &mut imports,
                &mut functions,
            ),
            MirStmt::Noop { .. }
        ));

        assert!(matches!(
            lower_syntax_statement(
                &Stmt::TypeDecl(TypeDeclStmt {
                    name: "User".to_string(),
                    fields: Vec::new(),
                    span: Span::new(50, 62),
                }),
                &mut imports,
                &mut functions,
            ),
            MirStmt::Noop { .. }
        ));
    }
}
