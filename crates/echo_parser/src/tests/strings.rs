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
