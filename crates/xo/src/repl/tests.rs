use super::*;

#[test]
fn repl_prompt_uses_xo_paren_with_ansi_color() {
    assert_eq!(repl_prompt(), "\x1b[32mxo)\x1b[0m ");
}

#[test]
fn repl_expression_info_describes_addition() {
    let source = source_file_from_text(
        PathBuf::from("repl.echo"),
        "5+3".to_string(),
        ModeOverride {
            strict: false,
            unsafe_mode: false,
        },
    );
    let parsed = try_parse_repl_input(&source).expect("expression should parse");
    let ReplInput::Expression(info) = parsed.input else {
        panic!("bare expression should be classified as expression input");
    };

    assert_eq!(info.kind, "add expression");
    assert_eq!(info.static_type, "number");
    assert_eq!(info.span.start, 0);
    assert_eq!(info.span.end, 3);
}

#[test]
fn repl_expression_info_describes_subtraction() {
    let source = source_file_from_text(
        PathBuf::from("repl.echo"),
        "3-5".to_string(),
        ModeOverride {
            strict: false,
            unsafe_mode: false,
        },
    );
    let parsed = try_parse_repl_input(&source).expect("expression should parse");
    let ReplInput::Expression(info) = parsed.input else {
        panic!("bare expression should be classified as expression input");
    };

    assert_eq!(info.kind, "subtract expression");
    assert_eq!(info.static_type, "number");
    assert_eq!(info.span.start, 0);
    assert_eq!(info.span.end, 3);
}

#[test]
fn repl_expression_info_reflects_bare_function_call_return_type() {
    let source = source_file_from_text(
        PathBuf::from("repl.echo"),
        "is_float(42)".to_string(),
        ModeOverride {
            strict: false,
            unsafe_mode: false,
        },
    );
    let parsed = try_parse_repl_input(&source).expect("function call should parse");
    let ReplInput::Expression(info) = parsed.input else {
        panic!("bare function call should be classified as expression input");
    };

    assert_eq!(info.kind, "function call");
    assert_eq!(info.static_type, "bool");
    assert_eq!(info.span.start, 0);
    assert_eq!(info.span.end, 12);
}

#[test]
fn repl_expression_info_distinguishes_collection_literals() {
    let cases = [
        ("{1, 2}", "list expression", "list"),
        ("[1, 2]", "array expression", "array"),
        ("{ test: 5 }", "object expression", "object"),
    ];

    for (source, expected_kind, expected_type) in cases {
        let source = source_file_from_text(
            PathBuf::from("repl.echo"),
            source.to_string(),
            ModeOverride {
                strict: false,
                unsafe_mode: false,
            },
        );
        let parsed = try_parse_repl_input(&source).expect("expression should parse");
        let ReplInput::Expression(info) = parsed.input else {
            panic!("bare expression should be classified as expression input");
        };

        assert_eq!(info.kind, expected_kind);
        assert_eq!(info.static_type, expected_type);
    }
}

#[test]
fn repl_expression_info_uses_shared_semantics_for_variables() {
    let first = source_file_from_text(
        PathBuf::from("repl.echo"),
        "let $a = [];".to_string(),
        ModeOverride {
            strict: false,
            unsafe_mode: false,
        },
    );
    let second = source_file_from_text(
        PathBuf::from("repl.echo"),
        "$a".to_string(),
        ModeOverride {
            strict: false,
            unsafe_mode: false,
        },
    );
    let first = try_parse_repl_input(&first).expect("let should parse");
    let mut second = try_parse_repl_input(&second).expect("variable should parse");
    let mut program = second.program.clone();
    let mut statements = first.program.statements;
    statements.extend(program.statements);
    program.statements = statements;
    let analysis = echo_semantics::analyze(&program).expect("session should analyze");

    apply_repl_semantics(&mut second.input, &analysis);

    let ReplInput::Expression(info) = second.input else {
        panic!("bare variable should be expression input");
    };
    assert_eq!(info.kind, "variable");
    assert_eq!(info.static_type, "array");
}

#[test]
fn repl_live_process_join_reports_int_type() {
    let mode = ModeOverride {
        strict: false,
        unsafe_mode: false,
    };
    let first = source_file_from_text(
        PathBuf::from("repl.echo"),
        "$proc = spawn \"exit 7\"".to_string(),
        mode,
    );
    let second = source_file_from_text(PathBuf::from("repl.echo"), "join $proc".to_string(), mode);
    let mut first = try_parse_repl_input(&first).expect("spawn should parse");
    let mut second = try_parse_repl_input(&second).expect("join should parse");
    let mut session = ReplSession::default();

    let output = run_repl_process_handle_input(&mut session, &first.program, &mut first.input)
        .expect("spawn assignment should be handled by live process path");
    assert_eq!(output.status, 0);

    let output = run_repl_process_handle_input(&mut session, &second.program, &mut second.input)
        .expect("join should be handled by live process path");
    assert_eq!(output.status, 0);
    assert_eq!(output.stdout, b"7");

    let ReplInput::Expression(info) = second.input else {
        panic!("join should be expression input");
    };
    assert_eq!(info.kind, "join expression");
    assert_eq!(info.static_type, "int");
}
