use chumsky::prelude::*;
use chumsky::span::SimpleSpan;
use echo_ast::{
    AppendStmt, ArrayElement, ArrayExpr, AssignExpr, AssignRefStmt, AssignStmt, BinaryExpr,
    BinaryOp, BoolLiteral, BreakStmt, ClassDeclStmt, ClassMember, DeferExpr,
    DynamicFunctionCallStmt, EchoStmt, Expr, FieldExpr, ForkExpr, FunctionCallExpr,
    FunctionCallStmt, FunctionDeclStmt, IfStmt, ImportSource, ImportStmt, IndexExpr, JoinExpr,
    LetStmt, ListExpr, LoopExpr, LoopStmt, MagicConstantExpr, MagicConstantKind, MethodCallExpr,
    MethodDecl, NamespaceSource, NamespaceStmt, NullLiteral, NumberLiteral, ObjectExpr,
    ObjectField, Program, QualifiedName, RequireExpr, RequireKind, ReturnStmt, RunExpr, SpawnExpr,
    StaticCallExpr, Stmt, StringLiteral, TypeDeclStmt, TypeField, TypedParam, UnaryExpr, UnaryOp,
    UseStmt, VariableExpr, YieldStmt,
};
use echo_diagnostics::Diagnostic;
use echo_source::{SourceMode, Span};

pub fn parse(source: &str) -> Result<Program, Vec<Diagnostic>> {
    parse_with_mode(source, SourceMode::Echo)
}

pub fn parse_with_mode(source: &str, mode: SourceMode) -> Result<Program, Vec<Diagnostic>> {
    parse_with_validation(source, ValidationMode::from_source_mode(mode))
}

pub fn parse_trusted_std(source: &str) -> Result<Program, Vec<Diagnostic>> {
    parse_with_validation(source, ValidationMode::TrustedStd)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidationMode {
    Echo,
    Strict,
    TrustedStd,
}

impl ValidationMode {
    const fn from_source_mode(mode: SourceMode) -> Self {
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

fn parse_with_validation(source: &str, mode: ValidationMode) -> Result<Program, Vec<Diagnostic>> {
    // For now, run the Logos lexer first so lexer errors are caught.
    // The Chumsky parser below still parses the source text directly.
    echo_lexer::lex(source)?;

    let source = normalize_heredoc_literals(source);
    let source = virtualize_statement_terminators(&strip_comments_preserving_spans(&source));

    let program = parser().parse(&source).into_result().map_err(|errors| {
        errors
            .into_iter()
            .map(|error| {
                let span = error.span();
                Diagnostic::new(error.to_string(), Span::new(span.start, span.end))
            })
            .collect::<Vec<_>>()
    })?;

    validate_mode(&program, mode)?;

    Ok(program)
}

fn validate_mode(program: &Program, mode: ValidationMode) -> Result<(), Vec<Diagnostic>> {
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

fn strip_comments_preserving_spans(source: &str) -> String {
    let mut output = source.as_bytes().to_vec();
    let mut index = 0;
    let bytes = source.as_bytes();

    while index < bytes.len() {
        match bytes[index] {
            b'"' | b'\'' => {
                let quote = bytes[index];
                index += 1;
                while index < bytes.len() {
                    match bytes[index] {
                        b'\\' => index = (index + 2).min(bytes.len()),
                        byte if byte == quote => {
                            index += 1;
                            break;
                        }
                        _ => index += 1,
                    }
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                while index < bytes.len() && bytes[index] != b'\n' && bytes[index] != b'\r' {
                    output[index] = b' ';
                    index += 1;
                }
            }
            b'#' => {
                while index < bytes.len() && bytes[index] != b'\n' && bytes[index] != b'\r' {
                    output[index] = b' ';
                    index += 1;
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                // PHP C-style comments end at the first `*/`; they do not nest.
                // Source: https://www.php.net/manual/en/language.basic-syntax.comments.php
                output[index] = b' ';
                output[index + 1] = b' ';
                index += 2;

                while index < bytes.len() {
                    if bytes[index] == b'*' && bytes.get(index + 1) == Some(&b'/') {
                        output[index] = b' ';
                        output[index + 1] = b' ';
                        index += 2;
                        break;
                    }

                    if bytes[index] != b'\n' && bytes[index] != b'\r' {
                        output[index] = b' ';
                    }
                    index += 1;
                }
            }
            _ => index += 1,
        }
    }

    String::from_utf8(output).expect("comment stripping preserves UTF-8")
}

fn virtualize_statement_terminators(source: &str) -> String {
    let mut output = source.as_bytes().to_vec();
    let mut line_start = 0;
    let mut index = 0;
    let bytes = source.as_bytes();
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'"' | b'\'' => {
                let quote = bytes[index];
                index += 1;
                while index < bytes.len() {
                    match bytes[index] {
                        b'\\' => index = (index + 2).min(bytes.len()),
                        byte if byte == quote => {
                            index += 1;
                            break;
                        }
                        _ => index += 1,
                    }
                }
            }
            b'}' => {
                virtualize_before_close_brace(&mut output, bytes, index);
                index += 1;
            }
            b'(' => {
                paren_depth += 1;
                index += 1;
            }
            b')' => {
                paren_depth = paren_depth.saturating_sub(1);
                index += 1;
            }
            b'[' => {
                bracket_depth += 1;
                index += 1;
            }
            b']' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                index += 1;
            }
            b'\n' => {
                let next = next_significant_byte(bytes, index + 1);
                if paren_depth == 0
                    && bracket_depth == 0
                    && should_end_statement(&source[line_start..index], next)
                {
                    output[index] = b';';
                }
                line_start = index + 1;
                index += 1;
            }
            b'\r' => {
                if bytes.get(index + 1) == Some(&b'\n') {
                    output[index] = b' ';
                    let next = next_significant_byte(bytes, index + 2);
                    if paren_depth == 0
                        && bracket_depth == 0
                        && should_end_statement(&source[line_start..index], next)
                    {
                        output[index + 1] = b';';
                    }
                    index += 2;
                } else {
                    let next = next_significant_byte(bytes, index + 1);
                    if paren_depth == 0
                        && bracket_depth == 0
                        && should_end_statement(&source[line_start..index], next)
                    {
                        output[index] = b';';
                    }
                    index += 1;
                }
                line_start = index;
            }
            _ => index += 1,
        }
    }

    String::from_utf8(output).expect("virtual semicolons preserve UTF-8")
}

fn should_end_statement(line: &str, next: Option<u8>) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("<?php") {
        return false;
    }

    if matches!(
        next,
        Some(b'.' | b'+' | b'-' | b'*' | b'/' | b',' | b'{' | b')' | b']')
    ) {
        return false;
    }

    if trimmed.starts_with("function ") {
        return false;
    }

    if matches!(
        trimmed.as_bytes().last(),
        Some(b'=' | b'.' | b'+' | b'-' | b'*' | b'/' | b',' | b'(' | b'[' | b'{')
    ) {
        return false;
    }

    matches!(
        trimmed.as_bytes().last(),
        Some(b')' | b']' | b'}' | b'"')
            | Some(b'0'..=b'9')
            | Some(b'a'..=b'z')
            | Some(b'A'..=b'Z')
            | Some(b'_')
    )
}

fn next_significant_byte(source: &[u8], start: usize) -> Option<u8> {
    source[start..]
        .iter()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
}

fn virtualize_before_close_brace(output: &mut [u8], source: &[u8], close_brace: usize) {
    let Some(previous) = previous_significant_byte(source, close_brace) else {
        return;
    };

    if !matches!(previous, b')' | b']' | b'"' | b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_') {
        return;
    }

    for index in (0..close_brace).rev() {
        if !source[index].is_ascii_whitespace() {
            return;
        }

        output[index] = b';';
        return;
    }
}

fn previous_significant_byte(source: &[u8], end: usize) -> Option<u8> {
    source[..end]
        .iter()
        .rev()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
}

fn parser<'src>() -> impl Parser<'src, &'src str, Program, extra::Err<Rich<'src, char>>> {
    let open_php = just("<?php")
        .or(just("<?PHP"))
        .map_with(|_, extra| {
            let span: SimpleSpan = extra.span();
            Span::new(span.start, span.end)
        })
        .padded()
        .or_not();

    let expr = recursive(|expr| {
        let null = text::keyword("null")
            .or(text::keyword("NULL"))
            .map_with(|_, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Null(NullLiteral {
                    span: Span::new(span.start, span.end),
                })
            });

        let bool_literal = text::keyword("true")
            .or(text::keyword("TRUE"))
            .to(true)
            .or(text::keyword("false").or(text::keyword("FALSE")).to(false))
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

        let string = double_quoted_string.or(single_quoted_string);

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
            });

        let variable = just('$')
            .ignore_then(text::ident())
            .map_with(|name: &str, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Variable(VariableExpr {
                    name: name.to_string(),
                    span: Span::new(span.start, span.end),
                })
            });

        let magic_dir = just("__DIR__").map_with(|_, extra| {
            let span: SimpleSpan = extra.span();

            Expr::MagicConstant(MagicConstantExpr {
                kind: MagicConstantKind::Dir,
                span: Span::new(span.start, span.end),
            })
        });

        let args = expr
            .clone()
            .padded()
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect::<Vec<_>>();

        let function_name = text::ident()
            .separated_by(just('.'))
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|parts| parts.join("."));

        let function_call_expr = function_name
            .clone()
            .padded()
            .then_ignore(just('(').padded())
            .then(args.clone())
            .then_ignore(just(')').padded())
            .map_with(|(name, args): (String, Vec<Expr>), extra| {
                let span: SimpleSpan = extra.span();

                Expr::FunctionCall(FunctionCallExpr {
                    name,
                    args,
                    span: Span::new(span.start, span.end),
                })
            });

        let static_call_expr = text::ident()
            .separated_by(just('\\'))
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|parts| QualifiedName::new(parts.into_iter().map(str::to_string).collect()))
            .then_ignore(just("::").padded())
            .then(text::ident().padded())
            .then_ignore(just('(').padded())
            .then(args.clone())
            .then_ignore(just(')').padded())
            .map_with(
                |((class_name, method), args): ((QualifiedName, &str), Vec<Expr>), extra| {
                    let span: SimpleSpan = extra.span();

                    Expr::StaticCall(StaticCallExpr {
                        class_name,
                        method: method.to_string(),
                        args,
                        span: Span::new(span.start, span.end),
                    })
                },
            );

        let run_expr = text::keyword("run")
            .padded()
            .ignore_then(expr.clone())
            .map_with(|task, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Run(RunExpr::Task {
                    expr: Box::new(task),
                    span: Span::new(span.start, span.end),
                })
            });

        let spawn_expr = text::keyword("spawn")
            .padded()
            .ignore_then(expr.clone())
            .map_with(|command, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Spawn(SpawnExpr {
                    command: Box::new(command),
                    span: Span::new(span.start, span.end),
                })
            });

        let fork_expr = text::keyword("fork")
            .padded()
            .ignore_then(expr.clone())
            .map_with(|task, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Fork(ForkExpr::Task {
                    expr: Box::new(task),
                    span: Span::new(span.start, span.end),
                })
            });

        let join_expr = text::keyword("join")
            .padded()
            .ignore_then(expr.clone())
            .map_with(|handle, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Join(JoinExpr {
                    handle: Box::new(handle),
                    span: Span::new(span.start, span.end),
                })
            });

        let require_expr = text::keyword("require_once")
            .to(RequireKind::RequireOnce)
            .or(text::keyword("require").to(RequireKind::Require))
            .padded()
            .then(expr.clone())
            .map_with(|(kind, path), extra| {
                let span: SimpleSpan = extra.span();

                Expr::Require(Box::new(RequireExpr {
                    kind,
                    path,
                    span: Span::new(span.start, span.end),
                }))
            });

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
            });

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
            });

        let array_expr = array_element
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
            });

        let object_field = text::ident()
            .padded()
            .then_ignore(just(':').padded())
            .then(expr.clone().padded())
            .then_ignore(just(';').padded().repeated())
            .map(|(name, value): (&str, Expr)| ObjectField {
                name: name.to_string(),
                value,
            });

        let structural_object_expr = object_field
            .clone()
            .separated_by(just(',').padded().or(just(';').padded()))
            .allow_trailing()
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
            });

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
            });

        let parenthesized = expr
            .clone()
            .delimited_by(just('(').padded(), just(')').padded());

        let atom = run_expr
            .or(fork_expr)
            .or(spawn_expr)
            .or(join_expr)
            .or(require_expr)
            .or(parenthesized)
            .or(structural_object_expr)
            .or(object_expr)
            .or(static_call_expr)
            .or(function_call_expr)
            .or(variable)
            .or(magic_dir)
            .or(null)
            .or(bool_literal)
            .or(array_expr)
            .or(list_expr)
            .or(string)
            .or(number);

        #[derive(Clone)]
        enum Postfix {
            MethodCall { method: String, args: Vec<Expr> },
            Field(String),
            Index(Expr),
        }

        let method_call_postfix = just("->")
            .ignore_then(text::ident().padded())
            .then_ignore(just('(').padded())
            .then(args.clone())
            .then_ignore(just(')').padded())
            .map(|(method, args): (&str, Vec<Expr>)| Postfix::MethodCall {
                method: method.to_string(),
                args,
            });
        let field_postfix = just('.')
            .ignore_then(text::ident())
            .map(|field: &str| Postfix::Field(field.to_string()));
        let index_postfix = expr
            .clone()
            .padded()
            .delimited_by(just('[').padded(), just(']').padded())
            .map(Postfix::Index);

        let fielded = atom.clone().foldl(
            method_call_postfix
                .or(field_postfix)
                .or(index_postfix)
                .repeated(),
            |left, postfix| match postfix {
                Postfix::MethodCall { method, args } => {
                    let end = args
                        .last()
                        .map_or(left.span().end + method.len() + 4, |arg| arg.span().end + 1);
                    let span = Span::new(left.span().start, end);

                    Expr::MethodCall(Box::new(MethodCallExpr {
                        object: left,
                        method,
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
            },
        );

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
            });

        let unary_op = just('+')
            .to(UnaryOp::Plus)
            .or(just('-').to(UnaryOp::Minus))
            .padded();

        let signed = unary_op.repeated().foldr(powered, |op, expr| {
            let span = Span::new(expr.span().start.saturating_sub(1), expr.span().end);
            Expr::Unary(Box::new(UnaryExpr { op, expr, span }))
        });

        let multiplicative = signed.clone().foldl(
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
        );

        let dotted = multiplicative.clone().foldl(
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
        );

        let additive = dotted.clone().foldl(
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
        );

        let is_expr = additive.clone().foldl(
            text::keyword("is")
                .padded()
                .ignore_then(text::keyword("not").padded().to(true).or_not())
                .then_ignore(text::keyword("null").padded())
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
        );

        just('$')
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
            })
            .or(is_expr)
    });

    let statement = recursive(|statement| {
        let terminator = just(';').padded().ignored().or(end().ignored());

        let qualified_name = text::ident()
            .separated_by(just('\\'))
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|parts| QualifiedName::new(parts.into_iter().map(str::to_string).collect()));

        let type_expr = recursive(|type_expr| {
            let type_name = text::ident()
                .separated_by(just('\\'))
                .at_least(1)
                .collect::<Vec<_>>()
                .map(|parts| parts.join("\\"));

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
                .separated_by(just('|').padded())
                .at_least(1)
                .collect::<Vec<_>>()
                .map(|parts| parts.join("|"))
        });

        let namespace_stmt = text::keyword("namespace")
            .padded()
            .ignore_then(
                text::keyword("std")
                    .padded()
                    .ignore_then(qualified_name.clone())
                    .map(|name| (NamespaceSource::Std, name))
                    .or(qualified_name
                        .clone()
                        .map(|name| (NamespaceSource::Php, name))),
            )
            .then_ignore(terminator.clone())
            .map_with(|(source, name), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Namespace(NamespaceStmt {
                    source,
                    name,
                    span: Span::new(span.start, span.end),
                })
            });

        let use_stmt = text::keyword("use")
            .padded()
            .ignore_then(qualified_name.clone())
            .then(
                text::keyword("as")
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
            });

        let import_source = text::keyword("std")
            .padded()
            .to(ImportSource::Std)
            .or(just('"')
                .ignore_then(none_of('"').repeated().collect::<String>())
                .then_ignore(just('"'))
                .padded()
                .map(ImportSource::File));

        let import_stmt = text::keyword("from")
            .padded()
            .ignore_then(import_source)
            .then_ignore(text::keyword("use").padded())
            .then(qualified_name.clone())
            .then(
                text::keyword("as")
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
            );

        let echo_exprs = expr
            .clone()
            .padded()
            .separated_by(just(',').padded())
            .at_least(1)
            .collect::<Vec<_>>();

        let echo_stmt = just("echo")
            .padded()
            .ignore_then(echo_exprs)
            .then_ignore(terminator.clone())
            .map_with(|exprs, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Echo(EchoStmt {
                    exprs,
                    span: Span::new(span.start, span.end),
                })
            });

        let return_stmt = just("return")
            .padded()
            .ignore_then(expr.clone().padded())
            .then_ignore(terminator.clone())
            .map_with(|value, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Return(ReturnStmt {
                    value,
                    span: Span::new(span.start, span.end),
                })
            });

        let yield_stmt = text::keyword("yield")
            .padded()
            .ignore_then(expr.clone().padded())
            .then_ignore(terminator.clone())
            .map_with(|value, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Yield(YieldStmt {
                    value,
                    span: Span::new(span.start, span.end),
                })
            });

        let statement_function_name = text::ident()
            .separated_by(just('.'))
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|parts| parts.join("."));

        let function_call_stmt = statement_function_name
            .padded()
            .then_ignore(just('(').padded())
            .then(
                expr.clone()
                    .padded()
                    .separated_by(just(',').padded())
                    .allow_trailing()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(just(')').padded())
            .then_ignore(terminator.clone())
            .map_with(|(name, args): (String, Vec<Expr>), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::FunctionCall(FunctionCallStmt {
                    name,
                    args,
                    span: Span::new(span.start, span.end),
                })
            });

        let dynamic_function_call_stmt = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just('(').padded())
            .then(
                expr.clone()
                    .padded()
                    .separated_by(just(',').padded())
                    .allow_trailing()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(just(')').padded())
            .then_ignore(terminator.clone())
            .map_with(|(name, args): (&str, Vec<Expr>), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::DynamicFunctionCall(DynamicFunctionCallStmt {
                    name: name.to_string(),
                    args,
                    span: Span::new(span.start, span.end),
                })
            });

        let params = just('$')
            .ignore_then(text::ident())
            .padded()
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect::<Vec<_>>();

        let typed_param = type_expr
            .clone()
            .padded()
            .or_not()
            .then_ignore(just('$'))
            .then(text::ident().padded())
            .map(|(ty, name): (Option<String>, &str)| TypedParam {
                name: name.to_string(),
                ty,
            });

        let method_decl = text::keyword("intrinsic")
            .padded()
            .to(true)
            .or_not()
            .map(|is_intrinsic| is_intrinsic.unwrap_or(false))
            .then(
                text::keyword("static")
                    .padded()
                    .to(true)
                    .or_not()
                    .map(|is_static| is_static.unwrap_or(false)),
            )
            .then_ignore(text::keyword("function").padded())
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
                |((((is_intrinsic, is_static), name), params), return_type): (
                    (((bool, bool), &str), Vec<TypedParam>),
                    Option<String>,
                ),
                 extra| {
                    let span: SimpleSpan = extra.span();

                    ClassMember::Method(MethodDecl {
                        name: name.to_string(),
                        params,
                        return_type,
                        is_static,
                        is_intrinsic,
                        span: Span::new(span.start, span.end),
                    })
                },
            );

        let class_decl_stmt = text::keyword("class")
            .padded()
            .ignore_then(text::ident().padded())
            .then_ignore(just('{').padded())
            .then(method_decl.repeated().collect::<Vec<_>>())
            .then_ignore(just('}').padded())
            .then_ignore(terminator.clone().or_not())
            .map_with(|(name, members): (&str, Vec<ClassMember>), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::ClassDecl(ClassDeclStmt {
                    name: name.to_string(),
                    members,
                    span: Span::new(span.start, span.end),
                })
            });

        let type_field = text::keyword("const")
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
            );

        let type_decl_stmt = text::keyword("type")
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
            });

        let function_decl_stmt = just("function")
            .padded()
            .ignore_then(text::ident().padded())
            .then_ignore(just('(').padded())
            .then(params)
            .then_ignore(just(')').padded())
            .then_ignore(just('{').padded())
            .then(statement.clone().repeated().collect::<Vec<_>>())
            .then_ignore(just('}').padded())
            .then_ignore(terminator.clone().or_not())
            .map_with(
                |((name, params), body): ((&str, Vec<&str>), Vec<Stmt>), extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::FunctionDecl(FunctionDeclStmt {
                        name: name.to_string(),
                        params: params
                            .into_iter()
                            .map(|name| TypedParam {
                                name: name.to_string(),
                                ty: None,
                            })
                            .collect(),
                        return_type: None,
                        is_intrinsic: false,
                        is_generator: false,
                        body,
                        span: Span::new(span.start, span.end),
                    })
                },
            );

        let fn_decl_stmt = text::keyword("fn")
            .padded()
            .ignore_then(text::ident().padded())
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
                |(((name, params), return_type), body): (
                    ((&str, Vec<TypedParam>), Option<String>),
                    Vec<Stmt>,
                ),
                 extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::FunctionDecl(FunctionDeclStmt {
                        name: name.to_string(),
                        params,
                        return_type,
                        is_intrinsic: false,
                        is_generator: false,
                        body,
                        span: Span::new(span.start, span.end),
                    })
                },
            );

        let intrinsic_function_decl_stmt = text::keyword("intrinsic")
            .padded()
            .ignore_then(text::keyword("function").padded())
            .ignore_then(text::ident().padded())
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
                |((name, params), return_type): ((&str, Vec<TypedParam>), Option<String>),
                 extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::FunctionDecl(FunctionDeclStmt {
                        name: name.to_string(),
                        params,
                        return_type,
                        is_intrinsic: true,
                        is_generator: false,
                        body: Vec::new(),
                        span: Span::new(span.start, span.end),
                    })
                },
            );

        let gen_fn_decl_stmt = text::keyword("gen")
            .padded()
            .ignore_then(text::keyword("fn").padded())
            .ignore_then(text::ident().padded())
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
                |(((name, params), return_type), body): (
                    ((&str, Vec<TypedParam>), Option<String>),
                    Vec<Stmt>,
                ),
                 extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::FunctionDecl(FunctionDeclStmt {
                        name: name.to_string(),
                        params,
                        return_type,
                        is_intrinsic: false,
                        is_generator: true,
                        body,
                        span: Span::new(span.start, span.end),
                    })
                },
            );

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
            .delimited_by(just('{').padded(), just('}').padded());

        let let_stmt = text::keyword("let")
            .padded()
            .ignore_then(type_expr.clone().padded().or_not())
            .then_ignore(just('$'))
            .then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then(expr.clone())
            .then_ignore(terminator.clone())
            .map_with(
                |((ty, name), value): ((Option<String>, &str), Expr), extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::Let(LetStmt {
                        name: name.to_string(),
                        ty,
                        value,
                        span: Span::new(span.start, span.end),
                    })
                },
            );

        let let_join_run_block_stmt = text::keyword("let")
            .padded()
            .ignore_then(type_expr.clone().padded().or_not())
            .then_ignore(just('$'))
            .then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then_ignore(text::keyword("join").padded())
            .then_ignore(text::keyword("run").padded())
            .then(block.clone())
            .then_ignore(terminator.clone().or_not())
            .map_with(
                |((ty, name), body): ((Option<String>, &str), Vec<Stmt>), extra| {
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
            );

        let let_loop_block_stmt = text::keyword("let")
            .padded()
            .ignore_then(type_expr.clone().padded().or_not())
            .then_ignore(just('$'))
            .then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then_ignore(text::keyword("loop").padded())
            .then(block.clone())
            .then_ignore(terminator.clone().or_not())
            .map_with(
                |((ty, name), body): ((Option<String>, &str), Vec<Stmt>), extra| {
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
            );

        let let_run_block_stmt = text::keyword("let")
            .padded()
            .ignore_then(type_expr.clone().padded().or_not())
            .then_ignore(just('$'))
            .then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then_ignore(text::keyword("run").padded())
            .then(block.clone())
            .then_ignore(terminator.clone().or_not())
            .map_with(
                |((ty, name), body): ((Option<String>, &str), Vec<Stmt>), extra| {
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
            );

        let run_group_entries = block
            .clone()
            .padded()
            .separated_by(just(',').padded())
            .allow_trailing()
            .at_least(1)
            .collect::<Vec<_>>()
            .delimited_by(just('[').padded(), just(']').padded());

        let let_run_group_stmt = text::keyword("let")
            .padded()
            .ignore_then(type_expr.clone().padded().or_not())
            .then_ignore(just('$'))
            .then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then_ignore(text::keyword("run").padded())
            .then(run_group_entries.clone())
            .then_ignore(terminator.clone().or_not())
            .map_with(
                |((ty, name), entries): ((Option<String>, &str), Vec<Vec<Stmt>>), extra| {
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
            );

        let run_block_stmt = text::keyword("run")
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
            });

        let append_stmt = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just('[').padded())
            .then_ignore(just(']').padded())
            .then_ignore(just('=').padded())
            .then(expr.clone())
            .then_ignore(terminator.clone())
            .map_with(|(target, value): (&str, Expr), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Append(AppendStmt {
                    target: target.to_string(),
                    value,
                    span: Span::new(span.start, span.end),
                })
            });

        let loop_stmt = text::keyword("loop")
            .padded()
            .ignore_then(block.clone())
            .then_ignore(terminator.clone().or_not())
            .map_with(|body, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Loop(LoopStmt {
                    body,
                    span: Span::new(span.start, span.end),
                })
            });

        let if_stmt = text::keyword("if")
            .padded()
            .ignore_then(expr.clone())
            .then(block.clone())
            .then_ignore(terminator.clone().or_not())
            .map_with(|(condition, body), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::If(IfStmt {
                    condition,
                    body,
                    span: Span::new(span.start, span.end),
                })
            });

        let break_stmt = text::keyword("break")
            .padded()
            .ignore_then(expr.clone().padded().or_not())
            .then_ignore(terminator.clone())
            .map_with(|value, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Break(BreakStmt {
                    value,
                    span: Span::new(span.start, span.end),
                })
            });

        let assign_defer_block_stmt = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then_ignore(text::keyword("defer").padded())
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
            });

        let assign_run_block_stmt = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then_ignore(text::keyword("run").padded())
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
            });

        let assign_run_group_stmt = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then_ignore(text::keyword("run").padded())
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
            });

        let assign_fork_block_stmt = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then_ignore(text::keyword("fork").padded())
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
            });

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
            });

        let expr_stmt = expr
            .clone()
            .then_ignore(terminator.clone())
            .map_with(|expr, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Expr(echo_ast::ExprStmt {
                    expr,
                    span: Span::new(span.start, span.end),
                })
            });

        gen_fn_decl_stmt
            .or(fn_decl_stmt)
            .or(function_decl_stmt)
            .or(intrinsic_function_decl_stmt)
            .or(class_decl_stmt)
            .or(type_decl_stmt)
            .or(namespace_stmt)
            .or(use_stmt)
            .or(import_stmt)
            .or(echo_stmt)
            .or(return_stmt)
            .or(yield_stmt)
            .or(loop_stmt)
            .or(if_stmt)
            .or(break_stmt)
            .or(let_join_run_block_stmt)
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
            .or(assign_stmt)
            .or(dynamic_function_call_stmt)
            .or(function_call_stmt)
            .or(expr_stmt)
    });

    open_php
        .then(statement.repeated().at_least(1).collect::<Vec<_>>())
        .then_ignore(end())
        .map_with(|(open_tag, statements), extra| {
            let span: SimpleSpan = extra.span();

            Program {
                open_tag,
                statements,
                source_dir: None,
                span: Span::new(span.start, span.end),
            }
        })
}

fn normalize_heredoc_literals(source: &str) -> String {
    let mut output = source.as_bytes().to_vec();
    let mut index = 0;
    let bytes = source.as_bytes();

    while index + 3 <= bytes.len() {
        if &bytes[index..index + 3] != b"<<<" {
            index += 1;
            continue;
        }

        let Some((label, content_start)) = parse_heredoc_label(source, index + 3) else {
            index += 3;
            continue;
        };
        let Some(end_start) = find_heredoc_end(source, content_start, &label) else {
            index += 3;
            continue;
        };

        output[index] = b'"';
        for byte in output.iter_mut().take(content_start).skip(index + 1) {
            if *byte != b'\n' && *byte != b'\r' {
                *byte = b' ';
            }
        }
        for byte in output.iter_mut().take(end_start).skip(content_start) {
            if *byte == b'"' || *byte == b'\\' {
                *byte = b' ';
            }
        }
        output[end_start] = b'"';
        let mut end = end_start + label.len();
        if output.get(end) == Some(&b';') {
            end += 1;
        }
        for byte in output.iter_mut().take(end).skip(end_start + 1) {
            if *byte != b'\n' && *byte != b'\r' {
                *byte = b' ';
            }
        }
        index = end;
    }

    String::from_utf8(output).expect("heredoc normalization preserves UTF-8")
}

fn parse_heredoc_label(source: &str, mut index: usize) -> Option<(String, usize)> {
    let bytes = source.as_bytes();
    while matches!(bytes.get(index), Some(b' ' | b'\t')) {
        index += 1;
    }

    let quote = match bytes.get(index) {
        Some(b'\'' | b'"') => {
            let quote = bytes[index];
            index += 1;
            Some(quote)
        }
        _ => None,
    };

    let label_start = index;
    while matches!(
        bytes.get(index),
        Some(b'A'..=b'Z' | b'a'..=b'z' | b'_' | b'0'..=b'9')
    ) {
        index += 1;
    }
    if index == label_start {
        return None;
    }
    let label = source[label_start..index].to_string();

    if let Some(quote) = quote {
        if bytes.get(index) != Some(&quote) {
            return None;
        }
        index += 1;
    }
    while matches!(bytes.get(index), Some(b' ' | b'\t')) {
        index += 1;
    }
    match bytes.get(index) {
        Some(b'\n') => Some((label, index + 1)),
        Some(b'\r') if bytes.get(index + 1) == Some(&b'\n') => Some((label, index + 2)),
        _ => None,
    }
}

fn find_heredoc_end(source: &str, mut index: usize, label: &str) -> Option<usize> {
    while index < source.len() {
        let line_end = source[index..]
            .find(['\r', '\n'])
            .map(|offset| index + offset)
            .unwrap_or(source.len());
        let line = &source[index..line_end];
        let terminator = line.strip_suffix(';').unwrap_or(line);
        if terminator == label {
            return Some(index);
        }
        index = line_end;
        if source.as_bytes().get(index) == Some(&b'\r') {
            index += 1;
        }
        if source.as_bytes().get(index) == Some(&b'\n') {
            index += 1;
        }
    }
    None
}

fn unescape_double_quoted_string(input: String) -> String {
    let mut output = String::new();
    let mut chars = input.chars();

    while let Some(c) = chars.next() {
        if c != '\\' {
            output.push(c);
            continue;
        }

        match chars.next() {
            Some('n') => output.push('\n'),
            Some('t') => output.push('\t'),
            Some('r') => output.push('\r'),
            Some('"') => output.push('"'),
            Some('\\') => output.push('\\'),
            Some(other) => output.push(other),
            None => output.push('\\'),
        }
    }

    output
}

fn unescape_single_quoted_string(input: String) -> String {
    let mut output = String::new();
    let mut chars = input.chars();

    while let Some(c) = chars.next() {
        if c != '\\' {
            output.push(c);
            continue;
        }

        match chars.next() {
            Some('\\') => output.push('\\'),
            Some('\'') => output.push('\''),
            Some(other) => {
                output.push('\\');
                output.push(other);
            }
            None => output.push('\\'),
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_php_string_literal_forms() {
        let program = parse_with_mode(
            r#"<?php
echo 'single quoted';
echo "double quoted\n";
echo <<<'NOW'
nowdoc body
NOW;
echo <<<HTML
heredoc body
HTML;
"#,
            SourceMode::Echo,
        )
        .expect("program parses");

        assert_eq!(program.statements.len(), 4);
        assert!(matches!(
            &program.statements[0],
            Stmt::Echo(statement)
                if matches!(&statement.exprs[0], Expr::String(string) if string.value == "single quoted")
        ));
        assert!(matches!(
            &program.statements[1],
            Stmt::Echo(statement)
                if matches!(&statement.exprs[0], Expr::String(string) if string.value == "double quoted\n")
        ));
        assert!(matches!(
            &program.statements[2],
            Stmt::Echo(statement)
                if matches!(&statement.exprs[0], Expr::String(string) if string.value.contains("nowdoc body"))
        ));
        assert!(matches!(
            &program.statements[3],
            Stmt::Echo(statement)
                if matches!(&statement.exprs[0], Expr::String(string) if string.value.contains("heredoc body"))
        ));
    }

    #[test]
    fn parses_php_single_quoted_string_escapes() {
        let program = parse_with_mode(r#"<?php echo 'c:\path\n and \'quote\'';"#, SourceMode::Echo)
            .expect("program parses");

        assert!(matches!(
            &program.statements[0],
            Stmt::Echo(statement)
                if matches!(&statement.exprs[0], Expr::String(string) if string.value == r#"c:\path\n and 'quote'"#)
        ));
    }

    #[test]
    fn parses_concurrency_keyword_expressions() {
        let program = parse_with_mode(
            r#"<?php
$task = run $deferred;
$worker = fork $job;
$process = spawn "worker";
$value = join $task;
"#,
            SourceMode::Echo,
        )
        .expect("program parses");

        assert!(matches!(
            &program.statements[0],
            Stmt::Assign(statement) if matches!(statement.value, Expr::Run(_))
        ));
        assert!(matches!(
            &program.statements[1],
            Stmt::Assign(statement) if matches!(statement.value, Expr::Fork(_))
        ));
        assert!(matches!(
            &program.statements[2],
            Stmt::Assign(statement) if matches!(statement.value, Expr::Spawn(_))
        ));
        assert!(matches!(
            &program.statements[3],
            Stmt::Assign(statement) if matches!(statement.value, Expr::Join(_))
        ));
    }

    #[test]
    fn parses_concurrency_block_assignments() {
        let program = parse_with_mode(
            r#"<?php
$deferred = defer { return "later"; };
$task = run { return "soon"; };
$worker = fork { return "parallel"; };
"#,
            SourceMode::Echo,
        )
        .expect("program parses");

        assert!(matches!(
            &program.statements[0],
            Stmt::Assign(statement) if matches!(statement.value, Expr::Defer(_))
        ));
        assert!(matches!(
            &program.statements[1],
            Stmt::Assign(statement) if matches!(statement.value, Expr::Run(RunExpr::Block { .. }))
        ));
        assert!(matches!(
            &program.statements[2],
            Stmt::Assign(statement) if matches!(statement.value, Expr::Fork(ForkExpr::Block { .. }))
        ));
    }

    #[test]
    fn parses_run_group_assignments() {
        let program = parse_with_mode(
            r#"<?php
$tasks = run [
    { return "user"; },
    { return "posts"; },
];
let $more = run [
    { return "audit"; },
];
"#,
            SourceMode::Echo,
        )
        .expect("program parses");

        assert!(matches!(
            &program.statements[0],
            Stmt::Assign(statement)
                if matches!(&statement.value, Expr::Run(RunExpr::Group { entries, .. }) if entries.len() == 2)
        ));
        assert!(matches!(
            &program.statements[1],
            Stmt::Let(statement)
                if matches!(&statement.value, Expr::Run(RunExpr::Group { entries, .. }) if entries.len() == 1)
        ));
    }

    #[test]
    fn parses_optional_statement_semicolons() {
        let program = parse_with_mode(
            r#"<?php
namespace App\Http
use Psr\Log\LoggerInterface
echo "hello"
$name = "Echo"
$alias = "Alias"
strlen($name)
$fn()
function greet($name) { return $name }
"#,
            SourceMode::Echo,
        )
        .expect("program parses without semicolons");

        assert!(matches!(&program.statements[0], Stmt::Namespace(_)));
        assert!(matches!(&program.statements[1], Stmt::Use(_)));
        assert!(matches!(&program.statements[2], Stmt::Echo(_)));
        assert!(matches!(&program.statements[3], Stmt::Assign(_)));
        assert!(matches!(&program.statements[4], Stmt::Assign(_)));
        assert!(matches!(&program.statements[5], Stmt::FunctionCall(_)));
        assert!(matches!(
            &program.statements[6],
            Stmt::DynamicFunctionCall(_)
        ));
        assert!(matches!(&program.statements[7], Stmt::FunctionDecl(_)));
    }

    #[test]
    fn parses_optional_semicolons_after_concurrency_blocks() {
        let program = parse_with_mode(
            r#"<?php
$deferred = defer { return "later" }
$task = run { return "soon" }
$worker = fork { return "parallel" }
"#,
            SourceMode::Echo,
        )
        .expect("concurrency block assignments parse without semicolons");

        assert!(matches!(
            &program.statements[0],
            Stmt::Assign(statement) if matches!(statement.value, Expr::Defer(_))
        ));
        assert!(matches!(
            &program.statements[1],
            Stmt::Assign(statement) if matches!(statement.value, Expr::Run(RunExpr::Block { .. }))
        ));
        assert!(matches!(
            &program.statements[2],
            Stmt::Assign(statement) if matches!(statement.value, Expr::Fork(ForkExpr::Block { .. }))
        ));
    }

    #[test]
    fn preserves_multiline_concat_expressions() {
        let program = parse_with_mode(
            r#"<?php
$body = "Hello "
    . $name
    . "\n"
echo $body
"#,
            SourceMode::Echo,
        )
        .expect("multiline concat parses");

        assert!(matches!(
            &program.statements[0],
            Stmt::Assign(statement) if matches!(statement.value, Expr::Binary(_))
        ));
        assert!(matches!(&program.statements[1], Stmt::Echo(_)));
    }

    #[test]
    fn preserves_multiline_assignment_rhs() {
        let program = parse_with_mode(
            r#"<?php
$body =
    "Hello " . $name
echo $body
"#,
            SourceMode::Echo,
        )
        .expect("multiline assignment parses");

        assert!(matches!(&program.statements[0], Stmt::Assign(_)));
        assert!(matches!(&program.statements[1], Stmt::Echo(_)));
    }

    #[test]
    fn preserves_multiline_function_calls() {
        let program = parse_with_mode(
            r#"<?php
strlen(
    "Echo"
)
echo "done"
"#,
            SourceMode::Echo,
        )
        .expect("multiline function call parses");

        assert!(matches!(&program.statements[0], Stmt::FunctionCall(_)));
        assert!(matches!(&program.statements[1], Stmt::Echo(_)));
    }

    #[test]
    fn parses_std_net_module_source() {
        let program = parse_trusted_std(include_str!("../../../std/net.echo"))
            .expect("std net module parses");

        assert!(matches!(
            &program.statements[0],
                Stmt::Namespace(statement)
                    if statement.source == NamespaceSource::Std
                    && statement.name.as_string() == "net"
        ));
        assert!(matches!(
            &program.statements[7],
            Stmt::ClassDecl(statement) if statement.name == "TcpServer"
        ));
        assert!(matches!(
            &program.statements[8],
            Stmt::ClassDecl(statement) if statement.name == "TcpConnection"
        ));
    }

    #[test]
    fn parses_std_time_module_source() {
        let program = parse_trusted_std(include_str!("../../../std/time.echo"))
            .expect("std time module parses");

        assert!(matches!(
            &program.statements[0],
            Stmt::Namespace(statement)
                if statement.source == NamespaceSource::Std
                    && statement.name.as_string() == "time"
        ));
        assert!(matches!(&program.statements[1], Stmt::FunctionDecl(_)));
    }

    #[test]
    fn parses_std_http_module_source() {
        let program = parse_trusted_std(include_str!("../../../std/http.echo"))
            .expect("std http module parses");

        assert!(matches!(
            &program.statements[0],
            Stmt::Namespace(statement)
                if statement.source == NamespaceSource::Std
                    && statement.name.as_string() == "http"
        ));
        assert!(matches!(&program.statements[1], Stmt::FunctionDecl(_)));
    }

    #[test]
    fn parses_dotted_std_function_call() {
        let program = parse_with_mode(
            r#"from std use time
time.sleep(300)
"#,
            SourceMode::Strict,
        )
        .expect("dotted function call parses");

        assert!(matches!(
            &program.statements[1],
            Stmt::FunctionCall(statement) if statement.name == "time.sleep"
        ));
    }

    #[test]
    fn parses_negative_numeric_function_arguments() {
        let program = parse_with_mode(
            r#"<?php echo substr_compare("abcde", "de", -2, 2);"#,
            SourceMode::Echo,
        )
        .expect("negative numeric argument parses");

        assert!(matches!(
            &program.statements[0],
            Stmt::Echo(statement)
                if matches!(
                    &statement.exprs[0],
                    Expr::FunctionCall(call)
                        if matches!(
                            &call.args[2],
                            Expr::Unary(expr)
                                if expr.op == UnaryOp::Minus
                                    && matches!(&expr.expr, Expr::Number(number) if number.value == "2")
                        )
                )
        ));
    }

    #[test]
    fn parses_subtraction_expression() {
        let program = parse_with_mode("3-5", SourceMode::Strict).expect("subtraction parses");

        assert!(matches!(
            &program.statements[0],
            Stmt::Expr(statement)
                if matches!(
                    &statement.expr,
                    Expr::Binary(expr)
                        if expr.op == BinaryOp::Sub
                            && matches!(&expr.left, Expr::Number(number) if number.value == "3")
                            && matches!(&expr.right, Expr::Number(number) if number.value == "5")
                )
        ));
    }

    #[test]
    fn parses_php_arithmetic_precedence() {
        let program = parse_with_mode("2+3*4", SourceMode::Strict).expect("arithmetic parses");

        assert!(matches!(
            &program.statements[0],
            Stmt::Expr(statement)
                if matches!(
                    &statement.expr,
                    Expr::Binary(expr)
                        if expr.op == BinaryOp::Add
                            && matches!(&expr.right, Expr::Binary(right) if right.op == BinaryOp::Mul)
                )
        ));
    }

    #[test]
    fn parses_parenthesized_and_unary_arithmetic() {
        let program =
            parse_with_mode("-(2+3)", SourceMode::Strict).expect("parenthesized unary parses");

        assert!(matches!(
            &program.statements[0],
            Stmt::Expr(statement)
                if matches!(
                    &statement.expr,
                    Expr::Unary(expr)
                        if expr.op == UnaryOp::Minus
                            && matches!(&expr.expr, Expr::Binary(binary) if binary.op == BinaryOp::Add)
                )
        ));
    }

    #[test]
    fn parses_brace_values_as_lists_or_structural_objects() {
        let list = parse_with_mode("{1, 2, 3}", SourceMode::Strict).expect("list literal parses");
        assert!(matches!(
            &list.statements[0],
            Stmt::Expr(statement)
                if matches!(&statement.expr, Expr::List(expr) if expr.values.len() == 3)
        ));

        let object =
            parse_with_mode("{ test: 5 }", SourceMode::Strict).expect("object literal parses");
        assert!(matches!(
            &object.statements[0],
            Stmt::Expr(statement)
                if matches!(
                    &statement.expr,
                    Expr::Object(expr) if expr.name.is_empty()
                        && expr.fields.len() == 1
                        && expr.fields[0].name == "test"
                )
        ));
    }

    #[test]
    fn parses_bracket_values_as_arrays() {
        let program = parse_with_mode("[1, 2, 3]", SourceMode::Strict).expect("array parses");

        assert!(matches!(
            &program.statements[0],
            Stmt::Expr(statement)
                if matches!(&statement.expr, Expr::Array(expr) if expr.elements.len() == 3)
        ));
    }

    #[test]
    fn parses_index_access_expressions() {
        let program = parse_with_mode("$a[0]", SourceMode::Strict).expect("index access parses");

        assert!(matches!(
            &program.statements[0],
            Stmt::Expr(statement)
                if matches!(
                    &statement.expr,
                    Expr::Index(expr)
                        if matches!(&expr.collection, Expr::Variable(variable) if variable.name == "a")
                            && matches!(&expr.index, Expr::Number(number) if number.value == "0")
                )
        ));
    }

    #[test]
    fn echo_mode_accepts_php_compat_keyed_arrays() {
        let program = parse_with_mode(r#"["asdf" => 5]"#, SourceMode::Echo)
            .expect("PHP keyed array parses in Echo mode");

        assert!(matches!(
            &program.statements[0],
            Stmt::Expr(statement)
                if matches!(
                    &statement.expr,
                    Expr::Array(expr)
                        if expr.elements.len() == 1 && expr.elements[0].key.is_some()
                )
        ));
    }

    #[test]
    fn strict_mode_rejects_php_compat_keyed_arrays() {
        let diagnostics = parse_with_mode(r#"["asdf" => 5]"#, SourceMode::Strict)
            .expect_err("strict mode rejects keyed arrays");

        assert_eq!(
            diagnostics[0].message,
            "keyed array elements are not allowed in strict mode"
        );
    }

    #[test]
    fn strict_mode_rejects_user_std_namespace_declaration() {
        let diagnostics = parse_with_mode("namespace std net", SourceMode::Strict)
            .expect_err("user std namespace should be rejected");

        assert_eq!(
            diagnostics[0].message,
            "std namespace declarations are only allowed in trusted stdlib source"
        );
    }

    #[test]
    fn strict_mode_allows_php_namespace_named_std_net() {
        let program = parse_with_mode("namespace std\\Net", SourceMode::Strict)
            .expect("PHP namespace should stay valid");

        assert!(matches!(
            &program.statements[0],
            Stmt::Namespace(statement)
                if statement.source == NamespaceSource::Php
                    && statement.name.as_string() == "std\\Net"
        ));
    }

    #[test]
    fn parses_echo_fn_declaration() {
        let program = parse_with_mode(
            r#"fn responseBody($request, list<User> $users): string {
    let $body = "Hello " . $request.path . "\n"
    return $body
}
"#,
            SourceMode::Strict,
        )
        .expect("fn declaration parses");

        assert!(matches!(&program.statements[0], Stmt::FunctionDecl(_)));
    }

    #[test]
    fn parses_response_body_fn() {
        let program = parse_with_mode(
            r#"fn responseBody($request, list<User> $users): string {
    let $body = "Hello from Echo at " . $request.path . "\n"
    return $body . "Users seen: " . count($users) . "\n"
}
"#,
            SourceMode::Strict,
        )
        .expect("response body function parses");

        assert!(matches!(&program.statements[0], Stmt::FunctionDecl(_)));
    }

    #[test]
    fn parses_field_access_in_concat() {
        let program = parse_with_mode(
            r#"let $body = "Hello from Echo at " . $request.path . "\n"
return $body . "Users seen: " . count($users) . "\n"
"#,
            SourceMode::Strict,
        )
        .expect("field access in concat parses");

        assert!(matches!(&program.statements[0], Stmt::Let(_)));
        assert!(matches!(&program.statements[1], Stmt::Return(_)));
    }

    #[test]
    fn parses_type_declaration_before_fn() {
        let program = parse_with_mode(
            r#"type User = {
    const id: int
    email: string
    displayName?: string
}

fn responseBody($request, list<User> $users): string {
    return "ok"
}
"#,
            SourceMode::Strict,
        )
        .expect("type followed by fn parses");

        assert!(matches!(&program.statements[0], Stmt::TypeDecl(_)));
        assert!(matches!(&program.statements[1], Stmt::FunctionDecl(_)));
    }

    #[test]
    fn preserves_typed_let_annotation() {
        let program = parse_with_mode("let list<User> $users = {}", SourceMode::Strict)
            .expect("typed let parses");

        assert!(matches!(
            &program.statements[0],
            Stmt::Let(statement) if statement.name == "users" && statement.ty.as_deref() == Some("list<User>")
        ));
    }

    #[test]
    fn parses_target_namespace_import_type_fn_prefix() {
        let program = parse_with_mode(
            r#"namespace app\http

from std use net
from std use http

type User = {
    const id: int
    email: string
    displayName?: string
}

fn responseBody($request, list<User> $users): string {
    return "ok"
}
"#,
            SourceMode::Strict,
        )
        .expect("target prefix parses");

        assert_eq!(program.statements.len(), 5);
    }

    #[test]
    fn parses_http_server_target_fixture() {
        let program = parse_with_mode(
            include_str!("../../../tests/echo/011_http_server_target/program.echo"),
            SourceMode::Strict,
        )
        .expect("HTTP server target fixture parses");

        assert!(matches!(program.statements.last(), Some(Stmt::Loop(_))));
    }

    #[test]
    fn parses_target_server_loop() {
        let program = parse_with_mode(
            r#"loop {
    let $conn = join run {
        return net.accept($server)
    }

    run {
        let $request = http.readRequest($conn)

        $users[] = User {
            id: count($users) + 1
            email: "visitor" . count($users) . "@echo.local"
        }

        net.write($conn, http.responseText(responseBody($request, $users)))
        net.close($conn)
    }
}
"#,
            SourceMode::Strict,
        )
        .expect("target server loop parses");

        assert!(matches!(&program.statements[0], Stmt::Loop(_)));
    }

    #[test]
    fn parses_simple_loop() {
        let program = parse_with_mode(
            r#"loop {
    echo "x"
}
"#,
            SourceMode::Strict,
        )
        .expect("simple loop parses");

        assert!(matches!(&program.statements[0], Stmt::Loop(_)));
    }

    #[test]
    fn parses_loop_with_join_run() {
        let program = parse_with_mode(
            r#"loop {
    let $conn = join run {
        return net.accept($server)
    }
}
"#,
            SourceMode::Strict,
        )
        .expect("loop with join run parses");

        assert!(matches!(&program.statements[0], Stmt::Loop(_)));
    }

    #[test]
    fn parses_loop_with_run_block() {
        let program = parse_with_mode(
            r#"loop {
    run {
        net.close($conn)
    }
}
"#,
            SourceMode::Strict,
        )
        .expect("loop with run block parses");

        assert!(matches!(&program.statements[0], Stmt::Loop(_)));
    }

    #[test]
    fn parses_let_join_run_block() {
        let program = parse_with_mode(
            r#"let $conn = join run {
    return net.accept($server)
}
"#,
            SourceMode::Strict,
        )
        .expect("let join run block parses");

        assert!(matches!(&program.statements[0], Stmt::Let(_)));
    }

    #[test]
    fn parses_target_run_block() {
        let program = parse_with_mode(
            r#"run {
    let $request = http.readRequest($conn)

    $users[] = User {
        id: count($users) + 1
        email: "visitor" . count($users) . "@echo.local"
    }

    net.write($conn, http.responseText(responseBody($request, $users)))
    net.close($conn)
}
"#,
            SourceMode::Strict,
        )
        .expect("target run block parses");

        assert!(matches!(&program.statements[0], Stmt::Expr(_)));
    }

    #[test]
    fn parses_concurrency_expression_statements() {
        let program = parse_with_mode(
            r#"run $task
join $task
"#,
            SourceMode::Strict,
        )
        .expect("concurrency expression statements parse");

        assert!(matches!(
            &program.statements[0],
            Stmt::Expr(statement) if matches!(statement.expr, Expr::Run(_))
        ));
        assert!(matches!(
            &program.statements[1],
            Stmt::Expr(statement) if matches!(statement.expr, Expr::Join(_))
        ));
    }

    #[test]
    fn echo_mode_accepts_concurrency_keywords_in_php_files() {
        let program = parse_with_mode(
            r#"<?php
$task = run $deferred;
"#,
            SourceMode::Echo,
        )
        .expect("Echo superset mode accepts concurrency syntax");

        assert!(matches!(
            &program.statements[0],
            Stmt::Assign(statement) if matches!(statement.value, Expr::Run(_))
        ));
    }

    #[test]
    fn echo_mode_accepts_php_reference_assignment() {
        let program = parse_with_mode(
            r#"<?php
$a = "x";
$b =& $a;
"#,
            SourceMode::Echo,
        )
        .expect("Echo superset mode accepts PHP references");

        assert!(matches!(&program.statements[1], Stmt::AssignRef(_)));
    }

    #[test]
    fn strict_mode_rejects_php_reference_assignment() {
        let diagnostics = parse_with_mode(
            r#"let $a = "x"
$b =& $a
"#,
            SourceMode::Strict,
        )
        .expect_err("strict mode rejects PHP references");

        assert_eq!(
            diagnostics[0].message,
            "PHP references are not allowed in strict mode"
        );
    }

    #[test]
    fn echo_mode_accepts_php_array_append_assignment() {
        let program = parse_with_mode(
            r#"<?php
$a = [];
$a[] = 1;
"#,
            SourceMode::Echo,
        )
        .expect("Echo superset mode accepts PHP append syntax");

        assert!(matches!(&program.statements[1], Stmt::Append(_)));
    }

    #[test]
    fn strict_mode_parses_php_array_append_assignment_for_semantic_validation() {
        let program = parse_with_mode(
            r#"let $a = []
$a[] = 1
"#,
            SourceMode::Strict,
        )
        .expect("strict parser accepts append syntax for semantic validation");

        assert!(matches!(&program.statements[1], Stmt::Append(_)));
    }

    #[test]
    fn echo_mode_accepts_dynamic_function_calls() {
        let program = parse_with_mode(
            r#"<?php
$fn = "strlen";
$fn("Echo");
"#,
            SourceMode::Echo,
        )
        .expect("Echo superset mode accepts dynamic calls");

        assert!(matches!(
            &program.statements[1],
            Stmt::DynamicFunctionCall(_)
        ));
    }

    #[test]
    fn strict_mode_rejects_dynamic_function_calls() {
        let diagnostics = parse_with_mode(
            r#"let $fn = "strlen"
$fn("Echo")
"#,
            SourceMode::Strict,
        )
        .expect_err("strict mode rejects dynamic calls");

        assert_eq!(
            diagnostics[0].message,
            "dynamic function calls are not allowed in strict mode"
        );
    }
}
