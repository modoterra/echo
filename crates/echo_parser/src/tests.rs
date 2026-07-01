use super::*;

#[path = "tests/concurrency.rs"]
mod concurrency;
#[path = "tests/echo_surface.rs"]
mod echo_surface;
#[path = "tests/strings.rs"]
mod strings;

#[test]
fn parses_optional_statement_semicolons() {
    let program = parse(
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
fn echo_compat_mode_keeps_parent_self_static_as_php_variables() {
    let program = parse(
        r#"<?php
$parent = $class->getParentClass();
$self = "value";
$static = $parent->getName();
"#,
    )
    .expect("PHP-compatible dollar-prefixed names parse as variables");

    assert!(matches!(
        &program.statements[0],
        Stmt::Assign(statement)
            if statement.name == "parent"
                && matches!(
                    &statement.value,
                    Expr::MethodCall(call)
                        if matches!(
                            &call.object,
                            Expr::Variable(variable) if variable.name == "class"
                        )
                )
    ));
    assert!(matches!(
        &program.statements[2],
        Stmt::Assign(statement)
            if statement.name == "static"
                && matches!(
                    &statement.value,
                    Expr::MethodCall(call)
                        if matches!(
                            &call.object,
                            Expr::Variable(variable) if variable.name == "parent"
                        )
                )
    ));
}

#[test]
fn echo_compat_mode_parses_php_trait_declaration() {
    let program = parse(
        r#"<?php
trait ReflectsClosures {
    protected function closureReturnTypes(Closure $closure) {
        return [];
    }
}
"#,
    )
    .expect("PHP trait declaration parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::TraitDecl(statement)
            if statement.name == "ReflectsClosures"
                && matches!(
                    &statement.members[0],
                    ClassMember::Method(method) if method.name == "closureReturnTypes"
                )
    ));
}

#[test]
fn parses_php_interface_declaration() {
    let program = parse(
        r#"<?php
interface Template extends Renderable, Stringable {
    public const DEFAULT_VIEW = "index";

    public function setVariable(string $name, mixed $value): void;
    public function getHtml($template);
}
"#,
    )
    .expect("PHP interface declaration parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::InterfaceDecl(statement)
            if statement.name == "Template"
                && statement.parents
                    == [
                        QualifiedName::new(vec!["Renderable".to_string()]),
                        QualifiedName::new(vec!["Stringable".to_string()]),
                    ]
                && matches!(
                    &statement.members[..],
                    [
                        InterfaceMember::Const(constant),
                        InterfaceMember::Method(set_variable),
                        InterfaceMember::Method(get_html),
                    ] if constant.name == "DEFAULT_VIEW"
                        && set_variable.name == "setVariable"
                        && set_variable.return_type.as_deref() == Some("void")
                        && get_html.name == "getHtml"
                )
    ));
}

#[test]
fn parses_php_unit_enum_declaration() {
    let program = parse(
        r#"<?php
enum Status {
    case Draft;
    case Published;
}
"#,
    )
    .expect("PHP unit enum declaration parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::EnumDecl(statement)
            if statement.name == "Status"
                && statement.backing_type.is_none()
                && matches!(
                    &statement.members[..],
                    [
                        EnumMember::Case(first),
                        EnumMember::Case(second),
                    ] if first.name == "Draft"
                        && first.value.is_none()
                        && second.name == "Published"
                        && second.value.is_none()
                )
    ));
}

#[test]
fn parses_php_backed_enum_with_members() {
    let program = parse(
        r#"<?php
enum HttpMethod: string implements Stringable {
    use HasLabel;

    case Get = "GET";
    case Post = "POST";

    public function label(): string {
        return $this->value;
    }
}
"#,
    )
    .expect("PHP backed enum declaration parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::EnumDecl(statement)
            if statement.name == "HttpMethod"
                && statement.backing_type.as_deref() == Some("string")
                && statement.interfaces == [QualifiedName::new(vec!["Stringable".to_string()])]
                && matches!(
                    &statement.members[..],
                    [
                        EnumMember::TraitUse(name),
                        EnumMember::Case(get),
                        EnumMember::Case(post),
                        EnumMember::Method(method),
                    ] if name.parts == ["HasLabel"]
                        && get.name == "Get"
                        && matches!(get.value, Some(Expr::String(_)))
                        && post.name == "Post"
                        && matches!(post.value, Some(Expr::String(_)))
                        && method.name == "label"
                )
    ));
}

#[test]
fn preserves_multiline_concat_expressions() {
    let program = parse(
        r#"<?php
$body = "Hello "
    . $name
    . "\n"
echo $body
"#,
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
    let program = parse(
        r#"<?php
$body =
    "Hello " . $name
echo $body
"#,
    )
    .expect("multiline assignment parses");

    assert!(matches!(&program.statements[0], Stmt::Assign(_)));
    assert!(matches!(&program.statements[1], Stmt::Echo(_)));
}

#[test]
fn preserves_multiline_function_calls() {
    let program = parse(
        r#"<?php
strlen(
    "Echo"
)
echo "done"
"#,
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
                if statement.name.parts == ["std", "net"]
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
            if statement.name.parts == ["std", "time"]
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
            if statement.name.parts == ["std", "http"]
    ));
    assert!(matches!(&program.statements[1], Stmt::FunctionDecl(_)));
}

#[test]
fn parses_dotted_std_function_call() {
    let program = parse(
        r#"from std use time
time.sleep(300)
"#,
    )
    .expect("dotted function call parses");

    assert!(matches!(
        &program.statements[1],
        Stmt::FunctionCall(statement) if statement.name == "time.sleep"
    ));
}

#[test]
fn parses_echo_module_and_direct_dotted_imports() {
    let program = parse(
        r#"module acme.http_server.runtime
use std.http
use std.net
use acme.http_server.ServerKernel
"#,
    )
    .expect("Echo module and direct dotted imports parse");

    assert!(matches!(
        &program.statements[0],
        Stmt::Namespace(statement)
            if statement.source == NamespaceSource::Echo
                && statement.name.parts == ["acme", "http_server", "runtime"]
    ));
    assert!(matches!(
        &program.statements[1],
        Stmt::Import(statement)
            if statement.source == ImportSource::Std
                && statement.name.parts == ["http"]
    ));
    assert!(matches!(
        &program.statements[2],
        Stmt::Import(statement)
            if statement.source == ImportSource::Std
                && statement.name.parts == ["net"]
    ));
    assert!(matches!(
        &program.statements[3],
        Stmt::Use(statement)
            if statement.name.parts == ["acme", "http_server", "ServerKernel"]
    ));
}

#[test]
fn parses_compile_declaration_entries() {
    let program = parse(
        r#"module app.bootstrap
compile {
    "./routes/*.php"
    "/srv/app/shared/bootstrap.php"
    "modoterra/laravel-echo"
}
echo "ready"
"#,
    )
    .expect("compile declaration parses");

    assert!(matches!(
        &program.statements[1],
        Stmt::Compile(statement)
            if statement.entries.iter().map(|entry| entry.value.as_str()).collect::<Vec<_>>()
                == ["./routes/*.php", "/srv/app/shared/bootstrap.php", "modoterra/laravel-echo"]
    ));
}

#[test]
fn rejects_compile_after_executable_statement() {
    let diagnostics = parse(
        r#"echo "ready"
compile { "./routes/*.php" }
"#,
    )
    .expect_err("compile after executable statement is rejected");

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("before executable statements")),
        "{diagnostics:#?}"
    );
}

#[test]
fn rejects_compile_before_module_declaration() {
    let diagnostics = parse(
        r#"compile { "./routes/*.php" }
module app.bootstrap
"#,
    )
    .expect_err("module after compile should still violate the module prelude rule");

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("module")),
        "{diagnostics:#?}"
    );
}

#[test]
fn parses_include_and_include_once_expressions() {
    parse(
        r#"<?php
include __DIR__ . "/config.php";
$loaded = include_once __DIR__ . "/config.php";
"#,
    )
    .expect("include and include_once expressions should parse");
}

#[test]
fn parses_echo_http_server_surface() {
    parse(
        r#"module acme.http_server.runtime

use std.http
use std.net
use acme.http_server.ServerKernel

let $app = require_once config("server.paths.bootstrap")
let $kernel = ServerKernel.from($app)
let $address = config("echo.server.host", "127.0.0.1") . ":" . config("echo.server.port", 8080)
let $server = net.listen($address)

loop {
    let $connection = net.accept($server)
    let $request = http.readRequest($connection)
    let $response = $kernel.handle($request)

    net.write($connection, http.toBytes($response))
    net.close($connection)
}
"#,
    )
    .expect("Echo HTTP server syntax parses for LSP diagnostics");
}

#[test]
fn parses_echo_package_provider_surface() {
    parse(
        r#"class ReportFormatter {
    fn slug($name): string {
        return $name
    }

    pub fn title($name): string {
        return $this.slug($name)
    }
}
"#,
    )
    .expect("Echo package provider syntax parses for LSP diagnostics");
}

#[test]
fn parses_class_method_bodies_as_statements() {
    let program = parse(
        r#"<?php
class Example {
    public static function getLoader()
    {
        return "loader";
    }
}
"#,
    )
    .expect("class method body should parse");

    let Stmt::ClassDecl(class) = &program.statements[0] else {
        panic!("expected class declaration");
    };
    let echo_ast::ClassMember::Method(method) = &class.members[0] else {
        panic!("expected method declaration");
    };
    assert_eq!(method.name, "getLoader");
    assert_eq!(method.body.len(), 1);
    assert!(matches!(method.body[0], Stmt::Return(_)));
}

#[test]
fn rejects_plain_use_std_module_import_spelling() {
    parse(
        r#"use std time
time.sleep(300)
"#,
    )
    .expect_err("std imports must use `from std use ...`");
}

#[test]
fn parses_php_use_std_namespace_as_php_use_statement() {
    let program = parse(
        r#"use std\time
time.sleep(300)
"#,
    )
    .expect("PHP namespace use syntax remains parseable");

    assert!(matches!(&program.statements[0], Stmt::Use(statement)
        if statement.name.as_string() == "std\\time"));
    assert!(matches!(
        &program.statements[1],
        Stmt::FunctionCall(statement) if statement.name == "time.sleep"
    ));
}

#[test]
fn parses_negative_numeric_function_arguments() {
    let program = parse(r#"<?php echo substr_compare("abcde", "de", -2, 2);"#)
        .expect("negative numeric argument parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::Echo(statement)
            if matches!(
                &statement.exprs[0],
                Expr::FunctionCall(call)
                    if matches!(
                        &call.args[2].value,
                        Expr::Unary(expr)
                            if expr.op == UnaryOp::Minus
                                && matches!(&expr.expr, Expr::Number(number) if number.value == "2")
                    )
            )
    ));
}

#[test]
fn parses_strict_identity_comparison_in_if_condition() {
    let program = parse(
        r#"$payload = "signed:user-42";
$parts = explode(":", $payload);

if (count($parts) === 2) {
    echo strtoupper($parts[1]) . "\n";
}
"#,
    )
    .expect("strict identity comparison in if condition parses");

    assert!(matches!(
        &program.statements[2],
        Stmt::If(statement)
            if matches!(
                &statement.condition,
                Expr::Binary(expr) if expr.op == BinaryOp::Identical
            )
    ));
}

#[test]
fn parses_php_new_expression_as_method_argument() {
    let program = parse(
        r#"<?php
$status = $app->handleCommand(new ArgvInput);
"#,
    )
    .expect("new expression parses as a normal expression");

    assert!(matches!(
        &program.statements[0],
        Stmt::Assign(statement)
            if matches!(
                &statement.value,
                Expr::MethodCall(call)
                    if call.method == "handleCommand"
                        && matches!(
                            &call.args[0].value,
                            Expr::New(new_expr)
                                if matches!(
                                    &new_expr.target,
                                    echo_ast::NewTarget::Class(class_name)
                                        if class_name.as_string() == "ArgvInput"
                                )
                        )
            )
    ));
}

#[test]
fn parses_php_dynamic_new_expression() {
    let program = parse(
        r#"<?php
$instance = new $provider($app);
"#,
    )
    .expect("dynamic new expression parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::Assign(statement)
            if matches!(
                &statement.value,
                Expr::New(new_expr)
                    if matches!(
                        &new_expr.target,
                        echo_ast::NewTarget::Expr(target)
                            if matches!(target.as_ref(), Expr::Variable(variable) if variable.name == "provider")
                    )
            )
    ));
}

#[test]
fn parses_php_not_equal_expression() {
    let program = parse(
        r#"<?php
return $manifest['providers'] != $providers;
"#,
    )
    .expect("PHP != expression parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::Return(statement)
            if matches!(
                &statement.value,
                Some(Expr::Binary(expr)) if expr.op == BinaryOp::NotEqual
            )
    ));
}

#[test]
fn parses_php_index_append_assignment() {
    let program = parse(
        r#"<?php
$manifest['eager'][] = $provider;
"#,
    )
    .expect("PHP indexed append assignment parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::Append(statement)
            if matches!(&statement.target, Expr::Index(_))
                && matches!(&statement.value, Expr::Variable(variable) if variable.name == "provider")
    ));
}

#[test]
fn parses_php_shorthand_ternary_expression() {
    let program = parse(
        r#"<?php
$vendorPath = Env::get('COMPOSER_VENDOR_DIR') ?: $basePath.'/vendor';
"#,
    )
    .expect("PHP shorthand ternary parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::Assign(statement)
            if matches!(&statement.value, Expr::Ternary(expr) if expr.condition == expr.if_true)
    ));
}

#[test]
fn parses_php_null_coalesce_expression() {
    let program = parse(
        r#"<?php
return $configuration[$key] ?? [];
"#,
    )
    .expect("PHP null coalesce parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::Return(statement)
            if matches!(
                &statement.value,
                Some(Expr::Binary(expr)) if expr.op == BinaryOp::Coalesce
            )
    ));
}

#[test]
fn parses_php_named_static_call_argument() {
    let program = parse(r#"<?php return Bootstrapper::configure(basePath: dirname(__DIR__));"#)
        .expect("PHP named static call argument parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::Return(statement)
            if matches!(
                &statement.value,
                Some(Expr::StaticCall(call))
                    if call.class_name.as_string() == "Bootstrapper"
                        && call.method == "configure"
                        && matches!(call.args[0].name.as_deref(), Some("basePath"))
                        && matches!(&call.args[0].value, Expr::FunctionCall(inner) if inner.name == "dirname")
            )
    ));
}

#[test]
fn parses_php_closure_expression_as_method_argument() {
    let program = parse(
        r#"<?php
return Bootstrapper::configure(basePath: dirname(__DIR__))
    ->withMiddleware(function (Pipeline $middleware): void {
    })
    ->create();
"#,
    )
    .expect("PHP closure expression parses as a method argument");

    assert!(matches!(
        &program.statements[0],
        Stmt::Return(statement)
            if matches!(
                &statement.value,
                Some(Expr::MethodCall(create))
                    if create.method == "create"
                        && matches!(
                            &create.object,
                            Expr::MethodCall(with_middleware)
                                if with_middleware.method == "withMiddleware"
                                    && matches!(
                                        &with_middleware.args[0].value,
                                        Expr::Closure(closure)
                                            if closure.params[0].name == "middleware"
                                                && closure.params[0].ty.as_deref() == Some("Pipeline")
                                                && closure.return_type.as_deref() == Some("void")
                                    )
                        )
            )
    ));
}

#[test]
fn parses_php_closure_with_default_param_and_captures() {
    let program = parse(
        r#"<?php
return function ($container, $parameters = []) use ($abstract, $concrete) {
    return $container->build($concrete);
};
"#,
    )
    .expect("PHP closure default params and captures parse");

    assert!(matches!(
        &program.statements[0],
        Stmt::Return(statement)
            if matches!(
                &statement.value,
                Some(Expr::Closure(closure))
                    if closure.params.len() == 2
                        && closure.params[1].default_value.is_some()
                        && closure.captures == ["abstract", "concrete"]
            )
    ));
}

#[test]
fn parses_php_dynamic_method_call_name() {
    let program = parse(
        r#"<?php
$target->{$method}($instance);
"#,
    )
    .expect("PHP dynamic method call names parse");

    assert!(matches!(
        &program.statements[0],
        Stmt::Expr(statement)
            if matches!(
                &statement.expr,
                Expr::MethodCall(call)
                    if call.method == "${method}"
            )
    ));
}

#[test]
fn parses_php_assignment_expression_in_condition() {
    let program = parse(
        r#"<?php
if (! $callbacks = $this->getCallbacks()) {
    return;
}
"#,
    )
    .expect("PHP assignment expressions parse in conditions");

    assert!(matches!(
        &program.statements[0],
        Stmt::If(statement)
            if matches!(
                &statement.condition,
                Expr::Unary(unary)
                    if matches!(&unary.expr, Expr::Assign(assign) if assign.name == "callbacks")
            )
    ));
}

#[test]
fn parses_php_first_class_callable_placeholder() {
    let program = parse(
        r#"<?php
$reflector = new ReflectionFunction($callback(...));
"#,
    )
    .expect("PHP first-class callable placeholder parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::Assign(statement)
            if matches!(&statement.value, Expr::New(_))
    ));
}

#[test]
fn parses_php_argument_unpacking() {
    let program = parse(
        r#"<?php
$instance = new $concrete(...$instances);
"#,
    )
    .expect("PHP argument unpacking parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::Assign(statement)
            if matches!(&statement.value, Expr::New(_))
    ));
}

#[test]
fn parses_php_continue_statement() {
    let program = parse(
        r#"<?php
foreach ($items as $item) {
    continue;
}
"#,
    )
    .expect("PHP continue statement parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::Foreach(statement)
            if matches!(statement.body.first(), Some(Stmt::Continue(_)))
    ));
}

#[test]
fn parses_php_null_coalescing_assignment_statement() {
    let program = parse(
        r#"<?php
$result ??= fallback();
"#,
    )
    .expect("PHP null-coalescing assignment parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::CoalesceAssign(statement) if statement.name == "result"
    ));
}

#[test]
fn parses_php_static_property_null_coalescing_assignment_expression() {
    let program = parse(
        r#"<?php
class C {
    public static function getInstance() {
        return static::$instance ??= new static;
    }
}
"#,
    )
    .expect("PHP static property null-coalescing assignment parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::ClassDecl(class)
            if matches!(
                &class.members[0],
                ClassMember::Method(method)
                    if matches!(
                        &method.body[0],
                        Stmt::Return(statement)
                            if matches!(
                                &statement.value,
                                Some(Expr::StaticPropertyCoalesceAssign(_))
                            )
                    )
            )
    ));
}

#[test]
fn parses_php_global_qualified_parameter_type() {
    let program = parse(
        r#"<?php
class Hooks {
    public function register(\Closure $callback): void {}
}
"#,
    )
    .expect("PHP global-qualified parameter types parse");

    assert!(matches!(
        &program.statements[0],
        Stmt::ClassDecl(class)
            if matches!(
                &class.members[0],
                ClassMember::Method(method)
                    if method.params[0].ty.as_deref() == Some("Closure")
            )
    ));
}

#[test]
fn parses_php_arrow_function_expression_as_argument() {
    let program = parse(
        r#"<?php
$exceptions->shouldRenderJsonWhen(
    fn (Request $request) => $request->is('api/*'),
);
"#,
    )
    .expect("PHP arrow function expression parses as a method argument");

    assert!(matches!(
        &program.statements[0],
        Stmt::Expr(statement)
            if matches!(
                &statement.expr,
                Expr::MethodCall(call)
                    if call.method == "shouldRenderJsonWhen"
                        && matches!(
                            &call.args[0].value,
                            Expr::ArrowFunction(arrow)
                                if arrow.params[0].name == "request"
                                    && arrow.params[0].ty.as_deref() == Some("Request")
                                    && matches!(&arrow.body, Expr::MethodCall(body) if body.method == "is")
                        )
            )
    ));
}

#[test]
fn parses_elseif_as_explicit_clause() {
    let program = parse(
        r#"<?php
if ($a) {
    echo "a";
} elseif ($b) {
    echo "b";
} else {
    echo "c";
}
"#,
    )
    .expect("elseif parses as an explicit clause");

    assert!(matches!(
        &program.statements[0],
        Stmt::If(statement)
            if statement.elseif_clauses.len() == 1
                && matches!(
                    &statement.elseif_clauses[0].condition,
                    Expr::Variable(variable) if variable.name == "b"
                )
                && statement.else_body.len() == 1
    ));
}

#[test]
fn parses_subtraction_expression() {
    let program = parse("3-5").expect("subtraction parses");

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
    let program = parse("2+3*4").expect("arithmetic parses");

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
    let program = parse("-(2+3)").expect("parenthesized unary parses");

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
    let list = parse("{1, 2, 3}").expect("list literal parses");
    assert!(matches!(
        &list.statements[0],
        Stmt::Expr(statement)
            if matches!(&statement.expr, Expr::List(expr) if expr.values.len() == 3)
    ));

    let object = parse("{ test: 5 }").expect("object literal parses");
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
    let program = parse("[1, 2, 3]").expect("array parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::Expr(statement)
            if matches!(&statement.expr, Expr::Array(expr) if expr.elements.len() == 3)
    ));
}

#[test]
fn parses_index_access_expressions() {
    let program = parse("$a[0]").expect("index access parses");

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
fn parses_php_keyed_arrays() {
    let program = parse(r#"["asdf" => 5]"#).expect("PHP keyed array parses");

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
fn parses_keyed_arrays_in_single_language_mode() {
    let program = parse(r#"["asdf" => 5]"#).expect("keyed arrays parse");

    assert!(matches!(
        &program.statements[0],
        Stmt::Expr(statement) if matches!(statement.expr, Expr::Array(_))
    ));
}

#[test]
fn parses_legacy_std_namespace_declaration_for_now() {
    let program = parse("namespace std net").expect("std namespace parses");

    assert!(matches!(
        &program.statements[0],
        Stmt::Namespace(statement) if statement.source == NamespaceSource::Std
    ));
}

#[test]
fn parses_reserved_std_backslash_namespace_for_semantic_rejection() {
    let program = parse("namespace std\\Net").expect("std backslash namespace should parse");

    assert!(matches!(&program.statements[0], Stmt::Namespace(_)));
}

#[test]
fn parses_concurrency_expression_statements() {
    let program = parse(
        r#"run $task
join $task
"#,
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
fn single_language_accepts_concurrency_keywords_in_php_files() {
    let program = parse(
        r#"<?php
$task = run $deferred;
"#,
    )
    .expect("single language parser accepts concurrency syntax");

    assert!(matches!(
        &program.statements[0],
        Stmt::Assign(statement) if matches!(statement.value, Expr::Run(_))
    ));
}

#[test]
fn single_language_accepts_php_reference_assignment() {
    let program = parse(
        r#"<?php
$a = "x";
$b =& $a;
"#,
    )
    .expect("single language parser accepts PHP references");

    assert!(matches!(&program.statements[1], Stmt::AssignRef(_)));
}

#[test]
fn single_language_accepts_php_array_append_assignment() {
    let program = parse(
        r#"<?php
$a = [];
$a[] = 1;
"#,
    )
    .expect("single language parser accepts PHP append syntax");

    assert!(matches!(&program.statements[1], Stmt::Append(_)));
}

#[test]
fn parses_php_array_append_assignment_for_semantic_validation() {
    let program = parse(
        r#"let $a = []
$a[] = 1
"#,
    )
    .expect("parser accepts append syntax for semantic validation");

    assert!(matches!(&program.statements[1], Stmt::Append(_)));
}

#[test]
fn single_language_accepts_dynamic_function_calls() {
    let program = parse(
        r#"<?php
$fn = "strlen";
$fn("Echo");
"#,
    )
    .expect("single language parser accepts dynamic calls");

    assert!(matches!(
        &program.statements[1],
        Stmt::DynamicFunctionCall(_)
    ));
}

#[test]
fn single_language_accepts_php_class_method_with_visibility_and_body() {
    let program = parse(
        "<?php namespace Acme\\Runtime; class Kernel { public function dispatch($request) { } }",
    )
    .expect("single language parser accepts PHP method bodies");

    assert!(matches!(&program.statements[0], Stmt::Namespace(_)));
    assert!(matches!(
        &program.statements[1],
        Stmt::ClassDecl(statement)
            if statement.name == "Kernel"
                && matches!(
                    &statement.members[0],
                    ClassMember::Method(method) if method.name == "dispatch"
                )
    ));
}

#[test]
fn single_language_accepts_php_typed_class_properties_and_implements_list() {
    let program = parse(
        r#"<?php
class Kernel extends Container implements KernelContract, HttpKernelInterface {
    protected array $pendingProviders = [];
    public static string $name = "app";
}
"#,
    )
    .expect("single language parser accepts PHP typed properties and implements lists");

    assert!(matches!(
        &program.statements[0],
        Stmt::ClassDecl(statement)
            if statement.name == "Kernel"
                && statement.interfaces.len() == 2
                && matches!(
                    &statement.members[0],
                    ClassMember::Property(property)
                        if property.name == "pendingProviders"
                            && property.visibility == MethodVisibility::Protected
                            && !property.is_static
                )
                && matches!(
                    &statement.members[1],
                    ClassMember::Property(property)
                        if property.name == "name"
                            && property.visibility == MethodVisibility::Public
                            && property.is_static
                )
    ));
}

#[test]
fn single_language_accepts_php_class_constants() {
    let program = parse(
        r#"<?php
class Kernel {
    public const VERSION = '1.2.3';
    protected const FLAGS = [];
}
"#,
    )
    .expect("single language parser accepts PHP class constants");

    assert!(matches!(
        &program.statements[0],
        Stmt::ClassDecl(statement)
            if statement.name == "Kernel"
                && matches!(
                    &statement.members[0],
                    ClassMember::Const(constant)
                        if constant.name == "VERSION"
                            && constant.visibility == MethodVisibility::Public
                )
                && matches!(
                    &statement.members[1],
                    ClassMember::Const(constant)
                        if constant.name == "FLAGS"
                            && constant.visibility == MethodVisibility::Protected
                )
    ));
}

#[test]
fn parses_php_class_declaration_modifiers() {
    let program = parse(
        r#"<?php
abstract class BaseFormatter {
    abstract public function format($value);
}

final class JsonFormatter extends BaseFormatter {
}

readonly class DataTransferObject {
    public string $name;
}
"#,
    )
    .expect("PHP class declaration modifiers parse");

    assert!(matches!(
        &program.statements[0],
        Stmt::ClassDecl(statement)
            if statement.name == "BaseFormatter"
                && statement.modifiers == [ClassModifier::Abstract]
    ));
    assert!(matches!(
        &program.statements[1],
        Stmt::ClassDecl(statement)
            if statement.name == "JsonFormatter"
                && statement.modifiers == [ClassModifier::Final]
                && statement.parent == Some(QualifiedName::new(vec!["BaseFormatter".to_string()]))
    ));
    assert!(matches!(
        &program.statements[2],
        Stmt::ClassDecl(statement)
            if statement.name == "DataTransferObject"
                && statement.modifiers == [ClassModifier::Readonly]
    ));
}

#[test]
fn single_language_accepts_php_typed_class_property_with_empty_array_default() {
    let program = parse(
        r#"<?php
class RuntimeBuilder
{
    protected array $pendingProviders = [];
}
"#,
    )
    .expect("single language parser accepts PHP typed properties with [] defaults");

    assert!(matches!(
        &program.statements[0],
        Stmt::ClassDecl(statement)
            if statement.name == "RuntimeBuilder"
                && matches!(
                    &statement.members[0],
                    ClassMember::Property(property) if property.name == "pendingProviders"
                )
    ));
}

#[test]
fn single_language_accepts_php_promoted_constructor_parameters() {
    let program = parse(
        r#"<?php
class RuntimeBuilder
{
    public function __construct(protected Kernel $app)
    {
    }
}
"#,
    )
    .expect("single language parser accepts PHP constructor-promoted parameters");

    assert!(matches!(
        &program.statements[0],
        Stmt::ClassDecl(statement)
            if matches!(
                &statement.members[0],
                ClassMember::Method(method)
                    if method.name == "__construct"
                        && method.params.len() == 1
                        && method.params[0].name == "app"
                        && method.params[0].ty.as_deref() == Some("Kernel")
            )
    ));
}

#[test]
fn single_language_accepts_php_variadic_parameters() {
    let program = parse(
        r#"<?php
class Kernel {
    public function environment(...$environments) {
        return $environments;
    }
}
"#,
    )
    .expect("single language parser accepts PHP variadic parameters");

    assert!(matches!(
        &program.statements[0],
        Stmt::ClassDecl(statement)
            if matches!(
                &statement.members[0],
                ClassMember::Method(method)
                    if method.name == "environment"
                        && method.params.len() == 1
                        && method.params[0].name == "environments"
            )
    ));
}

#[test]
fn single_language_accepts_multiline_php_ternary_condition() {
    let program = parse(
        r#"<?php
$args = $this->runningInConsole() && isset($_SERVER['argv'])
    ? $_SERVER['argv']
    : null;
"#,
    )
    .expect("single language parser accepts multiline PHP ternaries");

    assert!(matches!(
        &program.statements[0],
        Stmt::Assign(statement) if matches!(&statement.value, Expr::Ternary(_))
    ));
}

#[test]
fn single_language_accepts_fully_qualified_php_constants() {
    let program = parse(
        r#"<?php
$running = \PHP_SAPI === 'cli';
"#,
    )
    .expect("single language parser accepts fully-qualified PHP constants");

    assert!(matches!(
        &program.statements[0],
        Stmt::Assign(statement)
            if matches!(&statement.value, Expr::Binary(expr) if matches!(&expr.left, Expr::Constant(constant) if constant.name == "PHP_SAPI"))
    ));
}

#[test]
fn single_language_accepts_php_instanceof_expression() {
    let program = parse(
        r#"<?php
$ok = $value instanceof $name;
"#,
    )
    .expect("single language parser accepts PHP instanceof expressions");

    assert!(matches!(
        &program.statements[0],
        Stmt::Assign(statement)
            if matches!(&statement.value, Expr::Binary(expr) if expr.op == BinaryOp::InstanceOf)
    ));
}

#[test]
fn single_language_accepts_php_loose_equality_expression() {
    let program = parse(
        r#"<?php
if ($code == 404) {
    throw $e;
}
"#,
    )
    .expect("single language parser accepts PHP loose equality expressions");

    assert!(matches!(
        &program.statements[0],
        Stmt::If(statement)
            if matches!(&statement.condition, Expr::Binary(expr) if expr.op == BinaryOp::Equal)
    ));
}

#[test]
fn single_language_accepts_dynamic_expression_call() {
    let program = parse(
        r#"<?php
$callbacks[$index]($this);
"#,
    )
    .expect("single language parser accepts dynamic expression calls");

    assert!(matches!(
        &program.statements[0],
        Stmt::Expr(statement) if matches!(&statement.expr, Expr::DynamicCall(_))
    ));
}

#[test]
fn single_language_accepts_by_ref_parameter_and_post_increment() {
    let program = parse(
        r#"<?php
class Kernel {
    protected function fire(array &$callbacks) {
        $index = 0;
        $index++;
    }
}
"#,
    )
    .expect("single language parser accepts by-ref params and post-increment");

    assert!(matches!(
        &program.statements[0],
        Stmt::ClassDecl(statement)
            if matches!(
                &statement.members[0],
                ClassMember::Method(method)
                    if method.params[0].name == "callbacks"
                        && matches!(method.body[1], Stmt::Expr(_))
            )
    ));
}

#[test]
fn single_language_accepts_php_try_catch_statement() {
    let program = parse(
        r#"<?php
try {
    throw $e;
} catch (\Throwable|RuntimeException $e) {
    echo "failed";
}
"#,
    )
    .expect("single language parser accepts PHP try/catch statements");

    assert!(matches!(
        &program.statements[0],
        Stmt::Try(statement)
            if statement.catches.len() == 1
                && statement.catches[0].types.len() == 2
                && statement.catches[0].variable.as_deref() == Some("e")
    ));
}

#[test]
fn single_language_accepts_php_try_finally_statement() {
    let program = parse(
        r#"<?php
try {
    work();
} finally {
    cleanup();
}
"#,
    )
    .expect("single language parser accepts PHP try/finally statements");

    assert!(matches!(
        &program.statements[0],
        Stmt::Try(statement)
            if statement.catches.is_empty() && statement.finally_body.len() == 1
    ));
}

#[test]
fn single_language_accepts_php_list_destructuring_assignment() {
    let program = parse(
        r#"<?php
[$commands, $paths] = $collection->partition(fn ($command) => class_exists($command));
"#,
    )
    .expect("single language parser accepts PHP list destructuring assignment");

    assert!(matches!(
        &program.statements[0],
        Stmt::ListAssign(statement)
            if statement.targets == ["commands", "paths"]
    ));
}

#[test]
fn single_language_accepts_php_keyed_list_destructuring_assignment() {
    let program = parse(
        r#"<?php
['queue' => $queue, 'job' => $job, 'command' => $command] = $array;
"#,
    )
    .expect("single language parser accepts PHP keyed list destructuring assignment");

    assert!(matches!(
        &program.statements[0],
        Stmt::ListAssign(statement)
            if statement.targets == ["queue", "job", "command"]
    ));
}

#[test]
fn single_language_accepts_arrow_function_as_call_argument() {
    let program = parse(
        r#"<?php
$this->app->afterResolving(Schedule::class, fn ($schedule) => $callback($schedule));
"#,
    )
    .expect("single language parser accepts PHP arrow functions as call arguments");

    assert!(matches!(&program.statements[0], Stmt::Expr(_)));
}

#[test]
fn single_language_accepts_php_use_before_class_declaration() {
    let program = parse(
        r#"<?php
use Acme\Routing\Router;

class RuntimeBuilder
{
}
"#,
    )
    .expect("single language parser accepts PHP use statements before classes");

    assert!(matches!(&program.statements[0], Stmt::Use(_)));
    assert!(matches!(
        &program.statements[1],
        Stmt::ClassDecl(statement) if statement.name == "RuntimeBuilder"
    ));
}

#[test]
fn single_language_accepts_php_use_alias_and_function_import_before_class() {
    let program = parse(
        r#"<?php
namespace Acme\Runtime;

use Closure;
use Acme\Console\CommandApplication as ConsoleApplication;
use function Acme\Filesystem\join_paths;

class Kernel
{
}
"#,
    )
    .expect("single language parser accepts PHP aliased and function imports before classes");

    assert!(matches!(&program.statements[0], Stmt::Namespace(_)));
    assert!(matches!(&program.statements[1], Stmt::Use(_)));
    assert!(matches!(&program.statements[2], Stmt::Use(_)));
    assert!(matches!(&program.statements[3], Stmt::Use(_)));
    assert!(matches!(
        &program.statements[4],
        Stmt::ClassDecl(statement) if statement.name == "Kernel"
    ));
}

#[test]
fn single_language_accepts_fn_class_methods_with_private_default() {
    let program = parse(
        r#"class ReportFormatter {
    fn slug($name): string {
        return $name
    }

    pub fn title($name): string {
        return $name
    }
}
"#,
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
fn parses_dynamic_function_calls_in_single_language_mode() {
    let program = parse(
        r#"let $fn = "strlen"
$fn("Echo")
"#,
    )
    .expect("dynamic calls parse");

    assert!(matches!(
        &program.statements[1],
        Stmt::DynamicFunctionCall(_)
    ));
}

#[test]
fn parses_php_bracketed_namespace_blocks() {
    let program = parse(
        r#"<?php
namespace Illuminate\Http {
class Request {}
}

namespace {
return new Illuminate\Foundation\Application();
}
"#,
    )
    .expect("bracketed PHP namespace blocks parse");

    assert!(matches!(
        &program.statements[0],
        Stmt::Namespace(statement)
            if statement.name.as_string() == "Illuminate\\Http"
    ));
    assert!(matches!(
        &program.statements[1],
        Stmt::ClassDecl(statement) if statement.name == "Request"
    ));
    assert!(matches!(
        &program.statements[2],
        Stmt::Namespace(statement) if statement.name.parts.is_empty()
    ));
    assert!(matches!(&program.statements[3], Stmt::Return(_)));
}
