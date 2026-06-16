use chumsky::prelude::*;
use chumsky::span::SimpleSpan;
use echo_ast::{
    AssignStmt, BinaryExpr, BinaryOp, EchoStmt, Expr, FunctionCallExpr, FunctionCallStmt,
    NumberLiteral, Program, Stmt, StringLiteral, VariableExpr,
};
use echo_diagnostics::Diagnostic;
use echo_source::Span;

pub fn parse(source: &str) -> Result<Program, Vec<Diagnostic>> {
    // For now, run the Logos lexer first so lexer errors are caught.
    // The Chumsky parser below still parses the source text directly.
    echo_lexer::lex(source)?;

    parser().parse(source).into_result().map_err(|errors| {
        errors
            .into_iter()
            .map(|error| {
                let span = error.span();
                Diagnostic::new(error.to_string(), Span::new(span.start, span.end))
            })
            .collect()
    })
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

    let function_call_expr = text::ident()
        .padded()
        .then_ignore(just('(').padded())
        .then_ignore(just(')').padded())
        .map_with(|name: &str, extra| {
            let span: SimpleSpan = extra.span();

            Expr::FunctionCall(FunctionCallExpr {
                name: name.to_string(),
                span: Span::new(span.start, span.end),
            })
        });

    let atom = function_call_expr.or(variable).or(string).or(number);

    let expr = atom.clone().foldl(
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
    );

    let echo_exprs = expr
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

    let function_call_stmt = text::ident()
        .padded()
        .then_ignore(just('(').padded())
        .then_ignore(just(')').padded())
        .then_ignore(just(';').padded())
        .map_with(|name: &str, extra| {
            let span: SimpleSpan = extra.span();

            Stmt::FunctionCall(FunctionCallStmt {
                name: name.to_string(),
                span: Span::new(span.start, span.end),
            })
        });

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

    let statement = echo_stmt.or(function_call_stmt).or(assign_stmt);

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
