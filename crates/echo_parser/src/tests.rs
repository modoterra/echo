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
    let program =
        parse_trusted_std(include_str!("../../../std/net.echo")).expect("std net module parses");

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
    let program =
        parse_trusted_std(include_str!("../../../std/time.echo")).expect("std time module parses");

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
    let program =
        parse_trusted_std(include_str!("../../../std/http.echo")).expect("std http module parses");

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

    let object = parse_with_mode("{ test: 5 }", SourceMode::Strict).expect("object literal parses");
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
fn parses_dot_receiver_method_call() {
    let program = parse_with_mode(r#"$items.push("first")"#, SourceMode::Strict)
        .expect("dot receiver method call parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::Expr(statement)
            if matches!(
                &statement.expr,
                Expr::MethodCall(expr)
                    if expr.method == "push"
                        && matches!(&expr.object, Expr::Variable(variable) if variable.name == "items")
                        && expr.args.len() == 1
            )
    ));
}

#[test]
fn parses_type_ascribed_structural_literal_argument() {
    let program = parse_with_mode(
        r#"$users.push({
    id: 1
    email: "first@example.test"
}: User)"#,
        SourceMode::Strict,
    )
    .expect("type-ascribed structural literal parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::Expr(statement)
            if matches!(
                &statement.expr,
                Expr::MethodCall(expr)
                    if expr.method == "push"
                        && matches!(&expr.args[0], Expr::TypeAscription(ascription) if ascription.ty == "User")
            )
    ));
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
    let program = parse_with_mode("let $users: list<User> = {}", SourceMode::Strict)
        .expect("typed let parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::Let(statement) if statement.name == "users" && statement.ty.as_deref() == Some("list<User>")
    ));
}

#[test]
fn rejects_legacy_prefix_typed_let_annotation() {
    parse_with_mode("let list<User> $users = {}", SourceMode::Strict)
        .expect_err("typed let annotations must follow the symbol");
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

        $users.push(User {
            id: count($users) + 1
            email: "visitor" . count($users) . "@echo.local"
        })

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

    $users.push(User {
        id: count($users) + 1
        email: "visitor" . count($users) . "@echo.local"
    })

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
fn echo_mode_accepts_php_class_method_with_visibility_and_body() {
    let program = parse_with_mode(
            "<?php namespace Illuminate\\Foundation; class Application { public function handleRequest($request) { } }",
            SourceMode::Echo,
        )
        .expect("Echo superset mode accepts PHP method bodies");

    assert!(matches!(&program.statements[0], Stmt::Namespace(_)));
    assert!(matches!(
        &program.statements[1],
        Stmt::ClassDecl(statement)
            if statement.name == "Application"
                && matches!(
                    &statement.members[0],
                    ClassMember::Method(method) if method.name == "handleRequest"
                )
    ));
}

#[test]
fn echo_mode_accepts_fn_class_methods_with_private_default() {
    let program = parse_with_mode(
        r#"class ReportFormatter {
    fn slug($name): string {
        return $name
    }

    pub fn title($name): string {
        return $name
    }
}
"#,
        SourceMode::Strict,
    )
    .expect("Echo class fn methods parse");

    assert!(matches!(
        &program.statements[0],
        Stmt::ClassDecl(statement)
            if statement.name == "ReportFormatter"
                && matches!(
                    &statement.members[0],
                    ClassMember::Method(method)
                        if method.name == "slug"
                            && method.visibility == MethodVisibility::Private
                )
                && matches!(
                    &statement.members[1],
                    ClassMember::Method(method)
                        if method.name == "title"
                            && method.visibility == MethodVisibility::Public
                )
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
