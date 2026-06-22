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
