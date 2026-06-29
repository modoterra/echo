use std::fs;
use std::process::Command;

mod support;

use serde_json::Value;
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
fn run_reports_source_snippet_and_include_stack_for_compile_errors() {
    let dir = std::env::temp_dir().join(format!("xo-cli-diagnostics-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("temporary test directory should be created");
    let main = dir.join("main.php");
    let child = dir.join("child.php");
    fs::write(
        &main,
        "<?php\nrequire __DIR__ . \"/child.php\";\necho \"done\\n\";\n",
    )
    .expect("main program should be written");
    fs::write(&child, "<?php\nif (!ready()) {\n    echo \"no\\n\";\n}\n")
        .expect("child program should be written");

    let mut command = Command::new(env!("CARGO_BIN_EXE_xo"));
    command
        .current_dir(workspace_root())
        .arg("run")
        .arg("--diagnostics")
        .arg("json")
        .arg(&main);
    let output = command_output(command);

    let _ = fs::remove_file(&main);
    let _ = fs::remove_file(&child);
    let _ = fs::remove_dir(&dir);

    assert!(
        !output.status.success(),
        "xo run should fail for compile error"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("\x1b["));
    let json: Value = serde_json::from_str(&stderr)
        .expect("xo run --diagnostics json should emit valid JSON diagnostics");
    let reports = json["reports"]
        .as_array()
        .expect("diagnostic JSON should contain a reports array");
    assert_eq!(reports.len(), 1);

    let report = &reports[0];
    assert_eq!(report["kind"], "compile_failed");
    assert_eq!(report["phase"], "compile");
    assert!(
        report["file"]
            .as_str()
            .is_some_and(|file| file.ends_with("child.php")),
        "diagnostic file should point at child.php: {report:#?}"
    );

    let groups = report["groups"]
        .as_array()
        .expect("diagnostic report should contain groups");
    assert_eq!(groups.len(), 1);
    assert_eq!(
        groups[0]["message"],
        "unsupported expression in LLVM codegen"
    );
    assert_eq!(groups[0]["count"], 1);

    let occurrences = groups[0]["occurrences"]
        .as_array()
        .expect("diagnostic group should contain occurrences");
    assert_eq!(occurrences.len(), 1);
    assert_eq!(occurrences[0]["line"], 2);
    assert_eq!(occurrences[0]["column"], 6);
    assert_eq!(occurrences[0]["source"], "if (!ready()) {");
    assert_eq!(occurrences[0]["marker"], "     ^^^^^^^");
    assert_eq!(occurrences[0]["span"]["start"], 11);
    assert_eq!(occurrences[0]["span"]["end"], 18);

    let stack = report["stack"]
        .as_array()
        .expect("diagnostic report should contain an include stack");
    assert_eq!(stack.len(), 1);
    assert_eq!(stack[0]["kind"], "include");
    assert!(
        stack[0]["file"]
            .as_str()
            .is_some_and(|file| file.ends_with("main.php")),
        "include stack should point at main.php: {stack:#?}"
    );
    assert_eq!(stack[0]["line"], 2);
    assert_eq!(stack[0]["source"], "require __DIR__ . \"/child.php\";");
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
