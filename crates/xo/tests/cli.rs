use std::fs;
use std::process::Command;

mod support;

use support::{assert_output_success, assert_stdout_eq, command_output, workspace_root};

#[test]
fn run_forwards_trailing_program_arguments() {
    let dir = std::env::temp_dir().join(format!("xo-cli-args-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("temporary test directory should be created");
    let program = dir.join("program.php");
    fs::write(&program, "<?php\necho \"ok\\n\";\n").expect("test program should be written");

    let mut command = Command::new(env!("CARGO_BIN_EXE_xo"));
    command
        .current_dir(workspace_root())
        .arg("run")
        .arg(&program)
        .arg("echo:start")
        .arg("--verbose");
    let output = command_output(command);

    let _ = fs::remove_file(&program);
    let _ = fs::remove_dir(&dir);

    assert_output_success(&output, "xo run with trailing program arguments");
    assert_stdout_eq(&output.stdout, b"ok\n", "xo run");
}

#[test]
fn tools_grammar_tree_sitter_generates_package_assets() {
    let dir = std::env::temp_dir().join(format!("xo-tree-sitter-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);

    let mut command = Command::new(env!("CARGO_BIN_EXE_xo"));
    command
        .current_dir(workspace_root())
        .arg("tools")
        .arg("grammar")
        .arg("tree-sitter")
        .arg("--output")
        .arg(&dir);
    let output = command_output(command);

    assert_output_success(&output, "xo tools grammar tree-sitter");

    let grammar = fs::read_to_string(dir.join("grammar.js"))
        .expect("tree-sitter grammar should be generated");
    let highlights = fs::read_to_string(dir.join("queries").join("highlights.scm"))
        .expect("tree-sitter highlight query should be generated");

    assert!(
        grammar.contains("module_declaration"),
        "grammar should include Echo module declarations"
    );
    assert!(
        grammar.contains("\"tree-sitter\"") || dir.join("tree-sitter.json").exists(),
        "tree-sitter package metadata should be generated"
    );
    assert!(
        highlights.contains("\"fn\"") && highlights.contains("@keyword"),
        "highlight query should be generated from Echo keyword facts"
    );

    let _ = fs::remove_dir_all(&dir);
}
