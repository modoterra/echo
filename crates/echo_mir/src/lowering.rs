use crate::{
    MirArrayElement, MirCallArg, MirCatchClause, MirElseIfClause, MirExpr, MirForkExpr,
    MirFunction, MirFunctionCall, MirNewTarget, MirObjectField, MirProgram, MirRunExpr, MirStmt,
};
use echo_ast::{CallArg, ClassMember, Expr, FunctionDeclStmt, ImportStmt, MethodDecl, Stmt};
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
        Stmt::ClassDecl(statement) => {
            for member in &statement.members {
                if let ClassMember::Method(method) = member {
                    functions.push(lower_method(&statement.name, method));
                }
            }
        }
        Stmt::TraitDecl(statement) => {
            for member in &statement.members {
                if let ClassMember::Method(method) = member {
                    functions.push(lower_method(&statement.name, method));
                }
            }
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
                args: statement.args.iter().map(lower_call_arg).collect(),
                span: statement.span,
            },
        },
        Stmt::DynamicFunctionCall(statement) => MirStmt::DynamicFunctionCall {
            source: Stmt::DynamicFunctionCall(statement.clone()),
            name: statement.name.clone(),
            args: statement.args.iter().map(lower_call_arg).collect(),
        },
        Stmt::Assign(statement) => MirStmt::Assign {
            source: Stmt::Assign(statement.clone()),
            name: statement.name.clone(),
            value: lower_expr(&statement.value),
        },
        Stmt::CoalesceAssign(statement) => MirStmt::CoalesceAssign {
            source: Stmt::CoalesceAssign(statement.clone()),
            name: statement.name.clone(),
            value: lower_expr(&statement.value),
        },
        Stmt::ListAssign(statement) => MirStmt::ListAssign {
            source: Stmt::ListAssign(statement.clone()),
            targets: statement.targets.clone(),
            value: lower_expr(&statement.value),
        },
        Stmt::Let(statement) => MirStmt::Let {
            source: Stmt::Let(statement.clone()),
            name: statement.name.clone(),
            value: lower_expr(&statement.value),
        },
        Stmt::Return(statement) => MirStmt::Return {
            source: Stmt::Return(statement.clone()),
            value: statement.value.as_ref().map(lower_expr),
        },
        Stmt::Throw(statement) => MirStmt::Throw {
            source: Stmt::Throw(statement.clone()),
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
        Stmt::While(statement) => MirStmt::While {
            source: Stmt::While(statement.clone()),
            condition: lower_expr(&statement.condition),
            body: statement
                .body
                .iter()
                .map(|statement| lower_syntax_statement(statement, imports, functions))
                .collect(),
        },
        Stmt::Foreach(statement) => MirStmt::Foreach {
            source: Stmt::Foreach(statement.clone()),
            iterable: lower_expr(&statement.iterable),
            key: statement.key.clone(),
            value: statement.value.clone(),
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
            elseif_clauses: statement
                .elseif_clauses
                .iter()
                .map(|clause| MirElseIfClause {
                    condition: lower_expr(&clause.condition),
                    body: clause
                        .body
                        .iter()
                        .map(|statement| lower_syntax_statement(statement, imports, functions))
                        .collect(),
                    span: clause.span,
                })
                .collect(),
            else_body: statement
                .else_body
                .iter()
                .map(|statement| lower_syntax_statement(statement, imports, functions))
                .collect(),
        },
        Stmt::Try(statement) => MirStmt::Try {
            source: Stmt::Try(statement.clone()),
            body: statement
                .body
                .iter()
                .map(|statement| lower_syntax_statement(statement, imports, functions))
                .collect(),
            catches: statement
                .catches
                .iter()
                .map(|catch| MirCatchClause {
                    types: catch.types.clone(),
                    variable: catch.variable.clone(),
                    body: catch
                        .body
                        .iter()
                        .map(|statement| lower_syntax_statement(statement, imports, functions))
                        .collect(),
                    span: catch.span,
                })
                .collect(),
            finally_body: statement
                .finally_body
                .iter()
                .map(|statement| lower_syntax_statement(statement, imports, functions))
                .collect(),
        },
        Stmt::Break(statement) => MirStmt::Break {
            source: Stmt::Break(statement.clone()),
            value: statement.value.as_ref().map(lower_expr),
        },
        Stmt::Continue(statement) => MirStmt::Continue {
            source: Stmt::Continue(statement.clone()),
            value: statement.value.as_ref().map(lower_expr),
        },
        Stmt::Append(statement) => MirStmt::Append {
            source: Stmt::Append(statement.clone()),
            target: lower_expr(&statement.target),
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
        | Stmt::UnnamedExport(_)
        | Stmt::ClassDecl(_)
        | Stmt::TraitDecl(_)
        | Stmt::ExtendDecl(_)
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

fn lower_method(class_name: &str, method: &MethodDecl) -> MirFunction {
    let source = FunctionDeclStmt {
        name: format!("{class_name}::{}", method.name),
        params: method.params.clone(),
        return_type: method.return_type.clone(),
        is_intrinsic: method.is_intrinsic,
        is_generator: false,
        body: method.body.clone(),
        span: method.span,
    };

    MirFunction {
        source,
        name: format!("{class_name}::{}", method.name),
        params: method.params.clone(),
        return_type: method.return_type.clone(),
        body: method
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

pub fn lower_expr(expr: &Expr) -> MirExpr {
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
        Expr::Constant(value) => MirExpr::Constant {
            source: expr.clone(),
            name: value.name.clone(),
        },
        Expr::ReceiverConst(value) => MirExpr::ReceiverConst {
            source: expr.clone(),
            kind: value.kind,
        },
        Expr::StaticPropertyFetch(value) => MirExpr::StaticPropertyFetch {
            source: expr.clone(),
            class_name: value.class_name.clone(),
            property: value.property.clone(),
        },
        Expr::StaticPropertyAssign(value) => MirExpr::StaticPropertyAssign {
            source: expr.clone(),
            class_name: value.class_name.clone(),
            property: value.property.clone(),
            value: Box::new(lower_expr(&value.value)),
        },
        Expr::StaticPropertyCoalesceAssign(value) => MirExpr::StaticPropertyAssign {
            source: expr.clone(),
            class_name: value.class_name.clone(),
            property: value.property.clone(),
            value: Box::new(lower_expr(&value.value)),
        },
        Expr::ClassConstantFetch(value) => MirExpr::ClassConstantFetch {
            source: expr.clone(),
            class_name: value.class_name.clone(),
            constant: value.constant.clone(),
        },
        Expr::FunctionCall(value) => MirExpr::FunctionCall {
            source: expr.clone(),
            call: MirFunctionCall {
                name: value.name.clone(),
                args: value.args.iter().map(lower_call_arg).collect(),
                span: value.span,
            },
        },
        Expr::DynamicFunctionCall(value) => MirExpr::DynamicFunctionCall {
            source: expr.clone(),
            name: value.name.clone(),
            args: value.args.iter().map(lower_call_arg).collect(),
        },
        Expr::DynamicCall(value) => MirExpr::DynamicCall {
            source: expr.clone(),
            callee: Box::new(lower_expr(&value.callee)),
            args: value.args.iter().map(lower_call_arg).collect(),
        },
        Expr::MethodCall(value) => MirExpr::MethodCall {
            source: expr.clone(),
            object: Box::new(lower_expr(&value.object)),
            method: value.method.clone(),
            method_span: value.method_span,
            args: value.args.iter().map(lower_call_arg).collect(),
        },
        Expr::StaticCall(value) => MirExpr::StaticCall {
            source: expr.clone(),
            class_name: value.class_name.clone(),
            method: value.method.clone(),
            args: value.args.iter().map(lower_call_arg).collect(),
        },
        Expr::New(value) => MirExpr::New {
            source: expr.clone(),
            target: match &value.target {
                echo_ast::NewTarget::Class(class_name) => MirNewTarget::Class(class_name.clone()),
                echo_ast::NewTarget::Expr(target) => {
                    MirNewTarget::Expr(Box::new(lower_expr(target)))
                }
            },
            args: value.args.iter().map(lower_call_arg).collect(),
        },
        Expr::Closure(value) => MirExpr::Closure {
            source: expr.clone(),
            params: value.params.clone(),
            captures: value.captures.clone(),
            return_type: value.return_type.clone(),
            body: value
                .body
                .iter()
                .map(lower_function_body_statement)
                .collect(),
        },
        Expr::ArrowFunction(value) => MirExpr::ArrowFunction {
            source: expr.clone(),
            params: value.params.clone(),
            return_type: value.return_type.clone(),
            body: Box::new(lower_expr(&value.body)),
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
        Expr::Include(value) => MirExpr::Include {
            source: expr.clone(),
            kind: value.kind.clone(),
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
        Expr::Cast(value) => MirExpr::Cast {
            source: expr.clone(),
            ty: value.ty.clone(),
            expr: Box::new(lower_expr(&value.expr)),
        },
        Expr::Binary(value) => MirExpr::Binary {
            source: expr.clone(),
            left: Box::new(lower_expr(&value.left)),
            op: value.op,
            right: Box::new(lower_expr(&value.right)),
        },
        Expr::Ternary(value) => MirExpr::Ternary {
            source: expr.clone(),
            condition: Box::new(lower_expr(&value.condition)),
            if_true: Box::new(lower_expr(&value.if_true)),
            if_false: Box::new(lower_expr(&value.if_false)),
        },
        Expr::Match(value) => value
            .arms
            .first()
            .map(|arm| lower_expr(&arm.value))
            .unwrap_or_else(|| MirExpr::Null {
                source: expr.clone(),
            }),
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
        Expr::TargetAssign(value) => MirExpr::TargetAssign {
            source: expr.clone(),
            target: Box::new(lower_expr(&value.target)),
            value: Box::new(lower_expr(&value.value)),
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

fn lower_call_arg(arg: &CallArg) -> MirCallArg {
    MirCallArg {
        name: arg.name.clone(),
        value: lower_expr(&arg.value),
        span: arg.span,
    }
}
