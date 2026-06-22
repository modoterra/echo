use crate::{
    MirArrayElement, MirExpr, MirForkExpr, MirFunction, MirFunctionCall, MirObjectField,
    MirProgram, MirRunExpr, MirStmt,
};
use echo_ast::{Expr, FunctionDeclStmt, ImportStmt, Stmt};
use echo_diagnostics::Diagnostic;
use echo_hir::{HirProgram, HirStmt};

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

pub(crate) fn lower_syntax_statement(
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
        Expr::TypeAscription(value) => lower_expr(&value.expr),
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
