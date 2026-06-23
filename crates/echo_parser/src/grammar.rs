use chumsky::prelude::*;
use chumsky::span::SimpleSpan;
use echo_ast::{
    AppendStmt, ArrayElement, ArrayExpr, AssignExpr, AssignRefStmt, AssignStmt, BinaryExpr,
    BinaryOp, BoolLiteral, BreakStmt, ClassDeclStmt, ClassMember, DeferExpr,
    DynamicFunctionCallStmt, EchoStmt, Expr, ExtendDeclStmt, FieldExpr, ForkExpr, FunctionCallExpr,
    FunctionCallStmt, FunctionDeclStmt, IfStmt, ImportSource, ImportStmt, IndexExpr, JoinExpr,
    LetStmt, ListExpr, LoopExpr, LoopStmt, MagicConstantExpr, MagicConstantKind, MethodCallExpr,
    MethodDecl, MethodVisibility, NamespaceSource, NamespaceStmt, NewExpr, NullLiteral,
    NumberLiteral, ObjectExpr, ObjectField, Program, QualifiedName, ReceiverConst,
    ReceiverConstExpr, RequireExpr, RequireKind, ReturnStmt, RunExpr, SpawnExpr, StaticCallExpr,
    Stmt, StringLiteral, TypeAscriptionExpr, TypeDeclStmt, TypeField, TypedParam, UnaryExpr,
    UnaryOp, UseStmt, VariableExpr, YieldStmt,
};
use echo_diagnostics::Diagnostic;
use echo_source::{SourceMode, Span};
use echo_syntax::keywords as kw;

#[path = "preprocess.rs"]
mod preprocess;
#[path = "validation.rs"]
mod validation;

use preprocess::{
    normalize_heredoc_literals, strip_comments_preserving_spans, unescape_double_quoted_string,
    unescape_single_quoted_string, virtualize_statement_terminators,
};
use validation::{ValidationMode, validate_mode};

pub fn parse(source: &str) -> Result<Program, Vec<Diagnostic>> {
    parse_with_mode(source, SourceMode::Echo)
}

pub fn parse_with_mode(source: &str, mode: SourceMode) -> Result<Program, Vec<Diagnostic>> {
    parse_with_validation(source, ValidationMode::from_source_mode(mode))
}

pub fn parse_trusted_std(source: &str) -> Result<Program, Vec<Diagnostic>> {
    parse_with_validation(source, ValidationMode::TrustedStd)
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

fn parser<'src>() -> impl Parser<'src, &'src str, Program, extra::Err<Rich<'src, char>>> {
    let open_php = just("<?php")
        .or(just("<?PHP"))
        .map_with(|_, extra| {
            let span: SimpleSpan = extra.span();
            Span::new(span.start, span.end)
        })
        .padded()
        .or_not();

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
    });

    let expr = recursive(|expr| {
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

        let new_expr = text::keyword(kw::NEW.text)
            .padded()
            .ignore_then(
                text::ident()
                    .separated_by(just('\\'))
                    .at_least(1)
                    .collect::<Vec<_>>()
                    .map(|parts| {
                        QualifiedName::new(parts.into_iter().map(str::to_string).collect())
                    })
                    .padded(),
            )
            .then(
                just('(')
                    .padded()
                    .then_ignore(just(')').padded())
                    .to(())
                    .or_not(),
            )
            .map_with(|(class_name, _): (QualifiedName, Option<()>), extra| {
                let span: SimpleSpan = extra.span();

                Expr::New(Box::new(NewExpr {
                    class_name,
                    args: Vec::new(),
                    span: Span::new(span.start, span.end),
                }))
            })
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
            });

        let spawn_expr = text::keyword(kw::SPAWN.text)
            .padded()
            .ignore_then(expr.clone())
            .map_with(|command, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Spawn(SpawnExpr {
                    command: Box::new(command),
                    span: Span::new(span.start, span.end),
                })
            });

        let fork_expr = text::keyword(kw::FORK.text)
            .padded()
            .ignore_then(expr.clone())
            .map_with(|task, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Fork(ForkExpr::Task {
                    expr: Box::new(task),
                    span: Span::new(span.start, span.end),
                })
            });

        let join_expr = text::keyword(kw::JOIN.text)
            .padded()
            .ignore_then(expr.clone())
            .map_with(|handle, extra| {
                let span: SimpleSpan = extra.span();

                Expr::Join(JoinExpr {
                    handle: Box::new(handle),
                    span: Span::new(span.start, span.end),
                })
            });

        let require_expr = text::keyword(kw::REQUIRE_ONCE.text)
            .to(RequireKind::RequireOnce)
            .or(text::keyword(kw::REQUIRE.text).to(RequireKind::Require))
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

        let atom = choice((
            run_expr,
            fork_expr,
            spawn_expr,
            join_expr,
            require_expr,
            new_expr,
            parenthesized,
            structural_object_expr,
            object_expr,
            static_call_expr,
            function_call_expr,
            variable,
            magic_dir,
            null,
            bool_literal,
            array_expr,
            list_expr,
            string,
            number,
        ))
        .boxed();

        #[derive(Clone)]
        enum Postfix {
            MethodCall { method: String, args: Vec<Expr> },
            TypeAscription(String),
            Field(String),
            Index(Expr),
        }

        let arrow_method_call_postfix = just("->")
            .ignore_then(text::ident().padded())
            .then_ignore(just('(').padded())
            .then(args.clone())
            .then_ignore(just(')').padded())
            .map(|(method, args): (&str, Vec<Expr>)| Postfix::MethodCall {
                method: method.to_string(),
                args,
            });
        let dot_postfix = just('.')
            .ignore_then(text::ident())
            .then(
                args.clone()
                    .delimited_by(just('(').padded(), just(')').padded())
                    .or_not(),
            )
            .map(|(name, args): (&str, Option<Vec<Expr>>)| match args {
                Some(args) => Postfix::MethodCall {
                    method: name.to_string(),
                    args,
                },
                None => Postfix::Field(name.to_string()),
            });
        let type_ascription_postfix = just(':')
            .padded()
            .ignore_then(type_expr.clone().padded())
            .map(Postfix::TypeAscription);
        let index_postfix = expr
            .clone()
            .padded()
            .delimited_by(just('[').padded(), just(']').padded())
            .map(Postfix::Index);

        let fielded = atom.clone().foldl(
            arrow_method_call_postfix
                .or(dot_postfix)
                .or(type_ascription_postfix)
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
                Postfix::TypeAscription(ty) => {
                    let span = Span::new(left.span().start, left.span().end + ty.len() + 2);

                    Expr::TypeAscription(Box::new(TypeAscriptionExpr {
                        expr: left,
                        ty,
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
        );

        let comparison = is_expr.clone().foldl(
            just("===")
                .to(BinaryOp::Identical)
                .padded()
                .then(is_expr)
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
            .or(comparison)
    });

    let statement = recursive(|statement| {
        let terminator = just(';').padded().ignored().or(end().ignored());

        let qualified_name = text::ident()
            .separated_by(just('\\'))
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|parts| QualifiedName::new(parts.into_iter().map(str::to_string).collect()));

        let dotted_name = text::ident()
            .separated_by(just('.'))
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|parts| QualifiedName::new(parts.into_iter().map(str::to_string).collect()));

        let module_stmt = text::keyword(kw::MODULE.text)
            .padded()
            .ignore_then(dotted_name.clone())
            .then_ignore(terminator.clone())
            .map_with(|name, extra| {
                let span: SimpleSpan = extra.span();

                Stmt::Namespace(NamespaceStmt {
                    source: NamespaceSource::Php,
                    name,
                    span: Span::new(span.start, span.end),
                })
            });

        let namespace_stmt = text::keyword(kw::NAMESPACE.text)
            .padded()
            .ignore_then(
                text::keyword(kw::STD.text)
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

        let use_stmt = text::keyword(kw::USE.text)
            .padded()
            .ignore_then(qualified_name.clone())
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
            });

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
            });

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
            });

        let import_source = text::keyword(kw::STD.text)
            .padded()
            .to(ImportSource::Std)
            .or(just('"')
                .ignore_then(none_of('"').repeated().collect::<String>())
                .then_ignore(just('"'))
                .padded()
                .map(ImportSource::File));

        let import_stmt = text::keyword(kw::FROM.text)
            .padded()
            .ignore_then(import_source)
            .then_ignore(text::keyword(kw::USE.text).padded())
            .then(qualified_name.clone())
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

        let prefix_typed_param = type_expr
            .clone()
            .padded()
            .or_not()
            .then_ignore(just('$'))
            .then(text::ident().padded())
            .map(|(ty, name): (Option<String>, &str)| TypedParam {
                name: name.to_string(),
                ty,
            });

        let suffix_typed_param = just('$')
            .ignore_then(text::ident().padded())
            .then(
                just(':')
                    .padded()
                    .ignore_then(type_expr.clone().padded())
                    .or_not(),
            )
            .map(|(name, ty): (&str, Option<String>)| TypedParam {
                name: name.to_string(),
                ty,
            });

        let typed_param = suffix_typed_param.or(prefix_typed_param);

        let method_param = typed_param.clone();

        let method_body = recursive(|body| {
            none_of("{}")
                .ignored()
                .or(body)
                .repeated()
                .ignored()
                .delimited_by(just('{').padded(), just('}').padded())
                .ignored()
        });
        let method_end = method_body
            .then_ignore(terminator.clone().or_not())
            .or(terminator.clone())
            .ignored();

        let method_visibility = text::keyword(kw::PUB.text)
            .to(MethodVisibility::Public)
            .or(text::keyword(kw::PUBLIC.text).to(MethodVisibility::Public))
            .or(text::keyword(kw::PROTECTED.text).to(MethodVisibility::Protected))
            .or(text::keyword(kw::PRIVATE.text).to(MethodVisibility::Private))
            .padded()
            .or_not();

        let function_keyword = text::keyword(kw::FN.text)
            .or(text::keyword(kw::FUNCTION.text))
            .padded();

        let method_decl = method_visibility
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
            .then_ignore(method_end)
            .map_with(
                |(((((visibility, is_intrinsic), is_static), name), params), return_type): (
                    (
                        (((Option<MethodVisibility>, bool), bool), &str),
                        Vec<TypedParam>,
                    ),
                    Option<String>,
                ),
                 extra| {
                    let span: SimpleSpan = extra.span();

                    ClassMember::Method(MethodDecl {
                        name: name.to_string(),
                        params,
                        return_type,
                        body: Vec::new(),
                        visibility: visibility.unwrap_or(MethodVisibility::Private),
                        is_static,
                        is_intrinsic,
                        span: Span::new(span.start, span.end),
                    })
                },
            );

        let class_decl_stmt = just(kw::CLASS.text)
            .padded()
            .ignore_then(text::ident().padded())
            .then(
                text::keyword("extends")
                    .padded()
                    .ignore_then(qualified_name.clone().or(dotted_name.clone()))
                    .or_not(),
            )
            .then_ignore(just('{').padded())
            .then(method_decl.clone().repeated().collect::<Vec<_>>())
            .then_ignore(just('}').padded())
            .then_ignore(terminator.clone().or_not())
            .map_with(
                |((name, parent), members): ((&str, Option<QualifiedName>), Vec<ClassMember>),
                 extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::ClassDecl(ClassDeclStmt {
                        name: name.to_string(),
                        parent,
                        members,
                        span: Span::new(span.start, span.end),
                    })
                },
            );

        let extend_decl_stmt = just("extend")
            .padded()
            .ignore_then(qualified_name.clone().or(dotted_name.clone()))
            .then_ignore(just('{').padded())
            .then(method_decl.repeated().collect::<Vec<_>>())
            .then_ignore(just('}').padded())
            .then_ignore(terminator.clone().or_not())
            .map_with(
                |(target, members): (QualifiedName, Vec<ClassMember>), extra| {
                    let span: SimpleSpan = extra.span();

                    Stmt::ExtendDecl(ExtendDeclStmt {
                        target,
                        members,
                        span: Span::new(span.start, span.end),
                    })
                },
            );

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
            );

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

        let fn_decl_stmt = text::keyword(kw::FN.text)
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

        let intrinsic_function_decl_stmt = text::keyword(kw::INTRINSIC.text)
            .padded()
            .ignore_then(
                text::keyword(kw::FN.text)
                    .or(text::keyword(kw::FUNCTION.text))
                    .padded(),
            )
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

        let gen_fn_decl_stmt = text::keyword(kw::GEN.text)
            .padded()
            .ignore_then(text::keyword(kw::FN.text).padded())
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
            );

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
            );

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
            );

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
            );

        let run_group_entries = block
            .clone()
            .padded()
            .separated_by(just(',').padded())
            .allow_trailing()
            .at_least(1)
            .collect::<Vec<_>>()
            .delimited_by(just('[').padded(), just(']').padded());

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
            );

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
            });

        let if_stmt = text::keyword(kw::IF.text)
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
            });

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
            });

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
            });

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
            });

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

        class_decl_stmt
            .or(extend_decl_stmt)
            .or(gen_fn_decl_stmt)
            .or(fn_decl_stmt)
            .or(function_decl_stmt)
            .or(intrinsic_function_decl_stmt)
            .or(type_decl_stmt)
            .or(module_stmt)
            .or(echo_std_use_stmt)
            .or(dotted_use_stmt)
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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
