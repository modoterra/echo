use chumsky::input::MapExtra;
use chumsky::prelude::*;
use chumsky::span::SimpleSpan;
use echo_ast::{
    AnonymousClassExpr, AppendStmt, ArrayElement, ArrayExpr, ArrowFunctionExpr, AssignExpr,
    AssignRefStmt, AssignStmt, AttributeDecl, BinaryExpr, BinaryOp, BoolLiteral, BreakStmt,
    CallArg, CastExpr, CatchClause, ClassConstDecl, ClassConstantFetchExpr, ClassDeclStmt,
    ClassMember, ClassModifier, ClosureExpr, CoalesceAssignStmt, CompileEntry, CompileStmt,
    ConstantExpr, ContinueStmt, DeferExpr, DoWhileStmt, DynamicCallExpr, DynamicFunctionCallExpr,
    DynamicFunctionCallStmt, EchoStmt, ElseIfClause, EnumCaseDecl, EnumDeclStmt, EnumMember, Expr,
    FacetDeclStmt, FieldExpr, ForStmt, ForeachStmt, ForkExpr, FunctionCallExpr, FunctionCallStmt,
    FunctionDeclStmt, GlobalStmt, GotoStmt, IfStmt, ImportSource, ImportStmt, IncludeExpr,
    IncludeKind, IndexExpr, InterfaceDeclStmt, InterfaceMember, JoinExpr, LabelStmt, LetStmt,
    ListAssignStmt, ListExpr, LoopExpr, LoopStmt, MagicConstantExpr, MagicConstantKind, MatchArm,
    MatchExpr, MethodCallExpr, MethodDecl, MethodVisibility, NamespaceSource, NamespaceStmt,
    NewExpr, NewTarget, NullLiteral, NumberLiteral, ObjectExpr, ObjectField, PhpCloneWithExpr,
    PhpDeclareDirective, PhpDeclareStmt, PhpExitKind, PhpExitStmt, PhpInlineHtmlStmt, PrintExpr,
    Program, PropertyDecl, PropertyHookBody, PropertyHookDecl, PropertyHookKind, QualifiedName,
    ReceiverConst, ReceiverConstExpr, ReturnStmt, RunExpr, SpawnExpr, StaticCallExpr,
    StaticPropertyAssignExpr, StaticPropertyFetchExpr, StaticVarDecl, StaticVarStmt, Stmt,
    StringLiteral, SwitchCase, SwitchStmt, TargetAssignExpr, TernaryExpr, ThrowStmt, TraitDeclStmt,
    TryStmt, TypeAscriptionExpr, TypeDeclStmt, TypeField, TypedParam, UnaryExpr, UnaryOp,
    UnnamedExportStmt, UseStmt, VariableExpr, WhileStmt, YieldStmt,
};
use echo_diagnostics::Diagnostic;
use echo_source::{SourceFile, Span};
use echo_syntax::keywords as kw;

#[path = "preprocess.rs"]
mod preprocess;
#[path = "validation.rs"]
mod validation;

use preprocess::{
    normalize_bracketed_namespaces, normalize_heredoc_literals, strip_comments_preserving_spans,
    unescape_double_quoted_string, unescape_single_quoted_string, virtualize_statement_terminators,
};
use validation::validate_program;

type ParseExtra<'src> = extra::Err<Rich<'src, char>>;
type BoxedParser<'src, O> = chumsky::Boxed<'src, 'src, &'src str, O, ParseExtra<'src>>;

pub fn parse(source: &str) -> Result<Program, Vec<Diagnostic>> {
    parse_program(source)
}

pub fn parse_source_file(source: &SourceFile) -> Result<Program, Vec<Diagnostic>> {
    let mut program =
        parse_program(&source.text).map_err(|diagnostics| attach_source(source, diagnostics))?;
    program.source_id = source.id;
    Ok(program)
}

pub fn parse_trusted_std(source: &str) -> Result<Program, Vec<Diagnostic>> {
    parse_program(source)
}

pub fn parse_trusted_std_source(source: &SourceFile) -> Result<Program, Vec<Diagnostic>> {
    let mut program =
        parse_program(&source.text).map_err(|diagnostics| attach_source(source, diagnostics))?;
    program.source_id = source.id;
    Ok(program)
}

fn attach_source(source: &SourceFile, diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    let Some(source_id) = source.id else {
        return diagnostics;
    };

    diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.with_source_id(source_id))
        .collect()
}

fn parse_program(source: &str) -> Result<Program, Vec<Diagnostic>> {
    let (source, mut compile_statements) = extract_compile_declarations(source)?;
    // For now, run the Logos lexer first so lexer errors are caught.
    // The Chumsky parser below still parses the source text directly.
    echo_lexer::lex(&source)?;

    let source = normalize_heredoc_literals(&source);
    let source = strip_comments_preserving_spans(&source);
    let source = normalize_bracketed_namespaces(&source);
    let source = virtualize_statement_terminators(&source);

    let mut program = parser().parse(&source).into_result().map_err(|errors| {
        errors
            .into_iter()
            .map(|error| {
                let span = error.span();
                Diagnostic::new(error.to_string(), Span::new(span.start, span.end))
            })
            .collect::<Vec<_>>()
    })?;

    normalize_php_compat_receiver_variables(&mut program);
    if !compile_statements.is_empty() {
        program.statements.append(&mut compile_statements);
        program.statements.sort_by_key(|statement| {
            (
                statement_span(statement).start,
                statement_sort_rank(statement),
            )
        });
    }

    validate_program(&program)?;

    Ok(program)
}

fn extract_compile_declarations(source: &str) -> Result<(String, Vec<Stmt>), Vec<Diagnostic>> {
    let mut output = source.as_bytes().to_vec();
    let bytes = source.as_bytes();
    let mut statements = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        if !is_compile_keyword_at(bytes, index) {
            index += 1;
            continue;
        }

        let start = index;
        let mut cursor = index + kw::COMPILE.text.len();
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if bytes.get(cursor) != Some(&b'{') {
            index += 1;
            continue;
        }
        cursor += 1;

        let mut entries = Vec::new();
        loop {
            while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
                cursor += 1;
            }
            match bytes.get(cursor) {
                Some(b'}') => {
                    cursor += 1;
                    break;
                }
                Some(b'"') => {
                    let entry_start = cursor;
                    cursor += 1;
                    let value_start = cursor;
                    let mut escaped = false;
                    while cursor < bytes.len() {
                        let byte = bytes[cursor];
                        if escaped {
                            escaped = false;
                            cursor += 1;
                            continue;
                        }
                        if byte == b'\\' {
                            escaped = true;
                            cursor += 1;
                            continue;
                        }
                        if byte == b'"' {
                            let raw = &source[value_start..cursor];
                            cursor += 1;
                            entries.push(CompileEntry {
                                value: unescape_double_quoted_string(raw.to_string()),
                                span: Span::new(entry_start, cursor),
                            });
                            break;
                        }
                        cursor += 1;
                    }
                    if cursor > bytes.len() {
                        return Err(vec![Diagnostic::new(
                            "unterminated compile string literal",
                            Span::new(entry_start, bytes.len()),
                        )]);
                    }
                }
                Some(_) => {
                    return Err(vec![Diagnostic::new(
                        "compile declarations accept string literal entries only",
                        Span::new(cursor, cursor + 1),
                    )]);
                }
                None => {
                    return Err(vec![Diagnostic::new(
                        "unterminated compile declaration",
                        Span::new(start, bytes.len()),
                    )]);
                }
            }
        }

        for offset in start..cursor {
            if output[offset] != b'\n' && output[offset] != b'\r' {
                output[offset] = b' ';
            }
        }
        statements.push(Stmt::Compile(CompileStmt {
            entries,
            span: Span::new(start, cursor),
        }));
        index = cursor;
    }

    Ok((
        String::from_utf8(output).expect("compile extraction preserves UTF-8"),
        statements,
    ))
}

fn is_compile_keyword_at(source: &[u8], index: usize) -> bool {
    let keyword = kw::COMPILE.text.as_bytes();
    source.get(index..index + keyword.len()) == Some(keyword)
        && index
            .checked_sub(1)
            .and_then(|before| source.get(before))
            .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
        && source
            .get(index + keyword.len())
            .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
}

fn statement_sort_rank(statement: &Stmt) -> usize {
    match statement {
        Stmt::Compile(_) => 0,
        _ => 1,
    }
}

fn statement_span(statement: &Stmt) -> Span {
    match statement {
        Stmt::Compile(statement) => statement.span,
        Stmt::Echo(statement) => statement.span,
        Stmt::FunctionCall(statement) => statement.span,
        Stmt::DynamicFunctionCall(statement) => statement.span,
        Stmt::FunctionDecl(statement) => statement.span,
        Stmt::Assign(statement) => statement.span,
        Stmt::CoalesceAssign(statement) => statement.span,
        Stmt::ListAssign(statement) => statement.span,
        Stmt::Let(statement) => statement.span,
        Stmt::AssignRef(statement) => statement.span,
        Stmt::Return(statement) => statement.span,
        Stmt::Throw(statement) => statement.span,
        Stmt::Yield(statement) => statement.span,
        Stmt::Goto(statement) => statement.span,
        Stmt::Label(statement) => statement.span,
        Stmt::PhpDeclare(statement) => statement.span,
        Stmt::PhpExit(statement) => statement.span,
        Stmt::PhpInlineHtml(statement) => statement.span,
        Stmt::Global(statement) => statement.span,
        Stmt::StaticVar(statement) => statement.span,
        Stmt::Expr(statement) => statement.span,
        Stmt::Namespace(statement) => statement.span,
        Stmt::Use(statement) => statement.span,
        Stmt::Import(statement) => statement.span,
        Stmt::UnnamedExport(statement) => statement.span,
        Stmt::ClassDecl(statement) => statement.span,
        Stmt::InterfaceDecl(statement) => statement.span,
        Stmt::TraitDecl(statement) => statement.span,
        Stmt::EnumDecl(statement) => statement.span,
        Stmt::FacetDecl(statement) => statement.span,
        Stmt::TypeDecl(statement) => statement.span,
        Stmt::Loop(statement) => statement.span,
        Stmt::While(statement) => statement.span,
        Stmt::DoWhile(statement) => statement.span,
        Stmt::For(statement) => statement.span,
        Stmt::Foreach(statement) => statement.span,
        Stmt::Switch(statement) => statement.span,
        Stmt::If(statement) => statement.span,
        Stmt::Try(statement) => statement.span,
        Stmt::Break(statement) => statement.span,
        Stmt::Continue(statement) => statement.span,
        Stmt::Append(statement) => statement.span,
    }
}

fn normalize_php_compat_receiver_variables(program: &mut Program) {
    for statement in &mut program.statements {
        normalize_php_compat_statement(statement);
    }
}

fn normalize_php_compat_statement(statement: &mut Stmt) {
    match statement {
        Stmt::Echo(statement) => {
            for expr in &mut statement.exprs {
                normalize_php_compat_expr(expr);
            }
        }
        Stmt::FunctionCall(statement) => {
            for arg in &mut statement.args {
                normalize_php_compat_expr(&mut arg.value);
            }
        }
        Stmt::DynamicFunctionCall(statement) => {
            for arg in &mut statement.args {
                normalize_php_compat_expr(&mut arg.value);
            }
        }
        Stmt::FunctionDecl(statement) => {
            normalize_php_compat_attributes(&mut statement.attributes);
            for param in &mut statement.params {
                normalize_php_compat_attributes(&mut param.attributes);
                if let Some(value) = &mut param.default_value {
                    normalize_php_compat_expr(value);
                }
            }
            for statement in &mut statement.body {
                normalize_php_compat_statement(statement);
            }
        }
        Stmt::Assign(statement) => normalize_php_compat_expr(&mut statement.value),
        Stmt::CoalesceAssign(statement) => normalize_php_compat_expr(&mut statement.value),
        Stmt::ListAssign(statement) => normalize_php_compat_expr(&mut statement.value),
        Stmt::Let(statement) => normalize_php_compat_expr(&mut statement.value),
        Stmt::AssignRef(_) | Stmt::Break(_) | Stmt::Continue(_) | Stmt::Compile(_) => {}
        Stmt::Return(statement) => {
            if let Some(value) = &mut statement.value {
                normalize_php_compat_expr(value);
            }
        }
        Stmt::Throw(statement) => normalize_php_compat_expr(&mut statement.value),
        Stmt::Yield(statement) => normalize_php_compat_expr(&mut statement.value),
        Stmt::Goto(_) | Stmt::Label(_) => {}
        Stmt::PhpDeclare(statement) => {
            for directive in &mut statement.directives {
                normalize_php_compat_expr(&mut directive.value);
            }
            for statement in &mut statement.body {
                normalize_php_compat_statement(statement);
            }
        }
        Stmt::PhpExit(statement) => {
            if let Some(value) = &mut statement.value {
                normalize_php_compat_expr(value);
            }
        }
        Stmt::PhpInlineHtml(_) => {}
        Stmt::Global(_) => {}
        Stmt::StaticVar(statement) => {
            for var in &mut statement.vars {
                if let Some(value) = &mut var.value {
                    normalize_php_compat_expr(value);
                }
            }
        }
        Stmt::Expr(statement) => normalize_php_compat_expr(&mut statement.expr),
        Stmt::Loop(statement) => {
            for statement in &mut statement.body {
                normalize_php_compat_statement(statement);
            }
        }
        Stmt::While(statement) => {
            normalize_php_compat_expr(&mut statement.condition);
            for statement in &mut statement.body {
                normalize_php_compat_statement(statement);
            }
        }
        Stmt::DoWhile(statement) => {
            for statement in &mut statement.body {
                normalize_php_compat_statement(statement);
            }
            normalize_php_compat_expr(&mut statement.condition);
        }
        Stmt::For(statement) => {
            for expr in &mut statement.init {
                normalize_php_compat_expr(expr);
            }
            for expr in &mut statement.conditions {
                normalize_php_compat_expr(expr);
            }
            for expr in &mut statement.increments {
                normalize_php_compat_expr(expr);
            }
            for statement in &mut statement.body {
                normalize_php_compat_statement(statement);
            }
        }
        Stmt::Foreach(statement) => {
            normalize_php_compat_expr(&mut statement.iterable);
            for statement in &mut statement.body {
                normalize_php_compat_statement(statement);
            }
        }
        Stmt::Switch(statement) => {
            normalize_php_compat_expr(&mut statement.expr);
            for case in &mut statement.cases {
                if let Some(condition) = &mut case.condition {
                    normalize_php_compat_expr(condition);
                }
                for statement in &mut case.body {
                    normalize_php_compat_statement(statement);
                }
            }
        }
        Stmt::Try(statement) => {
            for statement in &mut statement.body {
                normalize_php_compat_statement(statement);
            }
            for catch in &mut statement.catches {
                for statement in &mut catch.body {
                    normalize_php_compat_statement(statement);
                }
            }
            for statement in &mut statement.finally_body {
                normalize_php_compat_statement(statement);
            }
        }
        Stmt::If(statement) => {
            normalize_php_compat_expr(&mut statement.condition);
            for statement in &mut statement.body {
                normalize_php_compat_statement(statement);
            }
            for clause in &mut statement.elseif_clauses {
                normalize_php_compat_expr(&mut clause.condition);
                for statement in &mut clause.body {
                    normalize_php_compat_statement(statement);
                }
            }
            for statement in &mut statement.else_body {
                normalize_php_compat_statement(statement);
            }
        }
        Stmt::Append(statement) => {
            normalize_php_compat_expr(&mut statement.target);
            normalize_php_compat_expr(&mut statement.value);
        }
        Stmt::UnnamedExport(statement) => normalize_php_compat_expr(&mut statement.value),
        Stmt::ClassDecl(statement) => {
            normalize_php_compat_attributes(&mut statement.attributes);
            for member in &mut statement.members {
                normalize_php_compat_class_member(member);
            }
        }
        Stmt::InterfaceDecl(statement) => {
            normalize_php_compat_attributes(&mut statement.attributes);
            for member in &mut statement.members {
                normalize_php_compat_interface_member(member);
            }
        }
        Stmt::TraitDecl(statement) => {
            normalize_php_compat_attributes(&mut statement.attributes);
            for member in &mut statement.members {
                normalize_php_compat_class_member(member);
            }
        }
        Stmt::EnumDecl(statement) => {
            normalize_php_compat_attributes(&mut statement.attributes);
            for member in &mut statement.members {
                normalize_php_compat_enum_member(member);
            }
        }
        Stmt::FacetDecl(statement) => {
            for member in &mut statement.members {
                normalize_php_compat_class_member(member);
            }
        }
        Stmt::Use(_) | Stmt::Import(_) | Stmt::Namespace(_) | Stmt::TypeDecl(_) => {}
    }
}

fn normalize_php_compat_class_member(member: &mut ClassMember) {
    match member {
        ClassMember::Method(method) => {
            normalize_php_compat_attributes(&mut method.attributes);
            for param in &mut method.params {
                normalize_php_compat_attributes(&mut param.attributes);
                if let Some(value) = &mut param.default_value {
                    normalize_php_compat_expr(value);
                }
            }
            for statement in &mut method.body {
                normalize_php_compat_statement(statement);
            }
        }
        ClassMember::Property(property) => {
            normalize_php_compat_attributes(&mut property.attributes);
            if let Some(value) = &mut property.value {
                normalize_php_compat_expr(value);
            }
            normalize_php_compat_property_hooks(&mut property.hooks);
        }
        ClassMember::Const(constant) => {
            normalize_php_compat_attributes(&mut constant.attributes);
            normalize_php_compat_expr(&mut constant.value);
        }
        ClassMember::TraitUse(_) => {}
    }
}

fn normalize_php_compat_interface_member(member: &mut InterfaceMember) {
    match member {
        InterfaceMember::Method(method) => {
            normalize_php_compat_attributes(&mut method.attributes);
            for param in &mut method.params {
                normalize_php_compat_attributes(&mut param.attributes);
                if let Some(value) = &mut param.default_value {
                    normalize_php_compat_expr(value);
                }
            }
            for statement in &mut method.body {
                normalize_php_compat_statement(statement);
            }
        }
        InterfaceMember::Property(property) => {
            normalize_php_compat_attributes(&mut property.attributes);
            if let Some(value) = &mut property.value {
                normalize_php_compat_expr(value);
            }
            normalize_php_compat_property_hooks(&mut property.hooks);
        }
        InterfaceMember::Const(constant) => {
            normalize_php_compat_attributes(&mut constant.attributes);
            normalize_php_compat_expr(&mut constant.value);
        }
    }
}

fn normalize_php_compat_attributes(attributes: &mut [AttributeDecl]) {
    for attribute in attributes {
        for arg in &mut attribute.args {
            normalize_php_compat_expr(&mut arg.value);
        }
    }
}

fn normalize_php_compat_property_hooks(hooks: &mut [PropertyHookDecl]) {
    for hook in hooks {
        if let Some(param) = &mut hook.param
            && let Some(value) = &mut param.default_value
        {
            normalize_php_compat_attributes(&mut param.attributes);
            normalize_php_compat_expr(value);
        } else if let Some(param) = &mut hook.param {
            normalize_php_compat_attributes(&mut param.attributes);
        }
        match &mut hook.body {
            PropertyHookBody::None => {}
            PropertyHookBody::Expr(expr) => normalize_php_compat_expr(expr),
            PropertyHookBody::Block(body) => {
                for statement in body {
                    normalize_php_compat_statement(statement);
                }
            }
        }
    }
}

fn normalize_php_compat_enum_member(member: &mut EnumMember) {
    match member {
        EnumMember::Case(case) => {
            normalize_php_compat_attributes(&mut case.attributes);
            if let Some(value) = &mut case.value {
                normalize_php_compat_expr(value);
            }
        }
        EnumMember::Method(method) => {
            normalize_php_compat_attributes(&mut method.attributes);
            for param in &mut method.params {
                normalize_php_compat_attributes(&mut param.attributes);
                if let Some(value) = &mut param.default_value {
                    normalize_php_compat_expr(value);
                }
            }
            for statement in &mut method.body {
                normalize_php_compat_statement(statement);
            }
        }
        EnumMember::TraitUse(_) => {}
    }
}

fn normalize_php_compat_expr(expr: &mut Expr) {
    match expr {
        Expr::ReceiverConst(receiver)
            if matches!(
                receiver.kind,
                ReceiverConst::SelfType | ReceiverConst::Parent | ReceiverConst::Static
            ) =>
        {
            *expr = Expr::Variable(VariableExpr {
                name: receiver.kind.variable_name().to_string(),
                span: receiver.span,
            });
        }
        Expr::ReceiverConst(_) => {}
        Expr::Defer(expr) => {
            for statement in &mut expr.body {
                normalize_php_compat_statement(statement);
            }
        }
        Expr::Run(RunExpr::Task { expr, .. }) | Expr::Fork(ForkExpr::Task { expr, .. }) => {
            normalize_php_compat_expr(expr);
        }
        Expr::Run(RunExpr::Block { body, .. })
        | Expr::Fork(ForkExpr::Block { body, .. })
        | Expr::Loop(LoopExpr { body, .. }) => {
            for statement in body {
                normalize_php_compat_statement(statement);
            }
        }
        Expr::Run(RunExpr::Group { entries, .. }) => {
            for entry in entries {
                for statement in entry {
                    normalize_php_compat_statement(statement);
                }
            }
        }
        Expr::Spawn(expr) => normalize_php_compat_expr(&mut expr.command),
        Expr::Join(expr) => normalize_php_compat_expr(&mut expr.handle),
        Expr::FunctionCall(expr) => {
            for arg in &mut expr.args {
                normalize_php_compat_expr(&mut arg.value);
            }
        }
        Expr::Print(expr) => normalize_php_compat_expr(&mut expr.value),
        Expr::DynamicFunctionCall(expr) => {
            for arg in &mut expr.args {
                normalize_php_compat_expr(&mut arg.value);
            }
        }
        Expr::DynamicCall(expr) => {
            normalize_php_compat_expr(&mut expr.callee);
            for arg in &mut expr.args {
                normalize_php_compat_expr(&mut arg.value);
            }
        }
        Expr::MethodCall(expr) => {
            normalize_php_compat_expr(&mut expr.object);
            for arg in &mut expr.args {
                normalize_php_compat_expr(&mut arg.value);
            }
        }
        Expr::StaticCall(expr) => {
            for arg in &mut expr.args {
                normalize_php_compat_expr(&mut arg.value);
            }
        }
        Expr::Closure(expr) => {
            for param in &mut expr.params {
                if let Some(value) = &mut param.default_value {
                    normalize_php_compat_expr(value);
                }
            }
            for statement in &mut expr.body {
                normalize_php_compat_statement(statement);
            }
        }
        Expr::ArrowFunction(expr) => {
            for param in &mut expr.params {
                if let Some(value) = &mut param.default_value {
                    normalize_php_compat_expr(value);
                }
            }
            normalize_php_compat_expr(&mut expr.body);
        }
        Expr::Assign(expr) => normalize_php_compat_expr(&mut expr.value),
        Expr::StaticPropertyAssign(expr) | Expr::StaticPropertyCoalesceAssign(expr) => {
            normalize_php_compat_expr(&mut expr.value);
        }
        Expr::Include(expr) => normalize_php_compat_expr(&mut expr.path),
        Expr::Binary(expr) => {
            normalize_php_compat_expr(&mut expr.left);
            normalize_php_compat_expr(&mut expr.right);
        }
        Expr::Ternary(expr) => {
            normalize_php_compat_expr(&mut expr.condition);
            normalize_php_compat_expr(&mut expr.if_true);
            normalize_php_compat_expr(&mut expr.if_false);
        }
        Expr::Match(expr) => {
            normalize_php_compat_expr(&mut expr.subject);
            for arm in &mut expr.arms {
                for condition in &mut arm.conditions {
                    normalize_php_compat_expr(condition);
                }
                normalize_php_compat_expr(&mut arm.value);
            }
        }
        Expr::Unary(expr) => normalize_php_compat_expr(&mut expr.expr),
        Expr::PhpCloneWith(expr) => {
            normalize_php_compat_expr(&mut expr.object);
            normalize_php_compat_expr(&mut expr.updates);
        }
        Expr::Cast(expr) => normalize_php_compat_expr(&mut expr.expr),
        Expr::TypeAscription(expr) => normalize_php_compat_expr(&mut expr.expr),
        Expr::Field(expr) => normalize_php_compat_expr(&mut expr.object),
        Expr::Index(expr) => {
            normalize_php_compat_expr(&mut expr.collection);
            normalize_php_compat_expr(&mut expr.index);
        }
        Expr::TargetAssign(expr) => {
            normalize_php_compat_expr(&mut expr.target);
            normalize_php_compat_expr(&mut expr.value);
        }
        Expr::Object(expr) => {
            for field in &mut expr.fields {
                normalize_php_compat_expr(&mut field.value);
            }
        }
        Expr::List(expr) => {
            for value in &mut expr.values {
                normalize_php_compat_expr(value);
            }
        }
        Expr::Array(expr) => {
            for element in &mut expr.elements {
                if let Some(key) = &mut element.key {
                    normalize_php_compat_expr(key);
                }
                normalize_php_compat_expr(&mut element.value);
            }
        }
        Expr::New(expr) => {
            match &mut expr.target {
                NewTarget::Expr(target) => normalize_php_compat_expr(target),
                NewTarget::AnonymousClass(class) => {
                    for member in &mut class.members {
                        normalize_php_compat_class_member(member);
                    }
                }
                NewTarget::Class(_) => {}
            }
            for arg in &mut expr.args {
                normalize_php_compat_expr(&mut arg.value);
            }
        }
        Expr::Null(_)
        | Expr::Bool(_)
        | Expr::String(_)
        | Expr::Number(_)
        | Expr::Variable(_)
        | Expr::Constant(_)
        | Expr::StaticPropertyFetch(_)
        | Expr::ClassConstantFetch(_)
        | Expr::MagicConstant(_) => {}
    }
}

fn qualified_name<'src>(separator: char) -> BoxedParser<'src, QualifiedName> {
    let name = text::ident()
        .separated_by(just(separator))
        .at_least(1)
        .collect::<Vec<_>>();

    if separator == '\\' {
        just(separator)
            .or_not()
            .ignore_then(name)
            .map(|parts| QualifiedName::new(parts.into_iter().map(str::to_string).collect()))
            .boxed()
    } else {
        name.map(|parts| QualifiedName::new(parts.into_iter().map(str::to_string).collect()))
            .boxed()
    }
}

fn dotted_function_name<'src>() -> BoxedParser<'src, String> {
    just('\\')
        .or_not()
        .ignore_then(
            text::ident()
                .separated_by(just('.'))
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .map(|parts| parts.join("."))
        .boxed()
}

fn type_expr_parser<'src>() -> BoxedParser<'src, String> {
    recursive(|type_expr| {
        let type_name = just('\\')
            .or_not()
            .ignore_then(
                text::ident()
                    .separated_by(just('\\'))
                    .at_least(1)
                    .collect::<Vec<_>>(),
            )
            .map(|parts| parts.join("\\"))
            .boxed();

        let generic_type = type_name
            .clone()
            .then(
                type_expr
                    .clone()
                    .separated_by(just(',').padded())
                    .at_least(1)
                    .collect::<Vec<_>>()
                    .delimited_by(just('<').padded(), just('>').padded())
                    .or_not(),
            )
            .map(|(name, args): (String, Option<Vec<String>>)| match args {
                Some(args) => format!("{name}<{}>", args.join(", ")),
                None => name,
            });

        generic_type
            .clone()
            .separated_by(just('|').padded())
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|parts| parts.join("|"))
            .map(|union| {
                union
                    .strip_prefix('?')
                    .map_or(union.clone(), |inner| format!("{inner}|null"))
            })
            .or(just('?')
                .ignore_then(generic_type.clone())
                .map(|inner| format!("{inner}|null")))
    })
    .boxed()
}

fn parser<'src>() -> impl Parser<'src, &'src str, Program, ParseExtra<'src>> {
    let open_php = just("<?php")
        .or(just("<?PHP"))
        .map_with(|_, extra| {
            let span: SimpleSpan = extra.span();
            Span::new(span.start, span.end)
        })
        .padded();

    let type_expr = type_expr_parser();
    let prefix_typed_param = type_expr
        .clone()
        .padded()
        .or_not()
        .then_ignore(just("...").padded().or_not())
        .then_ignore(just('&').padded().or_not())
        .then_ignore(just('$'))
        .then(text::ident().padded())
        .map(|(ty, name): (Option<String>, &str)| TypedParam {
            name: name.to_string(),
            attributes: Vec::new(),
            ty,
            default_value: None,
            promoted_visibility: None,
        });

    let suffix_typed_param = just("...")
        .padded()
        .or_not()
        .then_ignore(just('&').padded().or_not())
        .ignore_then(just('$'))
        .ignore_then(text::ident().padded())
        .then(
            just(':')
                .padded()
                .ignore_then(type_expr.clone().padded())
                .or_not(),
        )
        .map(|(name, ty): (&str, Option<String>)| TypedParam {
            name: name.to_string(),
            attributes: Vec::new(),
            ty,
            default_value: None,
            promoted_visibility: None,
        });

    let param_default_expr = just('[')
        .padded()
        .then_ignore(just(']').padded())
        .map_with(|_, extra| {
            let span: SimpleSpan = extra.span();
            Expr::Array(ArrayExpr {
                elements: Vec::new(),
                span: Span::new(span.start, span.end),
            })
        })
        .or(text::keyword(kw::NULL.text).map_with(|_, extra| {
            let span: SimpleSpan = extra.span();
            Expr::Null(NullLiteral {
                span: Span::new(span.start, span.end),
            })
        }));

    let typed_param = suffix_typed_param
        .or(prefix_typed_param)
        .then(just('=').padded().ignore_then(param_default_expr).or_not())
        .map(|(mut param, default_value)| {
            param.default_value = default_value;
            param
        })
        .boxed();

    let mut expr = Recursive::declare();
    let mut statement = Recursive::declare();
    let mut anonymous_class_target = Recursive::declare();

    expr.define({
        let expr = expr.clone();
        let statement = statement.clone();
        let anonymous_class_target = anonymous_class_target.clone();

        let null = text::keyword(kw::NULL.text)
            .or(text::keyword("NULL"))
            .map_with(|_, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Null(NullLiteral {
                    span: Span::new(span.start, span.end),
                })
            });

        let bool_literal = text::keyword(kw::TRUE.text)
            .or(text::keyword("TRUE"))
            .to(true)
            .or(text::keyword(kw::FALSE.text)
                .or(text::keyword("FALSE"))
                .to(false))
            .map_with(|value, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Bool(BoolLiteral {
                    value,
                    span: Span::new(span.start, span.end),
                })
            });

        let double_quoted_char = just('\\')
            .then(any())
            .map(|(_, c)| format!("\\{c}"))
            .or(none_of('"').map(|c: char| c.to_string()));
        let double_quoted_string = double_quoted_char
            .repeated()
            .collect::<Vec<_>>()
            .map(|parts| parts.concat())
            .delimited_by(just('"'), just('"'))
            .map(unescape_double_quoted_string)
            .map_with(|value, extra| {
                let span: SimpleSpan = extra.span();

                Expr::String(StringLiteral {
                    value,
                    span: Span::new(span.start, span.end),
                })
            });

        let single_quoted_char = just('\\')
            .then(any())
            .map(|(_, c)| format!("\\{c}"))
            .or(none_of('\'').map(|c: char| c.to_string()));
        let single_quoted_string = single_quoted_char
            .repeated()
            .collect::<Vec<_>>()
            .map(|parts| parts.concat())
            .delimited_by(just('\''), just('\''))
            .map(unescape_single_quoted_string)
            .map_with(|value, extra| {
                let span: SimpleSpan = extra.span();

                Expr::String(StringLiteral {
                    value,
                    span: Span::new(span.start, span.end),
                })
            });

        let string = double_quoted_string.or(single_quoted_string).boxed();

        let number = text::digits(10)
            .then(just('.').then(text::digits(10)).or_not())
            .then(
                just('e')
                    .or(just('E'))
                    .then(just('-').or(just('+')).or_not())
                    .then(text::digits(10))
                    .or_not(),
            )
            .to_slice()
            .map_with(|value: &str, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Number(NumberLiteral {
                    value: value.to_string(),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let variable = just('$')
            .ignore_then(text::ident())
            .map_with(|name: &str, extra| {
                let span: SimpleSpan = extra.span();

                let span = Span::new(span.start, span.end);
                ReceiverConst::from_variable_name(name).map_or_else(
                    || {
                        Expr::Variable(VariableExpr {
                            name: name.to_string(),
                            span,
                        })
                    },
                    |kind| Expr::ReceiverConst(ReceiverConstExpr { kind, span }),
                )
            })
            .boxed();

        let php_qualified_name = qualified_name('\\');

        let magic_dir = just("__DIR__")
            .map_with(|_, extra| {
                let span: SimpleSpan = extra.span();

                Expr::MagicConstant(MagicConstantExpr {
                    kind: MagicConstantKind::Dir,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let constant_expr = php_qualified_name
            .clone()
            .map_with(|name: QualifiedName, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Constant(ConstantExpr {
                    name: name.as_string(),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let named_arg = text::ident()
            .padded()
            .then_ignore(just(':').padded())
            .then(expr.clone())
            .map_with(|(name, value): (&str, Expr), extra| {
                let span: SimpleSpan = extra.span();

                CallArg {
                    name: Some(name.to_string()),
                    value,
                    span: Span::new(span.start, span.end),
                }
            });

        let positional_arg = just("...")
            .padded()
            .or_not()
            .ignore_then(expr.clone())
            .map_with(|value, extra| {
                let span: SimpleSpan = extra.span();

                CallArg {
                    name: None,
                    value,
                    span: Span::new(span.start, span.end),
                }
            });

        let first_class_callable_placeholder_arg = just("...").map_with(|_, extra| {
            let span: SimpleSpan = extra.span();
            CallArg {
                name: None,
                value: Expr::Null(NullLiteral {
                    span: Span::new(span.start, span.end),
                }),
                span: Span::new(span.start, span.end),
            }
        });

        let args = named_arg
            .or(positional_arg)
            .or(first_class_callable_placeholder_arg)
            .padded()
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect::<Vec<_>>()
            .boxed();

        let function_name = dotted_function_name();

        let function_call_expr = function_name
            .clone()
            .padded()
            .then_ignore(just('(').padded())
            .then(args.clone())
            .then_ignore(just(')').padded())
            .map_with(|(name, args): (String, Vec<CallArg>), extra| {
                let span: SimpleSpan = extra.span();

                Expr::FunctionCall(FunctionCallExpr {
                    name,
                    args,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let dynamic_function_call_expr = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just('(').padded())
            .then(args.clone())
            .then_ignore(just(')').padded())
            .map_with(|(name, args): (&str, Vec<CallArg>), extra| {
                let span: SimpleSpan = extra.span();

                Expr::DynamicFunctionCall(DynamicFunctionCallExpr {
                    name: name.to_string(),
                    args,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let static_call_expr = php_qualified_name
            .clone()
            .then_ignore(just("::").padded())
            .then(text::ident().padded())
            .then_ignore(just('(').padded())
            .then(args.clone())
            .then_ignore(just(')').padded())
            .map_with(
                |((class_name, method), args): ((QualifiedName, &str), Vec<CallArg>), extra| {
                    let span: SimpleSpan = extra.span();

                    Expr::StaticCall(StaticCallExpr {
                        class_name,
                        method: method.to_string(),
                        args,
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let static_property_fetch_expr = php_qualified_name
            .clone()
            .then_ignore(just("::").padded())
            .then_ignore(just('$').padded())
            .then(text::ident().padded())
            .map_with(|(class_name, property): (QualifiedName, &str), extra| {
                let span: SimpleSpan = extra.span();

                Expr::StaticPropertyFetch(Box::new(StaticPropertyFetchExpr {
                    class_name,
                    property: property.to_string(),
                    span: Span::new(span.start, span.end),
                }))
            })
            .boxed();

        let class_constant_fetch_expr = php_qualified_name
            .clone()
            .then_ignore(just("::").padded())
            .then(text::ident().padded())
            .map_with(|(class_name, constant): (QualifiedName, &str), extra| {
                let span: SimpleSpan = extra.span();

                Expr::ClassConstantFetch(Box::new(ClassConstantFetchExpr {
                    class_name,
                    constant: constant.to_string(),
                    span: Span::new(span.start, span.end),
                }))
            })
            .boxed();

        let dynamic_new_target =
            just('$')
                .ignore_then(text::ident().padded())
                .map_with(|name: &str, extra| {
                    let span: SimpleSpan = extra.span();
                    NewTarget::Expr(Box::new(Expr::Variable(VariableExpr {
                        name: name.to_string(),
                        span: Span::new(span.start, span.end),
                    })))
                });

        let new_target = dynamic_new_target
            .or(variable.clone().map(|expr| NewTarget::Expr(Box::new(expr))))
            .or(php_qualified_name.clone().map(NewTarget::Class));

        let anonymous_new_target = anonymous_class_target.clone();
        let named_new_target = new_target
            .padded()
            .then(
                args.clone()
                    .delimited_by(just('(').padded(), just(')').padded())
                    .or_not(),
            )
            .map(|(target, args): (NewTarget, Option<Vec<CallArg>>)| {
                (target, args.unwrap_or_default())
            });

        let new_expr = text::keyword(kw::NEW.text)
            .padded()
            .ignore_then(anonymous_new_target.or(named_new_target))
            .map_with(|(target, args): (NewTarget, Vec<CallArg>), extra| {
                let span: SimpleSpan = extra.span();

                Expr::New(Box::new(NewExpr {
                    target,
                    args,
                    span: Span::new(span.start, span.end),
                }))
            })
            .boxed();

        let closure_expr = text::keyword(kw::STATIC.text)
            .padded()
            .or_not()
            .ignore_then(text::keyword(kw::FUNCTION.text))
            .padded()
            .ignore_then(just('(').padded())
            .ignore_then(
                typed_param
                    .clone()
                    .padded()
                    .separated_by(just(',').padded())
                    .allow_trailing()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(just(')').padded())
            .then(
                text::keyword(kw::USE.text)
                    .padded()
                    .ignore_then(
                        just('&')
                            .padded()
                            .or_not()
                            .ignore_then(just('$'))
                            .ignore_then(text::ident().padded())
                            .separated_by(just(',').padded())
                            .allow_trailing()
                            .collect::<Vec<_>>()
                            .delimited_by(just('(').padded(), just(')').padded()),
                    )
                    .map(|captures| captures.into_iter().map(str::to_string).collect::<Vec<_>>())
                    .or_not(),
            )
            .then(
                just(':')
                    .padded()
                    .ignore_then(type_expr.clone().padded())
                    .or_not(),
            )
            .then(
                statement
                    .clone()
                    .repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(just('{').padded(), just('}').padded()),
            )
            .map_with(
                |(((params, captures), return_type), body): (
                    ((Vec<TypedParam>, Option<Vec<String>>), Option<String>),
                    Vec<Stmt>,
                ),
                 extra: &mut MapExtra<'src, '_, &'src str, ParseExtra<'src>>| {
                    let span: SimpleSpan = extra.span();

                    Expr::Closure(Box::new(ClosureExpr {
                        params,
                        captures: captures.unwrap_or_default(),
                        return_type,
                        body,
                        span: Span::new(span.start, span.end),
                    }))
                },
            )
            .boxed();

        let arrow_function_expr = text::keyword(kw::FN.text)
            .padded()
            .ignore_then(just('(').padded())
            .ignore_then(
                typed_param
                    .clone()
                    .padded()
                    .separated_by(just(',').padded())
                    .allow_trailing()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(just(')').padded())
            .then(
                just(':')
                    .padded()
                    .ignore_then(type_expr.clone().padded())
                    .or_not(),
            )
            .then_ignore(just("=>").padded())
            .then(expr.clone())
            .map_with(
                |((params, return_type), body): ((Vec<TypedParam>, Option<String>), Expr),
                 extra: &mut MapExtra<'src, '_, &'src str, ParseExtra<'src>>| {
                    let span: SimpleSpan = extra.span();

                    Expr::ArrowFunction(Box::new(ArrowFunctionExpr {
                        params,
                        return_type,
                        body,
                        span: Span::new(span.start, span.end),
                    }))
                },
            )
            .boxed();

        let run_expr = text::keyword(kw::RUN.text)
            .padded()
            .ignore_then(expr.clone())
            .map_with(|task, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Run(RunExpr::Task {
                    expr: Box::new(task),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let spawn_expr = text::keyword(kw::SPAWN.text)
            .padded()
            .ignore_then(expr.clone())
            .map_with(|command, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Spawn(SpawnExpr {
                    command: Box::new(command),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let fork_expr = text::keyword(kw::FORK.text)
            .padded()
            .ignore_then(expr.clone())
            .map_with(|task, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Fork(ForkExpr::Task {
                    expr: Box::new(task),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let join_expr = text::keyword(kw::JOIN.text)
            .padded()
            .ignore_then(expr.clone())
            .map_with(|handle, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Join(JoinExpr {
                    handle: Box::new(handle),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let include_expr = text::keyword(kw::REQUIRE_ONCE.text)
            .to(IncludeKind::RequireOnce)
            .or(text::keyword(kw::REQUIRE.text).to(IncludeKind::Require))
            .or(text::keyword(kw::INCLUDE_ONCE.text).to(IncludeKind::IncludeOnce))
            .or(text::keyword(kw::INCLUDE.text).to(IncludeKind::Include))
            .padded()
            .then(expr.clone())
            .map_with(|(kind, path), extra| {
                let span: SimpleSpan = extra.span();

                Expr::Include(Box::new(IncludeExpr {
                    kind,
                    path,
                    span: Span::new(span.start, span.end),
                }))
            })
            .boxed();

        let list_expr = expr
            .clone()
            .padded()
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just('{').padded(), just('}').padded())
            .map_with(|values, extra| {
                let span: SimpleSpan = extra.span();

                Expr::List(ListExpr {
                    values,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let array_element = expr
            .clone()
            .padded()
            .then(
                just('=')
                    .then_ignore(just('>'))
                    .padded()
                    .ignore_then(expr.clone().padded())
                    .or_not(),
            )
            .map_with(|(left, value): (Expr, Option<Expr>), extra| {
                let span: SimpleSpan = extra.span();

                match value {
                    Some(value) => ArrayElement {
                        key: Some(left),
                        value,
                        span: Span::new(span.start, span.end),
                    },
                    None => ArrayElement {
                        key: None,
                        value: left,
                        span: Span::new(span.start, span.end),
                    },
                }
            })
            .boxed();

        let array_expr = array_element
            .clone()
            .padded()
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just('[').padded(), just(']').padded())
            .map_with(|elements, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Array(ArrayExpr {
                    elements,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let empty_array_expr = just('[')
            .padded()
            .then_ignore(just(']').padded())
            .map_with(|_, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Array(ArrayExpr {
                    elements: Vec::new(),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let array_function_expr = text::keyword("array")
            .padded()
            .ignore_then(
                array_element
                    .clone()
                    .padded()
                    .separated_by(just(',').padded())
                    .allow_trailing()
                    .collect::<Vec<_>>()
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .map_with(|elements, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Array(ArrayExpr {
                    elements,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let object_field = text::ident()
            .padded()
            .then_ignore(just(':').padded())
            .then(expr.clone().padded())
            .then_ignore(just(';').padded().repeated())
            .map(|(name, value): (&str, Expr)| ObjectField {
                name: name.to_string(),
                value,
            })
            .boxed();

        let structural_object_expr = object_field
            .clone()
            .repeated()
            .at_least(1)
            .collect::<Vec<_>>()
            .delimited_by(just('{').padded(), just('}').padded())
            .map_with(|fields, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Object(ObjectExpr {
                    name: String::new(),
                    fields,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let type_ascribed_structural_object_expr = structural_object_expr
            .clone()
            .then_ignore(just(':').padded())
            .then(type_expr.clone().padded())
            .map_with(|(expr, ty), extra| {
                let span: SimpleSpan = extra.span();

                Expr::TypeAscription(Box::new(TypeAscriptionExpr {
                    expr,
                    ty,
                    span: Span::new(span.start, span.end),
                }))
            })
            .boxed();

        let object_expr = text::ident()
            .padded()
            .then(
                object_field
                    .repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(just('{').padded(), just('}').padded()),
            )
            .map_with(|(name, fields): (&str, Vec<ObjectField>), extra| {
                let span: SimpleSpan = extra.span();

                Expr::Object(ObjectExpr {
                    name: name.to_string(),
                    fields,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let parenthesized = expr
            .clone()
            .delimited_by(just('(').padded(), just(')').padded())
            .boxed();

        let php_clone_with_expr = text::keyword("clone")
            .padded()
            .ignore_then(
                expr.clone()
                    .then_ignore(just(',').padded())
                    .then(expr.clone())
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .map_with(|(object, updates), extra| {
                let span: SimpleSpan = extra.span();

                Expr::PhpCloneWith(Box::new(PhpCloneWithExpr {
                    object,
                    updates,
                    span: Span::new(span.start, span.end),
                }))
            })
            .boxed();

        let match_condition = text::keyword("default")
            .padded()
            .to(Vec::<Expr>::new())
            .or(expr
                .clone()
                .separated_by(just(',').padded())
                .at_least(1)
                .collect::<Vec<_>>())
            .boxed();
        let match_arm = match_condition
            .then_ignore(just("=>").padded())
            .then(expr.clone())
            .then_ignore(just(',').padded().or_not())
            .map_with(|(conditions, value): (Vec<Expr>, Expr), extra| {
                let span: SimpleSpan = extra.span();

                MatchArm {
                    conditions,
                    value,
                    span: Span::new(span.start, span.end),
                }
            })
            .boxed();
        let match_expr = text::keyword("match")
            .padded()
            .ignore_then(
                expr.clone()
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .then(
                match_arm
                    .repeated()
                    .at_least(1)
                    .collect::<Vec<_>>()
                    .delimited_by(just('{').padded(), just('}').padded()),
            )
            .map_with(|(subject, arms): (Expr, Vec<MatchArm>), extra| {
                let span: SimpleSpan = extra.span();

                Expr::Match(Box::new(MatchExpr {
                    subject,
                    arms,
                    span: Span::new(span.start, span.end),
                }))
            })
            .boxed();

        let callable_expr = dynamic_function_call_expr.or(function_call_expr).boxed();

        let print_expr = text::keyword("print")
            .padded()
            .ignore_then(expr.clone())
            .map_with(|value, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Print(Box::new(PrintExpr {
                    value,
                    span: Span::new(span.start, span.end),
                }))
            })
            .boxed();

        let primary_atom = choice((
            match_expr,
            print_expr,
            run_expr,
            fork_expr,
            spawn_expr,
            join_expr,
            include_expr,
            closure_expr,
            arrow_function_expr,
            new_expr,
            php_clone_with_expr,
            parenthesized,
            type_ascribed_structural_object_expr,
            structural_object_expr,
            object_expr,
            static_call_expr,
            static_property_fetch_expr,
            class_constant_fetch_expr,
            array_function_expr,
            callable_expr,
            variable,
        ))
        .boxed();

        let literal_atom = choice((
            magic_dir,
            null,
            bool_literal,
            empty_array_expr,
            array_expr,
            list_expr,
            string,
            number,
            constant_expr,
        ))
        .boxed();

        let atom = primary_atom.or(literal_atom).boxed();

        #[derive(Clone)]
        enum Postfix {
            MethodCall {
                method: String,
                method_span: Span,
                args: Vec<CallArg>,
            },
            Field(String),
            Index(Expr),
            Call(Vec<CallArg>),
        }

        let member_name = text::ident().map_with(|method: &str, extra| {
            let span: SimpleSpan = extra.span();
            (method.to_string(), Span::new(span.start, span.end))
        });
        let dynamic_member_name = just('{')
            .padded()
            .ignore_then(just('$'))
            .ignore_then(text::ident().padded())
            .then_ignore(just('}').padded())
            .map_with(|method: &str, extra| {
                let span: SimpleSpan = extra.span();
                (format!("${{{method}}}"), Span::new(span.start, span.end))
            });
        let arrow_postfix = just("->")
            .ignore_then(dynamic_member_name.or(member_name.clone()))
            .padded()
            .then(
                args.clone()
                    .delimited_by(just('(').padded(), just(')').padded())
                    .or_not(),
            )
            .map(
                |((name, name_span), args): ((String, Span), Option<Vec<CallArg>>)| match args {
                    Some(args) => Postfix::MethodCall {
                        method: name,
                        method_span: name_span,
                        args,
                    },
                    None => Postfix::Field(name),
                },
            );
        let dot_postfix = just('.')
            .ignore_then(member_name)
            .then(
                args.clone()
                    .delimited_by(just('(').padded(), just(')').padded())
                    .or_not(),
            )
            .map(
                |((name, name_span), args): ((String, Span), Option<Vec<CallArg>>)| match args {
                    Some(args) => Postfix::MethodCall {
                        method: name,
                        method_span: name_span,
                        args,
                    },
                    None => Postfix::Field(name),
                },
            );
        let index_postfix = expr
            .clone()
            .padded()
            .delimited_by(just('[').padded(), just(']').padded())
            .map(Postfix::Index);
        let call_postfix = args
            .clone()
            .delimited_by(just('(').padded(), just(')').padded())
            .map(Postfix::Call);

        let fielded = atom
            .clone()
            .foldl(
                arrow_postfix
                    .or(dot_postfix)
                    .or(index_postfix)
                    .or(call_postfix)
                    .repeated(),
                |left, postfix| match postfix {
                    Postfix::MethodCall {
                        method,
                        method_span,
                        args,
                    } => {
                        let end = args
                            .last()
                            .map_or(left.span().end + method.len() + 4, |arg| arg.span.end + 1);
                        let span = Span::new(left.span().start, end);

                        Expr::MethodCall(Box::new(MethodCallExpr {
                            object: left,
                            method,
                            method_span,
                            args,
                            span,
                        }))
                    }
                    Postfix::Field(field) => {
                        let span = Span::new(left.span().start, left.span().end + field.len() + 1);

                        Expr::Field(Box::new(FieldExpr {
                            object: left,
                            field,
                            span,
                        }))
                    }
                    Postfix::Index(index) => {
                        let span = Span::new(left.span().start, index.span().end + 1);

                        Expr::Index(Box::new(IndexExpr {
                            collection: left,
                            index,
                            span,
                        }))
                    }
                    Postfix::Call(args) => {
                        let end = args
                            .last()
                            .map_or(left.span().end + 2, |arg| arg.span.end + 1);
                        let span = Span::new(left.span().start, end);

                        Expr::DynamicCall(Box::new(DynamicCallExpr {
                            callee: left,
                            args,
                            span,
                        }))
                    }
                },
            )
            .boxed();

        let powered = fielded
            .clone()
            .then_ignore(just("**").padded())
            .repeated()
            .foldr(fielded.clone(), |left, right| {
                let span = Span::new(left.span().start, right.span().end);

                Expr::Binary(Box::new(BinaryExpr {
                    left,
                    op: BinaryOp::Pow,
                    right,
                    span,
                }))
            })
            .boxed();

        let unary_op = just('+')
            .to(UnaryOp::Plus)
            .or(just('-').to(UnaryOp::Minus))
            .or(just('!').to(UnaryOp::Not))
            .or(text::keyword("clone")
                .then_ignore(just('(').padded().not())
                .to(UnaryOp::Clone))
            .map_with(|op, extra| {
                let span: SimpleSpan = extra.span();
                (op, Span::new(span.start, span.end))
            })
            .padded();

        let cast_type = text::keyword("array")
            .or(text::keyword("string"))
            .or(text::keyword("int"))
            .or(text::keyword("integer"))
            .or(text::keyword("bool"))
            .or(text::keyword("boolean"))
            .or(text::keyword("float"))
            .or(text::keyword("double"))
            .or(text::keyword("void"))
            .padded()
            .delimited_by(just('(').padded(), just(')').padded());

        let casted = cast_type
            .repeated()
            .foldr(powered, |ty: &str, expr| {
                let span = Span::new(
                    expr.span().start.saturating_sub(ty.len() + 2),
                    expr.span().end,
                );
                Expr::Cast(Box::new(CastExpr {
                    ty: ty.to_string(),
                    expr,
                    span,
                }))
            })
            .boxed();

        let signed = unary_op
            .repeated()
            .foldr(casted, |(op, op_span), expr| {
                let span = Span::new(op_span.start, expr.span().end);
                Expr::Unary(Box::new(UnaryExpr { op, expr, span }))
            })
            .boxed();

        let multiplicative = signed
            .clone()
            .foldl(
                just('*')
                    .to(BinaryOp::Mul)
                    .or(just('/').to(BinaryOp::Div))
                    .or(just('%').to(BinaryOp::Mod))
                    .padded()
                    .then(signed)
                    .repeated(),
                |left, (op, right)| {
                    let span = Span::new(left.span().start, right.span().end);

                    Expr::Binary(Box::new(BinaryExpr {
                        left,
                        op,
                        right,
                        span,
                    }))
                },
            )
            .boxed();

        let dotted = multiplicative
            .clone()
            .foldl(
                just('.')
                    .padded()
                    .ignore_then(multiplicative.clone())
                    .repeated(),
                |left, right| {
                    let span = Span::new(left.span().start, right.span().end);

                    Expr::Binary(Box::new(BinaryExpr {
                        left,
                        op: BinaryOp::Concat,
                        right,
                        span,
                    }))
                },
            )
            .boxed();

        let additive = dotted
            .clone()
            .foldl(
                just('+')
                    .to(BinaryOp::Add)
                    .or(just('-').to(BinaryOp::Sub))
                    .padded()
                    .then(dotted)
                    .repeated(),
                |left, (op, right)| {
                    let span = Span::new(left.span().start, right.span().end);

                    Expr::Binary(Box::new(BinaryExpr {
                        left,
                        op,
                        right,
                        span,
                    }))
                },
            )
            .boxed();

        let is_expr = additive
            .clone()
            .foldl(
                text::keyword(kw::IS.text)
                    .padded()
                    .ignore_then(text::keyword(kw::NOT.text).padded().to(true).or_not())
                    .then_ignore(text::keyword(kw::NULL.text).padded())
                    .repeated(),
                |left, is_not| {
                    let span = left.span();

                    Expr::Binary(Box::new(BinaryExpr {
                        left,
                        op: if is_not.unwrap_or(false) {
                            BinaryOp::IsNot
                        } else {
                            BinaryOp::Is
                        },
                        right: Expr::Null(NullLiteral { span }),
                        span,
                    }))
                },
            )
            .boxed();

        let variable_assign = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then(expr.clone())
            .map_with(|(name, value): (&str, Expr), extra| {
                let span: SimpleSpan = extra.span();

                Expr::Assign(Box::new(AssignExpr {
                    name: name.to_string(),
                    value,
                    span: Span::new(span.start, span.end),
                }))
            });

        let not_variable_assign =
            just('!')
                .padded()
                .ignore_then(variable_assign.clone())
                .map(|expr| {
                    let span = Span::new(expr.span().start.saturating_sub(1), expr.span().end);
                    Expr::Unary(Box::new(UnaryExpr {
                        op: UnaryOp::Not,
                        expr,
                        span,
                    }))
                });

        let comparison_operand = not_variable_assign
            .clone()
            .or(variable_assign.clone())
            .or(is_expr.clone())
            .boxed();

        let comparison = comparison_operand
            .clone()
            .foldl(
                just("!==")
                    .to(BinaryOp::NotIdentical)
                    .or(just("===").to(BinaryOp::Identical))
                    .or(just("==").to(BinaryOp::Equal))
                    .or(just("!=").to(BinaryOp::NotEqual))
                    .or(text::keyword("instanceof").to(BinaryOp::InstanceOf))
                    .or(just(">=").to(BinaryOp::GreaterThanOrEqual))
                    .or(just("<=").to(BinaryOp::LessThanOrEqual))
                    .or(just('>').to(BinaryOp::GreaterThan))
                    .or(just('<').to(BinaryOp::LessThan))
                    .padded()
                    .then(comparison_operand)
                    .repeated(),
                |left, (op, right)| {
                    let span = Span::new(left.span().start, right.span().end);

                    Expr::Binary(Box::new(BinaryExpr {
                        left,
                        op,
                        right,
                        span,
                    }))
                },
            )
            .boxed();

        let static_property_assign = php_qualified_name
            .clone()
            .then_ignore(just("::").padded())
            .then_ignore(just('$').padded())
            .then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then(expr.clone())
            .map_with(
                |((class_name, property), value): ((QualifiedName, &str), Expr), extra| {
                    let span: SimpleSpan = extra.span();

                    Expr::StaticPropertyAssign(Box::new(StaticPropertyAssignExpr {
                        class_name,
                        property: property.to_string(),
                        value,
                        span: Span::new(span.start, span.end),
                    }))
                },
            );

        let static_property_coalesce_assign = php_qualified_name
            .clone()
            .then_ignore(just("::").padded())
            .then_ignore(just('$').padded())
            .then(text::ident().padded())
            .then_ignore(just("??=").padded())
            .then(expr.clone())
            .map_with(
                |((class_name, property), value): ((QualifiedName, &str), Expr), extra| {
                    let span: SimpleSpan = extra.span();

                    Expr::StaticPropertyCoalesceAssign(Box::new(StaticPropertyAssignExpr {
                        class_name,
                        property: property.to_string(),
                        value,
                        span: Span::new(span.start, span.end),
                    }))
                },
            );

        let index_assign = fielded
            .clone()
            .then_ignore(just('=').padded())
            .then(expr.clone())
            .map_with(|(target, value): (Expr, Expr), extra| {
                let span: SimpleSpan = extra.span();

                Expr::TargetAssign(Box::new(TargetAssignExpr {
                    target,
                    value,
                    span: Span::new(span.start, span.end),
                }))
            });

        let logical_and = comparison
            .clone()
            .foldl(
                just("&&")
                    .to(BinaryOp::And)
                    .padded()
                    .then(variable_assign.clone().or(comparison.clone()))
                    .repeated(),
                |left, (op, right)| {
                    let span = Span::new(left.span().start, right.span().end);

                    Expr::Binary(Box::new(BinaryExpr {
                        left,
                        op,
                        right,
                        span,
                    }))
                },
            )
            .boxed();

        let logical_or = logical_and
            .clone()
            .foldl(
                just("||")
                    .to(BinaryOp::Or)
                    .padded()
                    .then(variable_assign.clone().or(logical_and.clone()))
                    .repeated(),
                |left, (op, right)| {
                    let span = Span::new(left.span().start, right.span().end);

                    Expr::Binary(Box::new(BinaryExpr {
                        left,
                        op,
                        right,
                        span,
                    }))
                },
            )
            .boxed();

        let coalesce = logical_or
            .clone()
            .foldl(
                just("??")
                    .to(BinaryOp::Coalesce)
                    .padded()
                    .then(logical_or.clone())
                    .repeated(),
                |left, (op, right)| {
                    let span = Span::new(left.span().start, right.span().end);

                    Expr::Binary(Box::new(BinaryExpr {
                        left,
                        op,
                        right,
                        span,
                    }))
                },
            )
            .boxed();

        let pipe = coalesce
            .clone()
            .foldl(
                just("|>")
                    .to(BinaryOp::Pipe)
                    .padded()
                    .then(coalesce.clone())
                    .repeated(),
                |left, (op, right)| {
                    let span = Span::new(left.span().start, right.span().end);

                    Expr::Binary(Box::new(BinaryExpr {
                        left,
                        op,
                        right,
                        span,
                    }))
                },
            )
            .boxed();

        let ternary = pipe
            .clone()
            .then(
                just('?')
                    .padded()
                    .ignore_then(coalesce.clone().or_not())
                    .then_ignore(just(':').padded())
                    .then(coalesce.clone())
                    .or_not(),
            )
            .map(|(condition, branches)| match branches {
                Some((if_true, if_false)) => {
                    let if_true = if_true.unwrap_or_else(|| condition.clone());
                    let span = Span::new(condition.span().start, if_false.span().end);
                    Expr::Ternary(Box::new(TernaryExpr {
                        condition,
                        if_true,
                        if_false,
                        span,
                    }))
                }
                None => condition,
            })
            .boxed();

        static_property_coalesce_assign
            .or(static_property_assign)
            .or(variable_assign)
            .or(index_assign)
            .or(ternary)
            .boxed()
    });

    statement.define({
        let statement = statement.clone();
        let expr = expr.clone();

        let close_php = just("?>").padded().ignored();
        let terminator = just(';')
            .padded()
            .ignored()
            .or(close_php.clone())
            .or(end().ignored());

        let php_name = qualified_name('\\');
        let dotted_name = qualified_name('.');
        let module_stmt = text::keyword(kw::MODULE.text)
            .padded()
            .ignore_then(dotted_name.clone())
            .then_ignore(terminator.clone())
            .map_with(|name, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Namespace(NamespaceStmt {
                    source: NamespaceSource::Echo,
                    name,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let namespace_stmt = text::keyword(kw::NAMESPACE.text)
            .padded()
            .ignore_then(
                text::keyword(kw::STD.text)
                    .padded()
                    .ignore_then(php_name.clone())
                    .map(|name| (NamespaceSource::Std, name))
                    .or(php_name.clone().map(|name| (NamespaceSource::Php, name)))
                    .or_not()
                    .map(|namespace| {
                        namespace.unwrap_or_else(|| {
                            (NamespaceSource::Php, QualifiedName::new(Vec::new()))
                        })
                    }),
            )
            .then_ignore(terminator.clone())
            .map_with(|(source, name), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Namespace(NamespaceStmt {
                    source,
                    name,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let use_stmt = text::keyword(kw::USE.text)
            .padded()
            .ignore_then(text::keyword("function").padded().or_not())
            .ignore_then(php_name.clone())
            .then(
                text::keyword(kw::AS.text)
                    .padded()
                    .ignore_then(text::ident().padded())
                    .or_not(),
            )
            .then_ignore(terminator.clone())
            .map_with(|(name, alias): (QualifiedName, Option<&str>), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Use(UseStmt {
                    name,
                    alias: alias.map(str::to_string),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let echo_std_use_stmt = text::keyword(kw::USE.text)
            .padded()
            .ignore_then(text::keyword(kw::STD.text).padded())
            .ignore_then(just('.').padded())
            .ignore_then(dotted_name.clone())
            .then(
                text::keyword(kw::AS.text)
                    .padded()
                    .ignore_then(text::ident().padded())
                    .or_not(),
            )
            .then_ignore(terminator.clone())
            .map_with(|(name, alias): (QualifiedName, Option<&str>), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Import(ImportStmt {
                    source: ImportSource::Std,
                    name,
                    alias: alias.map(str::to_string),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let dotted_use_stmt = text::keyword(kw::USE.text)
            .padded()
            .ignore_then(dotted_name.clone())
            .then(
                text::keyword(kw::AS.text)
                    .padded()
                    .ignore_then(text::ident().padded())
                    .or_not(),
            )
            .then_ignore(terminator.clone())
            .map_with(|(name, alias): (QualifiedName, Option<&str>), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Use(UseStmt {
                    name,
                    alias: alias.map(str::to_string),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let import_source = text::keyword(kw::STD.text)
            .padded()
            .to(ImportSource::Std)
            .or(just('"')
                .ignore_then(none_of('"').repeated().collect::<String>())
                .then_ignore(just('"'))
                .padded()
                .map(ImportSource::File))
            .boxed();

        let import_stmt = text::keyword(kw::FROM.text)
            .padded()
            .ignore_then(import_source)
            .then_ignore(text::keyword(kw::USE.text).padded())
            .then(php_name.clone())
            .then(
                text::keyword(kw::AS.text)
                    .padded()
                    .ignore_then(text::ident().padded())
                    .or_not(),
            )
            .then_ignore(terminator.clone())
            .map_with(
                |((source, name), alias): ((ImportSource, QualifiedName), Option<&str>), extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::Import(ImportStmt {
                        source,
                        name,
                        alias: alias.map(str::to_string),
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let echo_exprs = expr
            .clone()
            .padded()
            .separated_by(just(',').padded())
            .at_least(1)
            .collect::<Vec<_>>();

        let echo_stmt = just("echo")
            .padded()
            .ignore_then(echo_exprs.clone())
            .then_ignore(terminator.clone())
            .map_with(|exprs, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Echo(EchoStmt {
                    exprs,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();
        let short_echo_stmt = just("<?=")
            .padded()
            .ignore_then(echo_exprs.clone())
            .then_ignore(just("?>").padded())
            .map_with(|exprs, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Echo(EchoStmt {
                    exprs,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let return_stmt = just("return")
            .padded()
            .ignore_then(expr.clone().padded().or_not())
            .then_ignore(terminator.clone())
            .map_with(|value, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Return(ReturnStmt {
                    value,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let throw_stmt = text::keyword(kw::THROW.text)
            .padded()
            .ignore_then(expr.clone().padded())
            .then_ignore(terminator.clone())
            .map_with(|value, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Throw(ThrowStmt {
                    value,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let yield_stmt = text::keyword(kw::YIELD.text)
            .padded()
            .ignore_then(expr.clone().padded())
            .then_ignore(terminator.clone())
            .map_with(|value, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Yield(YieldStmt {
                    value,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let goto_stmt = text::keyword("goto")
            .padded()
            .ignore_then(text::ident().padded())
            .then_ignore(terminator.clone())
            .map_with(|label: &str, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Goto(GotoStmt {
                    label: label.to_string(),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let label_stmt = text::ident()
            .padded()
            .then_ignore(just(':').padded())
            .map_with(|name: &str, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Label(LabelStmt {
                    name: name.to_string(),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let declare_directive = text::ident()
            .padded()
            .then_ignore(just('=').padded())
            .then(expr.clone())
            .map_with(|(name, value): (&str, Expr), extra| {
                let span: SimpleSpan = extra.span();

                PhpDeclareDirective {
                    name: name.to_string(),
                    value,
                    span: Span::new(span.start, span.end),
                }
            })
            .boxed();

        let global_name = just('$')
            .ignore_then(text::ident().padded())
            .map(str::to_string);
        let global_stmt = text::keyword("global")
            .padded()
            .ignore_then(
                global_name
                    .separated_by(just(',').padded())
                    .at_least(1)
                    .collect::<Vec<_>>(),
            )
            .then_ignore(terminator.clone())
            .map_with(|names, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Global(GlobalStmt {
                    names,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let static_var_decl = just('$')
            .ignore_then(text::ident().padded())
            .then(just('=').padded().ignore_then(expr.clone()).or_not())
            .map_with(|(name, value): (&str, Option<Expr>), extra| {
                let span: SimpleSpan = extra.span();

                StaticVarDecl {
                    name: name.to_string(),
                    value,
                    span: Span::new(span.start, span.end),
                }
            });
        let static_var_stmt = text::keyword(kw::STATIC.text)
            .padded()
            .ignore_then(
                static_var_decl
                    .separated_by(just(',').padded())
                    .at_least(1)
                    .collect::<Vec<_>>(),
            )
            .then_ignore(terminator.clone())
            .map_with(|vars, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::StaticVar(StaticVarStmt {
                    vars,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let statement_function_name = dotted_function_name();

        let statement_named_arg = text::ident()
            .padded()
            .then_ignore(just(':').padded())
            .then(expr.clone())
            .map_with(|(name, value): (&str, Expr), extra| {
                let span: SimpleSpan = extra.span();

                CallArg {
                    name: Some(name.to_string()),
                    value,
                    span: Span::new(span.start, span.end),
                }
            });

        let statement_positional_arg = just("...")
            .padded()
            .or_not()
            .ignore_then(expr.clone())
            .map_with(|value, extra| {
                let span: SimpleSpan = extra.span();

                CallArg {
                    name: None,
                    value,
                    span: Span::new(span.start, span.end),
                }
            });

        let statement_first_class_callable_placeholder_arg = just("...").map_with(|_, extra| {
            let span: SimpleSpan = extra.span();
            CallArg {
                name: None,
                value: Expr::Null(NullLiteral {
                    span: Span::new(span.start, span.end),
                }),
                span: Span::new(span.start, span.end),
            }
        });

        let statement_args = statement_named_arg
            .or(statement_positional_arg)
            .or(statement_first_class_callable_placeholder_arg)
            .padded()
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect::<Vec<_>>();

        let attribute = php_name
            .clone()
            .or(dotted_name.clone())
            .then(
                statement_args
                    .clone()
                    .delimited_by(just('(').padded(), just(')').padded())
                    .or_not(),
            )
            .map_with(
                |(name, args): (QualifiedName, Option<Vec<CallArg>>), extra| {
                    let span: SimpleSpan = extra.span();

                    AttributeDecl {
                        name,
                        args: args.unwrap_or_default(),
                        span: Span::new(span.start, span.end),
                    }
                },
            );

        let attributes = attribute
            .separated_by(just(',').padded())
            .at_least(1)
            .collect::<Vec<_>>()
            .delimited_by(just("#[").padded(), just(']').padded())
            .then_ignore(terminator.clone().or_not())
            .repeated()
            .collect::<Vec<Vec<_>>>()
            .map(|groups| groups.into_iter().flatten().collect::<Vec<_>>())
            .boxed();

        let function_call_stmt = statement_function_name
            .padded()
            .then_ignore(just('(').padded())
            .then(statement_args.clone())
            .then_ignore(just(')').padded())
            .then_ignore(terminator.clone())
            .try_map(|(name, args): (String, Vec<CallArg>), span| {
                if name.eq_ignore_ascii_case("print") {
                    Err(Rich::custom(
                        span,
                        "print is a language construct expression, not a function-call statement",
                    ))
                } else {
                    Ok((name, args))
                }
            })
            .map_with(|(name, args): (String, Vec<CallArg>), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::FunctionCall(FunctionCallStmt {
                    name,
                    args,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let dynamic_function_call_stmt = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just('(').padded())
            .then(statement_args.clone())
            .then_ignore(just(')').padded())
            .then_ignore(terminator.clone())
            .map_with(|(name, args): (&str, Vec<CallArg>), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::DynamicFunctionCall(DynamicFunctionCallStmt {
                    name: name.to_string(),
                    args,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let prefix_typed_param = type_expr
            .clone()
            .padded()
            .or_not()
            .then_ignore(just("...").padded().or_not())
            .then_ignore(just('&').padded().or_not())
            .then_ignore(just('$'))
            .then(text::ident().padded())
            .map(|(ty, name): (Option<String>, &str)| TypedParam {
                name: name.to_string(),
                attributes: Vec::new(),
                ty,
                default_value: None,
                promoted_visibility: None,
            });

        let suffix_typed_param = just("...")
            .padded()
            .or_not()
            .then_ignore(just('&').padded().or_not())
            .ignore_then(just('$'))
            .ignore_then(text::ident().padded())
            .then(
                just(':')
                    .padded()
                    .ignore_then(type_expr.clone().padded())
                    .or_not(),
            )
            .map(|(name, ty): (&str, Option<String>)| TypedParam {
                name: name.to_string(),
                attributes: Vec::new(),
                ty,
                default_value: None,
                promoted_visibility: None,
            });

        let typed_param = attributes
            .clone()
            .then(suffix_typed_param.or(prefix_typed_param))
            .then(just('=').padded().ignore_then(expr.clone()).or_not())
            .map(|((attributes, mut param), default_value)| {
                param.attributes = attributes;
                param.default_value = default_value;
                param
            })
            .boxed();

        let method_body = statement
            .clone()
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(just('{').padded(), just('}').padded())
            .boxed();

        let method_visibility = text::keyword(kw::PUB.text)
            .to(MethodVisibility::Public)
            .or(text::keyword(kw::PUBLIC.text).to(MethodVisibility::Public))
            .or(text::keyword(kw::PROTECTED.text).to(MethodVisibility::Protected))
            .or(text::keyword(kw::PRIVATE.text).to(MethodVisibility::Private))
            .padded()
            .or_not()
            .boxed();

        let method_param = method_visibility
            .clone()
            .then(typed_param.clone())
            .map(|(visibility, mut param)| {
                param.promoted_visibility = visibility;
                param
            })
            .or(typed_param.clone())
            .boxed();

        let function_keyword = text::keyword(kw::FN.text)
            .or(text::keyword(kw::FUNCTION.text))
            .padded()
            .boxed();

        let method_modifier = text::keyword(kw::ABSTRACT.text)
            .to("abstract")
            .or(text::keyword(kw::FINAL.text).to("final"))
            .padded();

        let method_decl = attributes
            .clone()
            .then(
                method_modifier
                    .repeated()
                    .collect::<Vec<_>>()
                    .then(method_visibility.clone()),
            )
            .clone()
            .then(
                text::keyword(kw::INTRINSIC.text)
                    .padded()
                    .to(true)
                    .or_not()
                    .map(|is_intrinsic| is_intrinsic.unwrap_or(false)),
            )
            .then(
                text::keyword(kw::STATIC.text)
                    .padded()
                    .to(true)
                    .or_not()
                    .map(|is_static| is_static.unwrap_or(false)),
            )
            .then_ignore(function_keyword)
            .then(text::ident().padded())
            .then_ignore(just('(').padded())
            .then(
                method_param
                    .clone()
                    .padded()
                    .separated_by(just(',').padded())
                    .allow_trailing()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(just(')').padded())
            .then(
                just(':')
                    .padded()
                    .ignore_then(type_expr.clone().padded())
                    .or_not(),
            )
            .then(method_body.or_not())
            .then_ignore(terminator.clone().or_not())
            .map_with(
                |(
                    (
                        (
                            (
                                (((attributes, (modifiers, visibility)), is_intrinsic), is_static),
                                name,
                            ),
                            params,
                        ),
                        return_type,
                    ),
                    body,
                ): (
                    (
                        (
                            (
                                (
                                    (
                                        (Vec<AttributeDecl>, (Vec<&str>, Option<MethodVisibility>)),
                                        bool,
                                    ),
                                    bool,
                                ),
                                &str,
                            ),
                            Vec<TypedParam>,
                        ),
                        Option<String>,
                    ),
                    Option<Vec<Stmt>>,
                ),
                 extra| {
                    let span: SimpleSpan = extra.span();

                    ClassMember::Method(MethodDecl {
                        name: name.to_string(),
                        attributes,
                        params,
                        return_type,
                        body: body.unwrap_or_default(),
                        visibility: visibility.unwrap_or(MethodVisibility::Private),
                        is_abstract: modifiers.contains(&"abstract"),
                        is_final: modifiers.contains(&"final"),
                        is_static,
                        is_intrinsic,
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let property_hook_kind = text::keyword("get")
            .to(PropertyHookKind::Get)
            .or(text::keyword("set").to(PropertyHookKind::Set))
            .padded()
            .boxed();

        let property_hook_param = type_expr
            .clone()
            .padded()
            .or_not()
            .then_ignore(just('&').padded().or_not())
            .then_ignore(just('$'))
            .then(text::ident().padded())
            .map(|(ty, name): (Option<String>, &str)| TypedParam {
                name: name.to_string(),
                attributes: Vec::new(),
                ty,
                default_value: None,
                promoted_visibility: None,
            })
            .delimited_by(just('(').padded(), just(')').padded())
            .or_not()
            .boxed();

        let property_hook_body = just("=>")
            .padded()
            .ignore_then(expr.clone())
            .then_ignore(terminator.clone())
            .map(PropertyHookBody::Expr)
            .or(statement
                .clone()
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just('{').padded(), just('}').padded())
                .then_ignore(terminator.clone().or_not())
                .map(PropertyHookBody::Block))
            .or(terminator.clone().to(PropertyHookBody::None))
            .boxed();

        let property_hook = property_hook_kind
            .then(property_hook_param)
            .then(property_hook_body)
            .map_with(
                |((kind, param), body): (
                    (PropertyHookKind, Option<TypedParam>),
                    PropertyHookBody,
                ),
                 extra| {
                    let span: SimpleSpan = extra.span();

                    PropertyHookDecl {
                        kind,
                        param,
                        body,
                        span: Span::new(span.start, span.end),
                    }
                },
            )
            .boxed();

        let property_hooks = property_hook
            .repeated()
            .at_least(1)
            .collect::<Vec<_>>()
            .delimited_by(just('{').padded(), just('}').padded())
            .boxed();

        let property_decl = attributes
            .clone()
            .then(
                method_visibility.clone().then(
                    text::keyword(kw::STATIC.text)
                        .padded()
                        .to(true)
                        .or_not()
                        .map(|is_static| is_static.unwrap_or(false)),
                ),
            )
            .then(type_expr.clone().padded().or_not())
            .then_ignore(just('$').padded())
            .then(text::ident().padded())
            .then(just('=').padded().ignore_then(expr.clone()).or_not())
            .then(property_hooks.clone().or_not())
            .then_ignore(terminator.clone().or_not())
            .map_with(
                |(((((attributes, (visibility, is_static)), ty), name), value), hooks): (
                    (
                        (
                            (
                                (Vec<AttributeDecl>, (Option<MethodVisibility>, bool)),
                                Option<String>,
                            ),
                            &str,
                        ),
                        Option<Expr>,
                    ),
                    Option<Vec<PropertyHookDecl>>,
                ),
                 extra| {
                    let span: SimpleSpan = extra.span();

                    ClassMember::Property(PropertyDecl {
                        name: name.to_string(),
                        attributes,
                        ty,
                        value,
                        hooks: hooks.unwrap_or_default(),
                        visibility: visibility.unwrap_or(MethodVisibility::Private),
                        is_static,
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let class_const_decl = attributes
            .clone()
            .then(
                method_visibility
                    .clone()
                    .then_ignore(text::keyword("const").padded()),
            )
            .then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then(expr.clone())
            .then_ignore(terminator.clone())
            .map_with(
                |(((attributes, visibility), name), value): (
                    ((Vec<AttributeDecl>, Option<MethodVisibility>), &str),
                    Expr,
                ),
                 extra| {
                    let span: SimpleSpan = extra.span();

                    ClassMember::Const(ClassConstDecl {
                        name: name.to_string(),
                        attributes,
                        value,
                        visibility: visibility.unwrap_or(MethodVisibility::Public),
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let trait_use_member = text::keyword(kw::USE.text)
            .padded()
            .ignore_then(php_name.clone().or(dotted_name.clone()))
            .then_ignore(terminator.clone())
            .map(ClassMember::TraitUse)
            .boxed();

        let class_member = trait_use_member
            .clone()
            .or(method_decl.clone())
            .or(class_const_decl.clone())
            .or(property_decl.clone())
            .boxed();

        let interface_member = method_decl
            .clone()
            .map(|member| match member {
                ClassMember::Method(method) => InterfaceMember::Method(method),
                _ => unreachable!("method_decl only produces method members"),
            })
            .or(class_const_decl.clone().map(|member| match member {
                ClassMember::Const(constant) => InterfaceMember::Const(constant),
                _ => unreachable!("class_const_decl only produces const members"),
            }))
            .or(property_decl.clone().map(|member| match member {
                ClassMember::Property(property) => InterfaceMember::Property(property),
                _ => unreachable!("property_decl only produces property members"),
            }))
            .boxed();

        let class_modifier = text::keyword(kw::ABSTRACT.text)
            .to(ClassModifier::Abstract)
            .or(text::keyword(kw::FINAL.text).to(ClassModifier::Final))
            .or(text::keyword(kw::READONLY.text).to(ClassModifier::Readonly))
            .padded();

        anonymous_class_target.define(
            class_modifier
                .clone()
                .repeated()
                .collect::<Vec<_>>()
                .then_ignore(just(kw::CLASS.text).padded())
                .then(
                    statement_args
                        .clone()
                        .delimited_by(just('(').padded(), just(')').padded())
                        .or_not(),
                )
                .then(
                    text::keyword("extends")
                        .padded()
                        .ignore_then(php_name.clone().or(dotted_name.clone()))
                        .or_not(),
                )
                .then(
                    text::keyword("implements")
                        .padded()
                        .ignore_then(
                            php_name
                                .clone()
                                .or(dotted_name.clone())
                                .separated_by(just(',').padded())
                                .at_least(1)
                                .collect::<Vec<_>>(),
                        )
                        .or_not(),
                )
                .then(
                    class_member
                        .clone()
                        .repeated()
                        .collect::<Vec<_>>()
                        .delimited_by(just('{').padded(), just('}').padded()),
                )
                .map_with(
                    |((((modifiers, args), parent), interfaces), members): (
                        (
                            (
                                (Vec<ClassModifier>, Option<Vec<CallArg>>),
                                Option<QualifiedName>,
                            ),
                            Option<Vec<QualifiedName>>,
                        ),
                        Vec<ClassMember>,
                    ),
                     extra| {
                        let span: SimpleSpan = extra.span();

                        (
                            NewTarget::AnonymousClass(Box::new(AnonymousClassExpr {
                                modifiers,
                                parent,
                                interfaces: interfaces.unwrap_or_default(),
                                members,
                                span: Span::new(span.start, span.end),
                            })),
                            args.unwrap_or_default(),
                        )
                    },
                )
                .boxed(),
        );

        let class_decl_stmt = attributes
            .clone()
            .then(class_modifier.repeated().collect::<Vec<_>>())
            .then_ignore(just(kw::CLASS.text).padded())
            .then(text::ident().padded())
            .map(
                |(attributes_modifiers, name): ((Vec<AttributeDecl>, Vec<ClassModifier>), &str)| {
                    let (attributes, modifiers) = attributes_modifiers;
                    (name, attributes, modifiers)
                },
            )
            .padded()
            .then(
                text::keyword("extends")
                    .padded()
                    .ignore_then(php_name.clone().or(dotted_name.clone()))
                    .or_not(),
            )
            .then(
                text::keyword("implements")
                    .padded()
                    .ignore_then(
                        php_name
                            .clone()
                            .or(dotted_name.clone())
                            .separated_by(just(',').padded())
                            .at_least(1)
                            .collect::<Vec<_>>(),
                    )
                    .or_not(),
            )
            .then_ignore(just('{').padded())
            .then(class_member.clone().repeated().collect::<Vec<_>>())
            .then_ignore(just('}').padded())
            .then_ignore(terminator.clone().or_not())
            .map_with(
                |((((name, attributes, modifiers), parent), interfaces), members): (
                    (
                        (
                            (&str, Vec<AttributeDecl>, Vec<ClassModifier>),
                            Option<QualifiedName>,
                        ),
                        Option<Vec<QualifiedName>>,
                    ),
                    Vec<ClassMember>,
                ),
                 extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::ClassDecl(ClassDeclStmt {
                        name: name.to_string(),
                        attributes,
                        modifiers,
                        parent,
                        interfaces: interfaces.unwrap_or_default(),
                        members,
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let trait_decl_stmt = attributes
            .clone()
            .then_ignore(text::keyword("trait").padded())
            .then(text::ident().padded())
            .then(
                class_member
                    .clone()
                    .repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(just('{').padded(), just('}').padded()),
            )
            .map_with(
                |((attributes, name), members): ((Vec<AttributeDecl>, &str), Vec<ClassMember>),
                 extra| {
                    let span: SimpleSpan = extra.span();
                    Stmt::TraitDecl(TraitDeclStmt {
                        name: name.to_string(),
                        attributes,
                        members,
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let interface_decl_stmt = attributes
            .clone()
            .then_ignore(text::keyword(kw::INTERFACE.text).padded())
            .then(text::ident().padded())
            .then(
                text::keyword("extends")
                    .padded()
                    .ignore_then(
                        php_name
                            .clone()
                            .or(dotted_name.clone())
                            .separated_by(just(',').padded())
                            .at_least(1)
                            .collect::<Vec<_>>(),
                    )
                    .or_not(),
            )
            .then(
                interface_member
                    .repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(just('{').padded(), just('}').padded()),
            )
            .map_with(
                |(((attributes, name), parents), members): (
                    ((Vec<AttributeDecl>, &str), Option<Vec<QualifiedName>>),
                    Vec<InterfaceMember>,
                ),
                 extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::InterfaceDecl(InterfaceDeclStmt {
                        name: name.to_string(),
                        attributes,
                        parents: parents.unwrap_or_default(),
                        members,
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let enum_case_member = attributes
            .clone()
            .then_ignore(text::keyword("case").padded())
            .then(text::ident().padded())
            .then(just('=').padded().ignore_then(expr.clone()).or_not())
            .then_ignore(terminator.clone())
            .map_with(
                |((attributes, name), value): ((Vec<AttributeDecl>, &str), Option<Expr>), extra| {
                    let span: SimpleSpan = extra.span();

                    EnumMember::Case(EnumCaseDecl {
                        name: name.to_string(),
                        attributes,
                        value,
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let enum_member = enum_case_member
            .or(trait_use_member.clone().map(|member| match member {
                ClassMember::TraitUse(name) => EnumMember::TraitUse(name),
                _ => unreachable!("trait_use_member only produces trait use members"),
            }))
            .or(method_decl.clone().map(|member| match member {
                ClassMember::Method(method) => EnumMember::Method(method),
                _ => unreachable!("method_decl only produces method members"),
            }))
            .boxed();

        let enum_decl_stmt = attributes
            .clone()
            .then_ignore(text::keyword("enum").padded())
            .then(text::ident().padded())
            .then(
                just(':')
                    .padded()
                    .ignore_then(type_expr.clone().padded())
                    .or_not(),
            )
            .then(
                text::keyword("implements")
                    .padded()
                    .ignore_then(
                        php_name
                            .clone()
                            .or(dotted_name.clone())
                            .separated_by(just(',').padded())
                            .at_least(1)
                            .collect::<Vec<_>>(),
                    )
                    .or_not(),
            )
            .then(
                enum_member
                    .repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(just('{').padded(), just('}').padded()),
            )
            .map_with(
                |((((attributes, name), backing_type), interfaces), members): (
                    (
                        ((Vec<AttributeDecl>, &str), Option<String>),
                        Option<Vec<QualifiedName>>,
                    ),
                    Vec<EnumMember>,
                ),
                 extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::EnumDecl(EnumDeclStmt {
                        name: name.to_string(),
                        attributes,
                        backing_type,
                        interfaces: interfaces.unwrap_or_default(),
                        members,
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let facet_decl_stmt = just("facet")
            .padded()
            .ignore_then(type_expr.clone().padded())
            .then_ignore(text::keyword("as").padded())
            .then_ignore(just('$').padded())
            .then(text::ident().padded())
            .then_ignore(just('{').padded())
            .then(class_member.repeated().collect::<Vec<_>>())
            .then_ignore(just('}').padded())
            .then_ignore(terminator.clone().or_not())
            .map_with(
                |((target, receiver), members): ((String, &str), Vec<ClassMember>), extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::FacetDecl(FacetDeclStmt {
                        target,
                        receiver: receiver.to_string(),
                        members,
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let type_field = text::keyword(kw::CONST.text)
            .padded()
            .to(true)
            .or_not()
            .map(|is_const| is_const.unwrap_or(false))
            .then(text::ident().padded())
            .then(just('?').padded().to(true).or_not())
            .then_ignore(just(':').padded())
            .then(type_expr.clone().padded())
            .then_ignore(just(';').padded().repeated())
            .map(
                |(((is_const, name), is_optional), ty): (((bool, &str), Option<bool>), String)| {
                    TypeField {
                        name: name.to_string(),
                        ty,
                        is_const,
                        is_optional: is_optional.unwrap_or(false),
                    }
                },
            )
            .boxed();

        let type_decl_stmt = text::keyword(kw::TYPE.text)
            .padded()
            .ignore_then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then(
                type_field
                    .repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(just('{').padded(), just('}').padded()),
            )
            .then_ignore(terminator.clone().or_not())
            .map_with(|(name, fields): (&str, Vec<TypeField>), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::TypeDecl(TypeDeclStmt {
                    name: name.to_string(),
                    fields,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let unnamed_export_stmt = text::keyword(kw::PUB.text)
            .padded()
            .ignore_then(expr.clone().padded())
            .then_ignore(terminator.clone().or_not())
            .map_with(|value, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::UnnamedExport(UnnamedExportStmt {
                    value,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let function_decl_stmt = attributes
            .clone()
            .then_ignore(just("function").padded())
            .then(text::ident().padded())
            .then_ignore(just('(').padded())
            .then(
                typed_param
                    .clone()
                    .padded()
                    .separated_by(just(',').padded())
                    .allow_trailing()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(just(')').padded())
            .then_ignore(just('{').padded())
            .then(statement.clone().repeated().collect::<Vec<_>>())
            .then_ignore(just('}').padded())
            .then_ignore(terminator.clone().or_not())
            .map_with(
                |(((attributes, name), params), body): (
                    ((Vec<AttributeDecl>, &str), Vec<TypedParam>),
                    Vec<Stmt>,
                ),
                 extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::FunctionDecl(FunctionDeclStmt {
                        name: name.to_string(),
                        attributes,
                        params,
                        return_type: None,
                        is_intrinsic: false,
                        is_generator: false,
                        body,
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let fn_decl_stmt = attributes
            .clone()
            .then_ignore(text::keyword(kw::FN.text).padded())
            .then(text::ident().padded())
            .then_ignore(just('(').padded())
            .then(
                typed_param
                    .clone()
                    .padded()
                    .separated_by(just(',').padded())
                    .allow_trailing()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(just(')').padded())
            .then(
                just(':')
                    .padded()
                    .ignore_then(type_expr.clone().padded())
                    .or_not(),
            )
            .then_ignore(just('{').padded())
            .then(statement.clone().repeated().collect::<Vec<_>>())
            .then_ignore(just('}').padded())
            .then_ignore(terminator.clone().or_not())
            .map_with(
                |((((attributes, name), params), return_type), body): (
                    (
                        ((Vec<AttributeDecl>, &str), Vec<TypedParam>),
                        Option<String>,
                    ),
                    Vec<Stmt>,
                ),
                 extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::FunctionDecl(FunctionDeclStmt {
                        name: name.to_string(),
                        attributes,
                        params,
                        return_type,
                        is_intrinsic: false,
                        is_generator: false,
                        body,
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let intrinsic_function_decl_stmt = attributes
            .clone()
            .then_ignore(text::keyword(kw::INTRINSIC.text).padded())
            .then_ignore(
                text::keyword(kw::FN.text)
                    .or(text::keyword(kw::FUNCTION.text))
                    .padded(),
            )
            .then(text::ident().padded())
            .then_ignore(just('(').padded())
            .then(
                typed_param
                    .clone()
                    .padded()
                    .separated_by(just(',').padded())
                    .allow_trailing()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(just(')').padded())
            .then(
                just(':')
                    .padded()
                    .ignore_then(type_expr.clone().padded())
                    .or_not(),
            )
            .then_ignore(terminator.clone())
            .map_with(
                |(((attributes, name), params), return_type): (
                    ((Vec<AttributeDecl>, &str), Vec<TypedParam>),
                    Option<String>,
                ),
                 extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::FunctionDecl(FunctionDeclStmt {
                        name: name.to_string(),
                        attributes,
                        params,
                        return_type,
                        is_intrinsic: true,
                        is_generator: false,
                        body: Vec::new(),
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let gen_fn_decl_stmt = attributes
            .clone()
            .then_ignore(text::keyword(kw::GEN.text).padded())
            .then_ignore(text::keyword(kw::FN.text).padded())
            .then(text::ident().padded())
            .then_ignore(just('(').padded())
            .then(
                typed_param
                    .clone()
                    .padded()
                    .separated_by(just(',').padded())
                    .allow_trailing()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(just(')').padded())
            .then(
                just(':')
                    .padded()
                    .ignore_then(type_expr.clone().padded())
                    .or_not(),
            )
            .then_ignore(just('{').padded())
            .then(statement.clone().repeated().collect::<Vec<_>>())
            .then_ignore(just('}').padded())
            .then_ignore(terminator.clone().or_not())
            .map_with(
                |((((attributes, name), params), return_type), body): (
                    (
                        ((Vec<AttributeDecl>, &str), Vec<TypedParam>),
                        Option<String>,
                    ),
                    Vec<Stmt>,
                ),
                 extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::FunctionDecl(FunctionDeclStmt {
                        name: name.to_string(),
                        attributes,
                        params,
                        return_type,
                        is_intrinsic: false,
                        is_generator: true,
                        body,
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let coalesce_assign_stmt = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just("??=").padded())
            .then(expr.clone())
            .then_ignore(terminator.clone())
            .map_with(|(name, value): (&str, Expr), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::CoalesceAssign(CoalesceAssignStmt {
                    name: name.to_string(),
                    value,
                    span: Span::new(span.start, span.end),
                })
            });

        let assign_stmt = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then(expr.clone())
            .then_ignore(terminator.clone())
            .map_with(|(name, value): (&str, Expr), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Assign(AssignStmt {
                    name: name.to_string(),
                    value,
                    span: Span::new(span.start, span.end),
                })
            });

        let list_assign_target = expr
            .clone()
            .then_ignore(just("=>").padded())
            .or_not()
            .ignore_then(just('$'))
            .ignore_then(text::ident().padded());

        let list_assign_stmt = just('[')
            .padded()
            .ignore_then(
                list_assign_target
                    .separated_by(just(',').padded())
                    .allow_trailing()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(just(']').padded())
            .then_ignore(just('=').padded())
            .then(expr.clone())
            .then_ignore(terminator.clone())
            .map_with(|(targets, value): (Vec<&str>, Expr), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::ListAssign(ListAssignStmt {
                    targets: targets.into_iter().map(str::to_string).collect(),
                    value,
                    span: Span::new(span.start, span.end),
                })
            });

        let stray_terminators = just(';').padded().repeated();
        let block = stray_terminators
            .clone()
            .ignore_then(
                statement
                    .clone()
                    .then_ignore(stray_terminators.clone())
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(stray_terminators.clone())
            .delimited_by(just('{').padded(), just('}').padded())
            .boxed();

        let declare_header = text::keyword("declare")
            .padded()
            .ignore_then(
                declare_directive
                    .separated_by(just(',').padded())
                    .at_least(1)
                    .collect::<Vec<_>>()
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .boxed();
        let declare_alternate_body_boundary = text::keyword("enddeclare").padded().ignored();
        let declare_alternate_body_statement = declare_alternate_body_boundary
            .clone()
            .not()
            .ignore_then(statement.clone())
            .boxed();
        let declare_alternate_body = just(':')
            .padded()
            .ignore_then(
                declare_alternate_body_statement
                    .then_ignore(stray_terminators.clone())
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(declare_alternate_body_boundary)
            .then_ignore(terminator.clone())
            .boxed();
        let declare_body = block
            .clone()
            .map(Some)
            .or(just(';').padded().to(None))
            .or(statement.clone().map(|statement| Some(vec![statement])))
            .boxed();
        let declare_alternate_stmt = declare_header
            .clone()
            .then(declare_alternate_body)
            .map_with(|(directives, body), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::PhpDeclare(PhpDeclareStmt {
                    directives,
                    body,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();
        let declare_stmt = declare_alternate_stmt
            .or(declare_header
                .then(declare_body)
                .then_ignore(terminator.clone().or_not())
                .map_with(|(directives, body), extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::PhpDeclare(PhpDeclareStmt {
                        directives,
                        body: body.unwrap_or_default(),
                        span: Span::new(span.start, span.end),
                    })
                }))
            .boxed();

        let php_exit_value = expr
            .clone()
            .or_not()
            .delimited_by(just('(').padded(), just(')').padded())
            .or(expr.clone().map(Some))
            .or_not()
            .map(Option::flatten)
            .boxed();
        let php_exit_stmt = text::keyword("exit")
            .to(PhpExitKind::Exit)
            .or(text::keyword("die").to(PhpExitKind::Die))
            .padded()
            .then(php_exit_value)
            .then_ignore(terminator.clone())
            .map_with(|(kind, value), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::PhpExit(PhpExitStmt {
                    kind,
                    value,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let let_binding = just('$').ignore_then(text::ident().padded()).then(
            just(':')
                .padded()
                .ignore_then(type_expr.clone().padded())
                .or_not(),
        );

        let let_stmt = text::keyword(kw::LET.text)
            .padded()
            .ignore_then(let_binding.clone())
            .then_ignore(just('=').padded())
            .then(expr.clone())
            .then_ignore(terminator.clone())
            .map_with(
                |((name, ty), value): ((&str, Option<String>), Expr), extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::Let(LetStmt {
                        name: name.to_string(),
                        ty,
                        value,
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let let_join_run_block_stmt = text::keyword(kw::LET.text)
            .padded()
            .ignore_then(let_binding.clone())
            .then_ignore(just('=').padded())
            .then_ignore(text::keyword(kw::JOIN.text).padded())
            .then_ignore(text::keyword(kw::RUN.text).padded())
            .then(block.clone())
            .then_ignore(terminator.clone().or_not())
            .map_with(
                |((name, ty), body): ((&str, Option<String>), Vec<Stmt>), extra| {
                    let span: SimpleSpan = extra.span();
                    let run_span = Span::new(span.start, span.end);

                    Stmt::Let(LetStmt {
                        name: name.to_string(),
                        ty,
                        value: Expr::Join(JoinExpr {
                            handle: Box::new(Expr::Run(RunExpr::Block {
                                body,
                                span: run_span,
                            })),
                            span: run_span,
                        }),
                        span: run_span,
                    })
                },
            )
            .boxed();

        let let_loop_block_stmt = text::keyword(kw::LET.text)
            .padded()
            .ignore_then(let_binding.clone())
            .then_ignore(just('=').padded())
            .then_ignore(text::keyword(kw::LOOP.text).padded())
            .then(block.clone())
            .then_ignore(terminator.clone().or_not())
            .map_with(
                |((name, ty), body): ((&str, Option<String>), Vec<Stmt>), extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::Let(LetStmt {
                        name: name.to_string(),
                        ty,
                        value: Expr::Loop(LoopExpr {
                            body,
                            span: Span::new(span.start, span.end),
                        }),
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let let_run_block_stmt = text::keyword(kw::LET.text)
            .padded()
            .ignore_then(let_binding.clone())
            .then_ignore(just('=').padded())
            .then_ignore(text::keyword(kw::RUN.text).padded())
            .then(block.clone())
            .then_ignore(terminator.clone().or_not())
            .map_with(
                |((name, ty), body): ((&str, Option<String>), Vec<Stmt>), extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::Let(LetStmt {
                        name: name.to_string(),
                        ty,
                        value: Expr::Run(RunExpr::Block {
                            body,
                            span: Span::new(span.start, span.end),
                        }),
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let run_group_entries = block
            .clone()
            .padded()
            .separated_by(just(',').padded())
            .allow_trailing()
            .at_least(1)
            .collect::<Vec<_>>()
            .delimited_by(just('[').padded(), just(']').padded())
            .boxed();

        let let_run_group_stmt = text::keyword(kw::LET.text)
            .padded()
            .ignore_then(let_binding.clone())
            .then_ignore(just('=').padded())
            .then_ignore(text::keyword(kw::RUN.text).padded())
            .then(run_group_entries.clone())
            .then_ignore(terminator.clone().or_not())
            .map_with(
                |((name, ty), entries): ((&str, Option<String>), Vec<Vec<Stmt>>), extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::Let(LetStmt {
                        name: name.to_string(),
                        ty,
                        value: Expr::Run(RunExpr::Group {
                            entries,
                            span: Span::new(span.start, span.end),
                        }),
                        span: Span::new(span.start, span.end),
                    })
                },
            )
            .boxed();

        let run_block_stmt = text::keyword(kw::RUN.text)
            .padded()
            .ignore_then(block.clone())
            .then_ignore(terminator.clone().or_not())
            .map_with(|body, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Expr(echo_ast::ExprStmt {
                    expr: Expr::Run(RunExpr::Block {
                        body,
                        span: Span::new(span.start, span.end),
                    }),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let append_stmt = expr
            .clone()
            .padded()
            .then_ignore(just('[').padded())
            .then_ignore(just(']').padded())
            .then_ignore(just('=').padded())
            .then(expr.clone())
            .then_ignore(terminator.clone())
            .map_with(|(target, value): (Expr, Expr), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Append(AppendStmt {
                    target,
                    value,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let loop_stmt = text::keyword(kw::LOOP.text)
            .padded()
            .ignore_then(block.clone())
            .then_ignore(terminator.clone().or_not())
            .map_with(|body, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Loop(LoopStmt {
                    body,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let condition_expr = expr
            .clone()
            .delimited_by(just('(').padded(), just(')').padded())
            .or(expr.clone())
            .boxed();

        let while_header = text::keyword("while")
            .padded()
            .ignore_then(condition_expr.clone())
            .boxed();
        let while_alternate_body_boundary = text::keyword("endwhile").padded().ignored();
        let while_alternate_body_statement = while_alternate_body_boundary
            .clone()
            .not()
            .ignore_then(statement.clone())
            .boxed();
        let while_alternate_body = just(':')
            .padded()
            .ignore_then(
                while_alternate_body_statement
                    .then_ignore(stray_terminators.clone())
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(while_alternate_body_boundary)
            .then_ignore(terminator.clone());
        let while_alternate_stmt = while_header
            .clone()
            .then(while_alternate_body)
            .map_with(|(condition, body), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::While(WhileStmt {
                    condition,
                    body,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();
        let while_stmt = while_alternate_stmt
            .or(while_header
                .then(block.clone())
                .then_ignore(terminator.clone().or_not())
                .map_with(|(condition, body), extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::While(WhileStmt {
                        condition,
                        body,
                        span: Span::new(span.start, span.end),
                    })
                }))
            .boxed();

        let do_while_stmt = just("do")
            .padded()
            .ignore_then(block.clone())
            .then_ignore(text::keyword("while").padded())
            .then(condition_expr)
            .then_ignore(terminator.clone())
            .map_with(|(body, condition), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::DoWhile(DoWhileStmt {
                    body,
                    condition,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let for_exprs = expr
            .clone()
            .padded()
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect::<Vec<_>>()
            .boxed();
        let for_header = text::keyword("for")
            .padded()
            .ignore_then(just('(').padded())
            .ignore_then(for_exprs.clone())
            .then_ignore(just(';').padded())
            .then(for_exprs.clone())
            .then_ignore(just(';').padded())
            .then(for_exprs)
            .then_ignore(just(')').padded())
            .boxed();
        let for_alternate_body_boundary = text::keyword("endfor").padded().ignored();
        let for_alternate_body_statement = for_alternate_body_boundary
            .clone()
            .not()
            .ignore_then(statement.clone())
            .boxed();
        let for_alternate_body = just(':')
            .padded()
            .ignore_then(
                for_alternate_body_statement
                    .then_ignore(stray_terminators.clone())
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(for_alternate_body_boundary)
            .then_ignore(terminator.clone());
        let for_alternate_stmt = for_header
            .clone()
            .then(for_alternate_body)
            .map_with(|(((init, conditions), increments), body), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::For(ForStmt {
                    init,
                    conditions,
                    increments,
                    body,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();
        let for_stmt = for_alternate_stmt
            .or(for_header
                .then(block.clone())
                .then_ignore(terminator.clone().or_not())
                .map_with(|(((init, conditions), increments), body), extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::For(ForStmt {
                        init,
                        conditions,
                        increments,
                        body,
                        span: Span::new(span.start, span.end),
                    })
                }))
            .boxed();

        let foreach_value = just('&')
            .padded()
            .or_not()
            .then(just('$').ignore_then(text::ident().padded()))
            .map(|(by_ref, value): (Option<char>, &str)| {
                (None, value.to_string(), by_ref.is_some())
            });
        let foreach_key_value = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just("=>").padded())
            .then(
                just('&')
                    .padded()
                    .or_not()
                    .then(just('$').ignore_then(text::ident().padded())),
            )
            .map(|(key, (by_ref, value)): (&str, (Option<char>, &str))| {
                (Some(key.to_string()), value.to_string(), by_ref.is_some())
            });
        let foreach_header = text::keyword("foreach")
            .padded()
            .ignore_then(just('(').padded())
            .ignore_then(expr.clone())
            .then_ignore(text::keyword("as").padded())
            .then(foreach_key_value.or(foreach_value))
            .then_ignore(just(')').padded())
            .boxed();
        let foreach_alternate_body_boundary = text::keyword("endforeach").padded().ignored();
        let foreach_alternate_body_statement = foreach_alternate_body_boundary
            .clone()
            .not()
            .ignore_then(statement.clone())
            .boxed();
        let foreach_alternate_body = just(':')
            .padded()
            .ignore_then(
                foreach_alternate_body_statement
                    .then_ignore(stray_terminators.clone())
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(foreach_alternate_body_boundary)
            .then_ignore(terminator.clone());
        let foreach_alternate_stmt = foreach_header
            .clone()
            .then(foreach_alternate_body)
            .map_with(|((iterable, (key, value, value_by_ref)), body), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Foreach(ForeachStmt {
                    iterable,
                    key,
                    value,
                    value_by_ref,
                    body,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();
        let foreach_stmt = foreach_alternate_stmt
            .or(foreach_header
                .then(block.clone())
                .then_ignore(terminator.clone().or_not())
                .map_with(|((iterable, (key, value, value_by_ref)), body), extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::Foreach(ForeachStmt {
                        iterable,
                        key,
                        value,
                        value_by_ref,
                        body,
                        span: Span::new(span.start, span.end),
                    })
                }))
            .boxed();

        let switch_case_separator = just(':').or(just(';')).padded();
        let switch_case_boundary = text::keyword("case")
            .padded()
            .ignored()
            .or(text::keyword("default").padded().ignored())
            .or(text::keyword("endswitch").padded().ignored());
        let switch_case_body_statement = switch_case_boundary
            .not()
            .ignore_then(statement.clone())
            .boxed();
        let switch_case = text::keyword("case")
            .padded()
            .ignore_then(expr.clone())
            .map(Some)
            .or(text::keyword("default").padded().to(None))
            .then_ignore(switch_case_separator)
            .then(switch_case_body_statement.repeated().collect::<Vec<_>>())
            .map_with(|(condition, body), extra| {
                let span: SimpleSpan = extra.span();
                SwitchCase {
                    condition,
                    body,
                    span: Span::new(span.start, span.end),
                }
            })
            .boxed();
        let switch_braced_body = switch_case
            .clone()
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(just('{').padded(), just('}').padded());
        let switch_alternate_body_boundary = text::keyword("endswitch").padded().ignored();
        let switch_alternate_case = switch_alternate_body_boundary
            .clone()
            .not()
            .ignore_then(switch_case.clone())
            .boxed();
        let switch_alternate_body = just(':')
            .padded()
            .ignore_then(switch_alternate_case.repeated().collect::<Vec<_>>())
            .then_ignore(switch_alternate_body_boundary)
            .then_ignore(terminator.clone());
        let switch_header = text::keyword("switch")
            .padded()
            .ignore_then(just('(').padded())
            .ignore_then(expr.clone())
            .then_ignore(just(')').padded())
            .boxed();
        let switch_alternate_stmt = switch_header
            .clone()
            .then(switch_alternate_body)
            .map_with(|(expr, cases), extra| {
                let span: SimpleSpan = extra.span();
                Stmt::Switch(SwitchStmt {
                    expr,
                    cases,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();
        let switch_stmt = switch_alternate_stmt
            .or(switch_header
                .then(switch_braced_body)
                .then_ignore(terminator.clone().or_not())
                .map_with(|(expr, cases), extra| {
                    let span: SimpleSpan = extra.span();
                    Stmt::Switch(SwitchStmt {
                        expr,
                        cases,
                        span: Span::new(span.start, span.end),
                    })
                }))
            .boxed();

        let elseif_clause = text::keyword(kw::ELSEIF.text)
            .padded()
            .or(text::keyword(kw::ELSE.text)
                .padded()
                .then_ignore(text::keyword(kw::IF.text).padded()))
            .ignore_then(expr.clone())
            .then(block.clone())
            .map_with(|(condition, body), extra| {
                let span: SimpleSpan = extra.span();
                ElseIfClause {
                    condition,
                    body,
                    span: Span::new(span.start, span.end),
                }
            })
            .boxed();

        let if_alternate_boundary = text::keyword(kw::ELSEIF.text)
            .padded()
            .ignored()
            .or(text::keyword(kw::ELSE.text).padded().ignored())
            .or(text::keyword("endif").padded().ignored());
        let if_alternate_body_statement = if_alternate_boundary
            .clone()
            .not()
            .ignore_then(statement.clone())
            .boxed();
        let if_alternate_body = if_alternate_body_statement
            .clone()
            .then_ignore(stray_terminators.clone())
            .repeated()
            .collect::<Vec<_>>();
        let alternate_elseif_clause = text::keyword(kw::ELSEIF.text)
            .padded()
            .ignore_then(expr.clone())
            .then_ignore(just(':').padded())
            .then(if_alternate_body.clone())
            .map_with(|(condition, body), extra| {
                let span: SimpleSpan = extra.span();
                ElseIfClause {
                    condition,
                    body,
                    span: Span::new(span.start, span.end),
                }
            })
            .boxed();
        let alternate_else_clause = text::keyword(kw::ELSE.text)
            .padded()
            .ignore_then(just(':').padded())
            .ignore_then(if_alternate_body.clone())
            .boxed();
        let alternate_if_stmt = text::keyword(kw::IF.text)
            .padded()
            .ignore_then(expr.clone())
            .then_ignore(just(':').padded())
            .then(if_alternate_body)
            .then(alternate_elseif_clause.repeated().collect::<Vec<_>>())
            .then(alternate_else_clause.or_not())
            .then_ignore(text::keyword("endif").padded())
            .then_ignore(terminator.clone())
            .map_with(|(((condition, body), elseif_clauses), else_body), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::If(IfStmt {
                    condition,
                    body,
                    elseif_clauses,
                    else_body: else_body.unwrap_or_default(),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let else_clause = text::keyword(kw::ELSE.text)
            .padded()
            .ignore_then(block.clone())
            .boxed();

        let braced_if_stmt = text::keyword(kw::IF.text)
            .padded()
            .ignore_then(expr.clone())
            .then(block.clone())
            .then(elseif_clause.repeated().collect::<Vec<_>>())
            .then(else_clause.or_not())
            .then_ignore(terminator.clone().or_not())
            .map_with(|(((condition, body), elseif_clauses), else_body), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::If(IfStmt {
                    condition,
                    body,
                    elseif_clauses,
                    else_body: else_body.unwrap_or_default(),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();
        let if_stmt = alternate_if_stmt.or(braced_if_stmt).boxed();

        let catch_type = just('\\')
            .or_not()
            .ignore_then(
                text::ident()
                    .separated_by(just('\\'))
                    .at_least(1)
                    .collect::<Vec<_>>(),
            )
            .map(|parts| QualifiedName::new(parts.into_iter().map(str::to_string).collect()))
            .boxed();

        let catch_clause = just("catch")
            .padded()
            .ignore_then(
                catch_type
                    .clone()
                    .separated_by(just('|').padded())
                    .at_least(1)
                    .collect::<Vec<_>>()
                    .padded()
                    .then(just('$').ignore_then(text::ident().padded()).or_not())
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .then(block.clone())
            .map_with(|((types, variable), body), extra| {
                let span: SimpleSpan = extra.span();
                CatchClause {
                    types,
                    variable: variable.map(str::to_string),
                    body,
                    span: Span::new(span.start, span.end),
                }
            })
            .boxed();

        let try_stmt = just("try")
            .padded()
            .ignore_then(block.clone())
            .then_ignore(terminator.clone().repeated())
            .then(catch_clause.repeated().collect::<Vec<_>>())
            .then(
                text::keyword("finally")
                    .padded()
                    .ignore_then(block.clone())
                    .or_not(),
            )
            .then_ignore(terminator.clone().or_not())
            .try_map(|((body, catches), finally_body), span: SimpleSpan| {
                if catches.is_empty() && finally_body.is_none() {
                    return Err(Rich::custom(
                        span,
                        "expected catch or finally after try block",
                    ));
                }

                Ok((body, catches, finally_body))
            })
            .map_with(|(body, catches, finally_body), extra| {
                let span: SimpleSpan = extra.span();
                Stmt::Try(TryStmt {
                    body,
                    catches,
                    finally_body: finally_body.unwrap_or_default(),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let break_stmt = text::keyword(kw::BREAK.text)
            .padded()
            .ignore_then(expr.clone().padded().or_not())
            .then_ignore(terminator.clone())
            .map_with(|value, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Break(BreakStmt {
                    value,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let continue_stmt = text::keyword("continue")
            .padded()
            .ignore_then(expr.clone().padded().or_not())
            .then_ignore(terminator.clone())
            .map_with(|value, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Continue(ContinueStmt {
                    value,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let assign_defer_block_stmt = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then_ignore(text::keyword(kw::DEFER.text).padded())
            .then(block.clone())
            .then_ignore(terminator.clone())
            .map_with(|(name, body): (&str, Vec<Stmt>), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Assign(AssignStmt {
                    name: name.to_string(),
                    value: Expr::Defer(DeferExpr {
                        body,
                        span: Span::new(span.start, span.end),
                    }),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let assign_run_block_stmt = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then_ignore(text::keyword(kw::RUN.text).padded())
            .then(block.clone())
            .then_ignore(terminator.clone())
            .map_with(|(name, body): (&str, Vec<Stmt>), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Assign(AssignStmt {
                    name: name.to_string(),
                    value: Expr::Run(RunExpr::Block {
                        body,
                        span: Span::new(span.start, span.end),
                    }),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let assign_run_group_stmt = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then_ignore(text::keyword(kw::RUN.text).padded())
            .then(run_group_entries)
            .then_ignore(terminator.clone())
            .map_with(|(name, entries): (&str, Vec<Vec<Stmt>>), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Assign(AssignStmt {
                    name: name.to_string(),
                    value: Expr::Run(RunExpr::Group {
                        entries,
                        span: Span::new(span.start, span.end),
                    }),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let assign_fork_block_stmt = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then_ignore(text::keyword(kw::FORK.text).padded())
            .then(block.clone())
            .then_ignore(terminator.clone())
            .map_with(|(name, body): (&str, Vec<Stmt>), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Assign(AssignStmt {
                    name: name.to_string(),
                    value: Expr::Fork(ForkExpr::Block {
                        body,
                        span: Span::new(span.start, span.end),
                    }),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let assign_ref_stmt = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then_ignore(just('&').padded())
            .then_ignore(just('$'))
            .then(text::ident().padded())
            .then_ignore(terminator.clone())
            .map_with(|(name, target): (&str, &str), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::AssignRef(AssignRefStmt {
                    name: name.to_string(),
                    target: target.to_string(),
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let post_increment_stmt = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just("++").padded())
            .then_ignore(terminator.clone())
            .map_with(|name: &str, extra| {
                let span: SimpleSpan = extra.span();
                let span = Span::new(span.start, span.end);
                let variable = Expr::Variable(VariableExpr {
                    name: name.to_string(),
                    span,
                });
                let one = Expr::Number(NumberLiteral {
                    value: "1".to_string(),
                    span,
                });

                Stmt::Expr(echo_ast::ExprStmt {
                    expr: Expr::Assign(Box::new(AssignExpr {
                        name: name.to_string(),
                        value: Expr::Binary(Box::new(BinaryExpr {
                            left: variable,
                            op: BinaryOp::Add,
                            right: one,
                            span,
                        })),
                        span,
                    })),
                    span,
                })
            })
            .boxed();

        let expr_stmt = expr
            .clone()
            .then_ignore(terminator.clone())
            .map_with(|expr, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Expr(echo_ast::ExprStmt {
                    expr,
                    span: Span::new(span.start, span.end),
                })
            })
            .boxed();

        let declaration_stmt = class_decl_stmt
            .or(interface_decl_stmt)
            .or(trait_decl_stmt)
            .or(enum_decl_stmt)
            .or(facet_decl_stmt)
            .or(gen_fn_decl_stmt)
            .or(fn_decl_stmt)
            .or(function_decl_stmt)
            .or(intrinsic_function_decl_stmt)
            .or(type_decl_stmt)
            .or(unnamed_export_stmt)
            .then_ignore(terminator.clone().or_not())
            .boxed();

        let module_import_stmt = module_stmt
            .or(echo_std_use_stmt)
            .or(namespace_stmt)
            .or(use_stmt)
            .or(dotted_use_stmt)
            .or(import_stmt)
            .boxed();

        let flow_stmt = try_stmt
            .clone()
            .or(declare_stmt)
            .or(php_exit_stmt)
            .or(short_echo_stmt)
            .or(echo_stmt)
            .or(return_stmt)
            .or(throw_stmt)
            .or(yield_stmt)
            .or(goto_stmt)
            .or(label_stmt)
            .or(global_stmt)
            .or(static_var_stmt)
            .or(loop_stmt)
            .or(do_while_stmt.clone())
            .or(while_stmt)
            .or(for_stmt)
            .or(foreach_stmt)
            .or(switch_stmt)
            .or(if_stmt)
            .or(break_stmt)
            .or(continue_stmt)
            .boxed();

        let binding_stmt = let_join_run_block_stmt
            .or(let_loop_block_stmt)
            .or(let_run_group_stmt)
            .or(let_run_block_stmt)
            .or(let_stmt)
            .or(assign_defer_block_stmt)
            .or(assign_run_group_stmt)
            .or(assign_run_block_stmt)
            .or(assign_fork_block_stmt)
            .or(run_block_stmt)
            .or(append_stmt)
            .or(assign_ref_stmt)
            .or(post_increment_stmt)
            .or(list_assign_stmt)
            .or(coalesce_assign_stmt)
            .or(assign_stmt)
            .boxed();

        let call_or_expr_stmt = dynamic_function_call_stmt
            .or(function_call_stmt)
            .or(expr_stmt)
            .boxed();

        flow_stmt
            .or(declaration_stmt)
            .or(module_import_stmt)
            .or(binding_stmt)
            .or(call_or_expr_stmt)
            .boxed()
    });

    let php_tag_boundary = just("<?php")
        .ignored()
        .or(just("<?PHP").ignored())
        .or(just("<?=").ignored())
        .boxed();

    let inline_html_stmt = php_tag_boundary
        .clone()
        .not()
        .ignore_then(any())
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map_with(|text, extra| {
            let span: SimpleSpan = extra.span();
            Stmt::PhpInlineHtml(PhpInlineHtmlStmt {
                text,
                span: Span::new(span.start, span.end),
            })
        })
        .boxed();

    let non_empty_php_statement_block = open_php
        .then(statement.clone().repeated().at_least(1).collect::<Vec<_>>())
        .then_ignore(just("?>").padded().or_not())
        .map(|(open_tag, statements)| (Some(open_tag), statements))
        .boxed();

    let empty_php_statement_block = open_php
        .then_ignore(just("?>").padded())
        .map(|open_tag| (Some(open_tag), Vec::new()))
        .boxed();

    let php_statement_block = non_empty_php_statement_block
        .or(empty_php_statement_block)
        .boxed();

    let echo_statement_chunk = statement
        .clone()
        .map(|statement| (None, vec![statement]))
        .boxed();

    let inline_html_chunk = inline_html_stmt
        .map(|statement| (None, vec![statement]))
        .boxed();

    let program_chunk = php_statement_block
        .or(echo_statement_chunk)
        .or(inline_html_chunk)
        .boxed();

    text::whitespace()
        .ignored()
        .or_not()
        .ignore_then(program_chunk.repeated().at_least(1).collect::<Vec<_>>())
        .then_ignore(end())
        .map_with(|chunks, extra| {
            let span: SimpleSpan = extra.span();
            let open_tag = chunks.iter().find_map(|(open_tag, _)| *open_tag);
            let statements = chunks
                .into_iter()
                .flat_map(|(_, statements)| statements)
                .collect();

            Program {
                open_tag,
                statements,
                source_id: None,
                source_dir: None,
                span: Span::new(span.start, span.end),
            }
        })
}

#[cfg(test)]
mod source_file_tests {
    use std::path::PathBuf;

    use echo_source::{SourceFile, SourceId, Span};

    use super::*;

    #[test]
    fn parse_source_file_records_program_source_id() {
        let source_id = SourceId::new(42);
        let source = SourceFile::new(PathBuf::from("app.echo"), "echo \"ok\"".to_string())
            .with_id(source_id);

        let program = parse_source_file(&source).expect("source file should parse");

        assert_eq!(program.source_id, Some(source_id));
        assert_eq!(
            program.source_span(Span::new(0, 4)),
            Some(echo_source::SourceSpan::new(source_id, Span::new(0, 4)))
        );
    }

    #[test]
    fn parse_source_file_attaches_source_id_to_diagnostics() {
        let source_id = SourceId::new(9);
        let source =
            SourceFile::new(PathBuf::from("broken.echo"), "!".to_string()).with_id(source_id);

        let diagnostics = parse_source_file(&source).expect_err("source file should fail");

        assert_eq!(diagnostics[0].source_span().unwrap().source_id, source_id);
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
