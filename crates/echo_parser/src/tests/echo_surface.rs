use super::super::*;

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
fn parses_receiver_constants_as_special_expressions() {
    let program = parse_with_mode(r#"db.insert($self.table(), $this)"#, SourceMode::Strict)
        .expect("receiver constants parse in expressions");

    assert!(matches!(
        &program.statements[0],
        Stmt::FunctionCall(call)
            if matches!(
                &call.args[0].value,
                Expr::MethodCall(method_call)
                    if matches!(
                        &method_call.object,
                        Expr::ReceiverConst(receiver)
                            if receiver.kind == ReceiverConst::SelfType
                    )
            )
            && matches!(
                &call.args[1].value,
                Expr::ReceiverConst(receiver)
                    if receiver.kind == ReceiverConst::This
            )
    ));
}

#[test]
fn rejects_strict_receiver_constant_assignment() {
    let diagnostics = parse_with_mode("$self = Other", SourceMode::Strict)
        .expect_err("strict receiver constant assignment should fail");

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message
        == "$self is a compiler-provided receiver constant and cannot be assigned."));
}

#[test]
fn rejects_strict_receiver_constant_parameter() {
    let diagnostics = parse_with_mode("fn bad($parent): void {}", SourceMode::Strict)
        .expect_err("strict receiver constant parameter should fail");

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message
        == "$parent is a compiler-provided receiver constant and cannot be declared."));
}

#[test]
fn parses_unnamed_export_object() {
    let program = parse_with_mode(
        r#"module app.config

pub {
    server: {
        host: "127.0.0.1"
        port: 8080
    }
}
"#,
        SourceMode::Strict,
    )
    .expect("unnamed export object parses");

    assert!(matches!(
        &program.statements[1],
        Stmt::UnnamedExport(statement)
            if matches!(&statement.value, Expr::Object(object) if object.fields.len() == 1)
    ));
}

#[test]
fn parses_extend_methods_with_receiver_constants() {
    let program = parse_with_mode(
        r#"extend Instant {
    pub fn add($duration: Duration): Instant {
        return time.add($this, $duration)
    }
}
"#,
        SourceMode::Strict,
    )
    .expect("extend methods parse");

    assert!(matches!(
        &program.statements[0],
        Stmt::ExtendDecl(statement)
            if statement.target.as_string() == "Instant"
                && matches!(
                    &statement.members[0],
                    ClassMember::Method(method)
                        if method.name == "add"
                            && method.params[0].name == "duration"
                            && method.params[0].ty.as_deref() == Some("Duration")
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
                        && matches!(&expr.args[0].value, Expr::TypeAscription(ascription) if ascription.ty == "User")
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
        include_str!("../../../../tests/echo/011_http_server_target/program.echo"),
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
