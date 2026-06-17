use chumsky::prelude::*;
use chumsky::span::SimpleSpan;
use echo_ast::{
    AssignRefStmt, AssignStmt, BinaryExpr, BinaryOp, DeferExpr, DynamicFunctionCallStmt, EchoStmt,
    Expr, ForkExpr, FunctionCallExpr, FunctionCallStmt, FunctionDeclStmt, JoinExpr, NamespaceStmt,
    NullLiteral, NumberLiteral, Program, QualifiedName, ReturnStmt, RunExpr, SpawnExpr, Stmt,
    StringLiteral, UseStmt, VariableExpr,
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

    let source = strip_comments_preserving_spans(source);

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
        Stmt::Namespace(_) | Stmt::Use(_) => {}
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

        let function_call_expr = text::ident()
            .padded()
            .then_ignore(just('(').padded())
            .then(args.clone())
            .then_ignore(just(')').padded())
            .map_with(|(name, args): (&str, Vec<Expr>), extra| {
                let span: SimpleSpan = extra.span();

                Expr::FunctionCall(FunctionCallExpr {
                    name: name.to_string(),
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
        let qualified_name = text::ident()
            .separated_by(just('\\'))
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|parts| QualifiedName::new(parts.into_iter().map(str::to_string).collect()));

        let namespace_stmt = text::keyword("namespace")
            .padded()
            .ignore_then(qualified_name.clone())
            .then_ignore(just(';').padded())
            .map_with(|name, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Namespace(NamespaceStmt {
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
            .then_ignore(just(';').padded())
            .map_with(|(name, alias): (QualifiedName, Option<&str>), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Use(UseStmt {
                    name,
                    alias: alias.map(str::to_string),
                    span: Span::new(span.start, span.end),
                })
            });

        let echo_exprs = expr
            .clone()
            .padded()
            .separated_by(just(',').padded())
            .at_least(1)
            .collect::<Vec<_>>();

        let echo_stmt = just("echo")
            .padded()
            .ignore_then(echo_exprs)
            .then_ignore(just(';').padded())
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
            .then_ignore(just(';').padded())
            .map_with(|value, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Return(ReturnStmt {
                    value,
                    span: Span::new(span.start, span.end),
                })
            });

        let function_call_stmt = text::ident()
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
            .then_ignore(just(';').padded())
            .map_with(|(name, args): (&str, Vec<Expr>), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::FunctionCall(FunctionCallStmt {
                    name: name.to_string(),
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
            .then_ignore(just(';').padded())
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

        let function_decl_stmt = just("function")
            .padded()
            .ignore_then(text::ident().padded())
            .then_ignore(just('(').padded())
            .then(params)
            .then_ignore(just(')').padded())
            .then_ignore(just('{').padded())
            .then(statement.clone().repeated().collect::<Vec<_>>())
            .then_ignore(just('}').padded())
            .map_with(
                |((name, params), body): ((&str, Vec<&str>), Vec<Stmt>), extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::FunctionDecl(FunctionDeclStmt {
                        name: name.to_string(),
                        params: params.into_iter().map(str::to_string).collect(),
                        body,
                        span: Span::new(span.start, span.end),
                    })
                },
            );

        let assign_stmt = just('$')
            .ignore_then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then(expr.clone())
            .then_ignore(just(';').padded())
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
            .then_ignore(just(';').padded())
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
            .then_ignore(just(';').padded())
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
            .then_ignore(just(';').padded())
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
            .then_ignore(just(';').padded())
            .map_with(|(name, target): (&str, &str), extra| {
                let span: SimpleSpan = extra.span();

                Stmt::AssignRef(AssignRefStmt {
                    name: name.to_string(),
                    target: target.to_string(),
                    span: Span::new(span.start, span.end),
                })
            });

        function_decl_stmt
            .or(namespace_stmt)
            .or(use_stmt)
            .or(echo_stmt)
            .or(return_stmt)
            .or(dynamic_function_call_stmt)
            .or(function_call_stmt)
            .or(assign_defer_block_stmt)
            .or(assign_run_block_stmt)
            .or(assign_fork_block_stmt)
            .or(assign_ref_stmt)
            .or(assign_stmt)
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
