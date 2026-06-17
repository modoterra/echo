use chumsky::prelude::*;
use chumsky::span::SimpleSpan;
use echo_ast::{
    AssignRefStmt, AssignStmt, BinaryExpr, BinaryOp, ClassDeclStmt, ClassMember, DeferExpr,
    DynamicFunctionCallStmt, EchoStmt, Expr, ForkExpr, FunctionCallExpr, FunctionCallStmt,
    FunctionDeclStmt, ImportSource, ImportStmt, JoinExpr, MethodDecl, NamespaceSource,
    NamespaceStmt, NullLiteral, NumberLiteral, Program, QualifiedName, ReturnStmt, RunExpr,
    SpawnExpr, Stmt, StringLiteral, TypedParam, UseStmt, VariableExpr,
};
use echo_diagnostics::Diagnostic;
use echo_source::{SourceMode, Span};

pub fn parse(source: &str) -> Result<Program, Vec<Diagnostic>> {
    parse_with_mode(source, SourceMode::Echo)
}

pub fn parse_with_mode(source: &str, mode: SourceMode) -> Result<Program, Vec<Diagnostic>> {
    // For now, run the Logos lexer first so lexer errors are caught.
    // The Chumsky parser below still parses the source text directly.
    echo_lexer::lex(source)?;

    let source = virtualize_statement_terminators(&strip_comments_preserving_spans(source));

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

fn validate_mode(program: &Program, mode: SourceMode) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    if mode == SourceMode::Strict {
        for statement in &program.statements {
            validate_statement_mode(statement, &mut diagnostics);
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn validate_statement_mode(statement: &Stmt, diagnostics: &mut Vec<Diagnostic>) {
    match statement {
        Stmt::Echo(statement) => {
            for expr in &statement.exprs {
                validate_expr_mode(expr, diagnostics);
            }
        }
        Stmt::FunctionCall(statement) => {
            for expr in &statement.args {
                validate_expr_mode(expr, diagnostics);
            }
        }
        Stmt::DynamicFunctionCall(statement) => {
            for expr in &statement.args {
                validate_expr_mode(expr, diagnostics);
            }
        }
        Stmt::FunctionDecl(statement) => {
            for statement in &statement.body {
                validate_statement_mode(statement, diagnostics);
            }
        }
        Stmt::Assign(statement) => validate_expr_mode(&statement.value, diagnostics),
        Stmt::AssignRef(_) => {}
        Stmt::Return(statement) => validate_expr_mode(&statement.value, diagnostics),
        Stmt::Expr(statement) => validate_expr_mode(&statement.expr, diagnostics),
        Stmt::Namespace(_) | Stmt::Use(_) | Stmt::Import(_) | Stmt::ClassDecl(_) => {}
    }
}

fn validate_expr_mode(expr: &Expr, diagnostics: &mut Vec<Diagnostic>) {
    match expr {
        Expr::Defer(_) | Expr::Run(_) | Expr::Fork(_) | Expr::Spawn(_) | Expr::Join(_) => {}
        Expr::FunctionCall(expr) => {
            for expr in &expr.args {
                validate_expr_mode(expr, diagnostics);
            }
        }
        Expr::Binary(expr) => {
            validate_expr_mode(&expr.left, diagnostics);
            validate_expr_mode(&expr.right, diagnostics);
        }
        Expr::Null(_) | Expr::String(_) | Expr::Number(_) | Expr::Variable(_) => {}
    }
}

fn strip_comments_preserving_spans(source: &str) -> String {
    let mut output = source.as_bytes().to_vec();
    let mut index = 0;
    let bytes = source.as_bytes();

    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                index += 1;
                while index < bytes.len() {
                    match bytes[index] {
                        b'\\' => index = (index + 2).min(bytes.len()),
                        b'"' => {
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
            b'"' => {
                index += 1;
                while index < bytes.len() {
                    match bytes[index] {
                        b'\\' => index = (index + 2).min(bytes.len()),
                        b'"' => {
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

    if !matches!(previous, b')' | b']' | b'}' | b'"' | b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_')
    {
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

        let string = none_of('"')
            .repeated()
            .collect::<String>()
            .delimited_by(just('"'), just('"'))
            .map(unescape_string)
            .map_with(|value, extra| {
                let span: SimpleSpan = extra.span();

                Expr::String(StringLiteral {
                    value,
                    span: Span::new(span.start, span.end),
                })
            });

        let number = text::digits(10).to_slice().map_with(|value: &str, extra| {
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

        let atom = run_expr
            .or(fork_expr)
            .or(spawn_expr)
            .or(join_expr)
            .or(function_call_expr)
            .or(variable)
            .or(null)
            .or(string)
            .or(number);

        atom.clone().foldl(
            just('.').padded().ignore_then(atom).repeated(),
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
    });

    let statement = recursive(|statement| {
        let terminator = just(';').padded().ignored().or(end().ignored());

        let qualified_name = text::ident()
            .separated_by(just('\\'))
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|parts| QualifiedName::new(parts.into_iter().map(str::to_string).collect()));

        let type_name = text::ident()
            .separated_by(just('\\'))
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|parts| parts.join("\\"));

        let type_expr = type_name
            .clone()
            .separated_by(just('|').padded())
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|parts| parts.join("|"));

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
                        params: params.into_iter().map(str::to_string).collect(),
                        return_type: None,
                        is_intrinsic: false,
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
                        params: params.into_iter().map(|param| param.name).collect(),
                        return_type,
                        is_intrinsic: true,
                        body: Vec::new(),
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

        let block = statement
            .clone()
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(just('{').padded(), just('}').padded());

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

        let assign_fork_block_stmt = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then_ignore(text::keyword("fork").padded())
            .then(block)
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

        function_decl_stmt
            .or(intrinsic_function_decl_stmt)
            .or(class_decl_stmt)
            .or(namespace_stmt)
            .or(use_stmt)
            .or(import_stmt)
            .or(echo_stmt)
            .or(return_stmt)
            .or(assign_defer_block_stmt)
            .or(assign_run_block_stmt)
            .or(assign_fork_block_stmt)
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
                span: Span::new(span.start, span.end),
            }
        })
}

fn unescape_string(input: String) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let program = parse_with_mode(include_str!("../../../std/net.echo"), SourceMode::Strict)
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
        let program = parse_with_mode(include_str!("../../../std/time.echo"), SourceMode::Strict)
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
}
