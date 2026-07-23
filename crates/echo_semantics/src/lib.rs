//! File-local semantics: scopes, bind rules, receiver legality, module imports.
//!
//! Authority: `docs/syntax.md`, `docs/semantics.md`, `docs/modules.md`.

#![forbid(unsafe_code)]

mod check;
mod const_eval;
mod effect;
mod infer;
mod model;
mod types;
mod unify;

pub use const_eval::{eval_const_expr, ConstError, ConstValue};
pub use effect::{effects_in_stmts, ReturnShape};
pub use infer::infer_last_expr_type;
pub use model::{BindFact, BindId, SemanticModel, ValueKind};
pub use types::Type;

use echo_ast::File;
use echo_diagnostics::Diagnostics;
use echo_parser::{parse, Parsed};
use echo_source::{SourceFile, Span};

/// Stable crate identity for workspace linkage checks.
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

/// Binding kind for locals, imports, and module exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingKind {
    Mutable,
    Immutable,
    Const,
    /// Primary `% struct_name` type (or re-exported type).
    Struct,
    /// Module object from `/ path` (use `module.export`).
    Module,
}

/// One export available on an imported module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleExport {
    pub name: String,
    pub kind: BindingKind,
    /// If the export is a function, its `^` / `!` result shape.
    pub return_shape: Option<ReturnShape>,
}

/// Module brought into scope by `/ path` (last path segment as name).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedModule {
    pub name: String,
    pub span: Span,
    pub exports: Vec<ModuleExport>,
}

/// Result of check through parse + local semantics.
#[derive(Debug, Clone)]
pub struct Checked {
    pub parsed: Parsed,
    pub diagnostics: Diagnostics,
}

/// Lex → parse → file-local semantic check (single-file, no imports).
#[must_use]
pub fn check_source(source: &SourceFile) -> Checked {
    let parsed = parse(source);
    let mut diagnostics = Diagnostics::new();
    for d in parsed.diagnostics.items() {
        diagnostics.push(d.clone());
    }
    if let Some(file) = &parsed.file {
        for d in check_file(file).into_iter() {
            diagnostics.push(d);
        }
    }
    Checked {
        parsed,
        diagnostics,
    }
}

/// Check with no imports.
#[must_use]
pub fn check_file(file: &File) -> Diagnostics {
    check_file_with_modules(file, &[])
}

/// Check with module-scoped imports already resolved.
#[must_use]
pub fn check_file_with_modules(file: &File, modules: &[ImportedModule]) -> Diagnostics {
    let mut diagnostics = check::analyze(file, modules);
    for d in infer::infer_file(file, modules).into_iter() {
        diagnostics.push(d);
    }
    diagnostics
}

/// Codes from this crate only (`sem-*`), one per line.
#[must_use]
pub fn format_sem_diag_codes(diagnostics: &Diagnostics) -> String {
    let mut out = String::new();
    for d in diagnostics.items() {
        if let Some(code) = d.code.as_deref() {
            if code.starts_with("sem-") {
                out.push_str(code);
                out.push('\n');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use echo_source::SourceMap;

    fn codes(src: &str) -> Vec<String> {
        let mut map = SourceMap::new();
        let id = map.add("t.echo", src);
        let c = check_source(map.get(id).unwrap());
        c.diagnostics
            .items()
            .iter()
            .filter_map(|d| d.code.clone())
            .collect()
    }

    #[test]
    fn dollar_ok() {
        assert!(codes("$ x = 1\n").is_empty());
    }

    #[test]
    fn capture_outer_mutable_rejected() {
        let c = codes("~ n = 1\n$ f = () {\n  ^ n\n}\n");
        assert!(c.iter().any(|x| x == "sem-capture"), "{c:?}");
    }

    #[test]
    fn capture_outer_param_as_callee_rejected() {
        // Nested closed body calling outer param used to skip capture (SEGV at run).
        let c = codes(
            "$ compose = (f) {\n  $ h = (x) {\n    ^ f(x)\n  }\n  ^ h\n}\n",
        );
        assert!(c.iter().any(|x| x == "sem-capture"), "{c:?}");
    }

    #[test]
    fn capture_outer_param_as_value_rejected() {
        let c = codes("$ wrap = (f) {\n  $ h = () {\n    ^ f\n  }\n  ^ h\n}\n");
        assert!(c.iter().any(|x| x == "sem-capture"), "{c:?}");
    }

    #[test]
    fn outer_free_fn_bind_ok_in_nested_body() {
        // Free fn bind has return shape → allowed code-ref from nested body.
        let c = codes(
            "$ double = (n) {\n  ^ n + n\n}\n$ make = () {\n  $ h = (x) {\n    ^ double(x)\n  }\n  ^ h\n}\n",
        );
        assert!(
            !c.iter().any(|x| x == "sem-capture"),
            "unexpected capture: {c:?}"
        );
    }

    #[test]
    fn struct_lit_missing_required_field() {
        let c = codes(
            "% user {\n    $ name\n    ~ visits = 0\n}\n$ u = user { visits: 1 }\n",
        );
        assert!(
            c.iter().any(|x| x == "sem-struct-missing-field"),
            "{c:?}"
        );
    }

    #[test]
    fn struct_lit_omitted_default_ok() {
        let c = codes("% user {\n    $ name\n    ~ visits = 0\n}\n$ u = user { name: \"Ada\" }\n");
        assert!(
            !c.iter().any(|x| x.starts_with("sem-struct-")),
            "{c:?}"
        );
    }

    #[test]
    fn struct_lit_unknown_field() {
        let c = codes("% user {\n    $ name\n}\n$ u = user { name: \"A\", z: 1 }\n");
        assert!(
            c.iter().any(|x| x == "sem-struct-unknown-field"),
            "{c:?}"
        );
    }

    #[test]
    fn no_shadow_dollar() {
        let c = codes("$ x = 1\n$ x = 2\n");
        assert!(c.iter().any(|x| x == "sem-shadow"), "{c:?}");
    }

    #[test]
    fn tilde_update_ok() {
        assert!(codes("~ x = 1\n~ x = 2\n").is_empty());
    }

    #[test]
    fn tilde_cannot_update_immutable() {
        let c = codes("$ x = 1\n~ x = 2\n");
        assert!(c.iter().any(|x| x == "sem-immutable"), "{c:?}");
    }

    #[test]
    fn receiver_illegal_in_free_fn() {
        let c = codes("$ f = () {\n    ^ .n\n}\n");
        assert!(c.iter().any(|x| x == "sem-receiver"), "{c:?}");
    }

    #[test]
    fn use_before_bind_any_value() {
        let c = codes("$ a = b + 4\n$ b = 5\n");
        assert!(c.iter().any(|x| x == "sem-unbound"), "{c:?}");
    }

    #[test]
    fn use_before_bind_function_value_same_rule() {
        let c = codes("$ a = b()\n$ b = () {\n  ^ 1\n}\n");
        assert!(c.iter().any(|x| x == "sem-unbound"), "{c:?}");
    }

    #[test]
    fn bind_then_use_ok() {
        assert!(codes("$ b = 5\n$ a = b + 4\n").is_empty());
    }

    #[test]
    fn self_call_in_function_value_ok() {
        let c = codes(
            "$ fact = (n) {\n  ? n == 0 {\n    ^ 1\n  }\n  ^ n * fact(n - 1)\n}\n",
        );
        assert!(!c.iter().any(|x| x == "sem-unbound"), "{c:?}");
    }

    #[test]
    fn width_tag_cannot_follow_unary() {
        let c = codes("$ a = -<i32>32\n");
        assert!(
            c.iter().any(|x| x == "sem-width-unary"),
            "expected sem-width-unary, got {c:?}"
        );
        let c2 = codes("$ a = <i32> -32\n");
        assert!(
            !c2.iter().any(|x| x == "sem-width-unary"),
            "signed lit after tag is ok: {c2:?}"
        );
    }

    #[test]
    fn receiver_ok_in_method() {
        let src = "\
% c {
    ~ n = 0
    $ get = () {
        ^ .n
    }
}
";
        assert!(codes(src).is_empty(), "{:?}", codes(src));
    }

    #[test]
    fn hash_must_scream() {
        let c = codes("# max = 1\n");
        assert!(c.iter().any(|x| x == "sem-hash-name"), "{c:?}");
    }

    #[test]
    fn hash_const_eval_ok() {
        assert!(
            codes("# A = 1\n# B = A + 2\n").is_empty(),
            "{:?}",
            codes("# A = 1\n# B = A + 2\n")
        );
    }

    #[test]
    fn hash_const_rejects_call() {
        let c = codes("# A = f()\n");
        assert!(c.iter().any(|x| x == "sem-const"), "{c:?}");
    }

    #[test]
    fn hash_const_rejects_runtime_name() {
        let c = codes("$ x = 1\n# A = x\n");
        assert!(c.iter().any(|x| x == "sem-const"), "{c:?}");
    }

    #[test]
    fn break_outside_loop() {
        let c = codes("<\n");
        assert!(c.iter().any(|x| x == "sem-break"), "{c:?}");
    }

    #[test]
    fn continue_outside_loop() {
        let c = codes(">\n");
        assert!(c.iter().any(|x| x == "sem-continue"), "{c:?}");
    }

    #[test]
    fn error_return_ok_at_top_level() {
        // Top-level is the program body; `!` is legal there (not "outside function").
        let c = codes("! \"x\"\n");
        assert!(!c.iter().any(|x| x == "sem-error-return"), "{c:?}");
    }

    /// Imported module name occupies the outer scope (same as old import-bind shadow).
    #[test]
    fn import_module_name_shadow() {
        use echo_source::BytePos;
        let mut map = SourceMap::new();
        let id = map.add("t.echo", "$ math = 1\n");
        let p = parse(map.get(id).unwrap());
        let file = p.file.unwrap();
        let modules = [ImportedModule {
            name: "math".into(),
            span: Span::new(id, BytePos(0), BytePos(1)),
            exports: vec![],
        }];
        let d = check_file_with_modules(&file, &modules);
        assert!(
            d.items().iter().any(|x| x.code.as_deref() == Some("sem-shadow")),
            "{:?}",
            d.items()
        );
    }

    #[test]
    fn option_must_be_handled() {
        let src = "\
$ f = () {
    ^
    ^ 1
}
$ x = f()
";
        let c = codes(src);
        assert!(c.iter().any(|x| x == "sem-unhandled-option"), "{c:?}");
    }

    #[test]
    fn module_export_missing() {
        use echo_source::BytePos;
        let mut map = SourceMap::new();
        let id = map.add("t.echo", "$ x = math.missing\n");
        let p = parse(map.get(id).unwrap());
        let file = p.file.unwrap();
        let modules = [ImportedModule {
            name: "math".into(),
            span: Span::new(id, BytePos(0), BytePos(1)),
            exports: vec![ModuleExport {
                name: "add".into(),
                kind: BindingKind::Immutable,
                return_shape: None,
            }],
        }];
        let d = check_file_with_modules(&file, &modules);
        assert!(
            d.items()
                .iter()
                .any(|x| x.code.as_deref() == Some("sem-module-export")),
            "{:?}",
            d.items()
        );
    }

    #[test]
    fn module_export_ok() {
        use echo_source::BytePos;
        let mut map = SourceMap::new();
        let id = map.add("t.echo", "$ x = math.add\n");
        let p = parse(map.get(id).unwrap());
        let file = p.file.unwrap();
        let modules = [ImportedModule {
            name: "math".into(),
            span: Span::new(id, BytePos(0), BytePos(1)),
            exports: vec![ModuleExport {
                name: "add".into(),
                kind: BindingKind::Immutable,
                return_shape: None,
            }],
        }];
        let d = check_file_with_modules(&file, &modules);
        assert_eq!(d.error_count(), 0, "{:?}", d.items());
    }

    #[test]
    fn result_must_be_handled() {
        let src = "\
$ f = () {
    ! \"x\"
}
$ x = f()
";
        let c = codes(src);
        assert!(c.iter().any(|x| x == "sem-unhandled-result"), "{c:?}");
    }

    #[test]
    fn result_handled_in_effect_block() {
        let src = "\
$ f = () {
    ! \"x\"
}
& {
    $ y = f()
    ^ y
}
";
        let c = codes(src);
        assert!(
            !c.iter().any(|x| *x == "sem-unhandled-result"),
            "{c:?}"
        );
    }

    #[test]
    fn effect_block_bind_ok() {
        let src = "\
$ f = () {
    ^ 1
}
& out = {
    $ y = f()
    ^ y
}
";
        let c = codes(src);
        assert!(
            !c.iter().any(|x| *x == "sem-unhandled-result"),
            "{c:?}"
        );
    }

    #[test]
    fn result_match_ok() {
        let src = "\
$ f = () {
    ! \"x\"
    ^ 1
}
$ main = () {
    | f() {
        $ v { ^ }
        ! e { ^ }
    }
}
";
        let c = codes(src);
        assert!(
            !c.iter().any(|x| *x == "sem-unhandled-result"),
            "{c:?}"
        );
    }

    #[test]
    fn option_match_ok() {
        let src = "\
$ f = () {
    ^
    ^ 1
}
$ main = () {
    | f() {
        $ v { ^ }
        : { ^ }
    }
}
";
        let c = codes(src);
        assert!(
            !c.iter().any(|x| *x == "sem-unhandled-option"),
            "{c:?}"
        );
    }

    #[test]
    fn infer_int_add_ok() {
        let c = codes("$ x = 1 + 2\n");
        assert!(
            !c.iter().any(|x| *x == "sem-type-mismatch"),
            "{c:?}"
        );
    }

    #[test]
    fn free_param_is_value_allows_mixed_call_sites() {
        // Unconstrained param (passthrough) pins to `value` — int and string ok.
        let c = codes(
            "\
$ id = (x) {
    ^ x
}
$ main = () {
    $ a = id(1)
    $ b = id(\"hi\")
    ^ a
}
",
        );
        assert!(
            !c.iter().any(|x| *x == "sem-type-mismatch"),
            "expected mixed id() calls to type-check, got {c:?}"
        );
    }

    #[test]
    fn list_len_helper_does_not_poison_caller_element_kind() {
        // Structural list use (for-in only) must not rewrite the caller's note
        // on `xs` to list(value) — later numeric ordering must still type-check.
        let c = codes(
            "\
$ count = (xs) {
    ~ n = 0
    * item : xs {
        ~ n = n + 1
    }
    ^ n
}
$ bubble = (xs) {
    $ n = count(xs)
    ~ j = 0
    * j < n - 1 {
        ? xs[j] > xs[j + 1] {
            ^ 1
        }
        ~ j = j + 1
    }
    ^ 0
}
$ main = () {
    ^ bubble([3, 1, 2])
}
",
        );
        assert!(
            !c.iter().any(|x| *x == "sem-type-mismatch"),
            "count(xs) must not poison xs element kind for later >, got {c:?}"
        );
    }

    #[test]
    fn dynamic_value_still_rejects_ordering() {
        // Real `value` payloads still cannot use >.
        let c = codes(
            "\
$ id = (x) {
    ^ x
}
$ main = () {
    $ a = id(1)
    $ b = id(2)
    ? a > b {
        ^ 1
    }
    ^ 0
}
",
        );
        assert!(
            c.iter().any(|x| *x == "sem-type-mismatch"),
            "expected value > value to fail, got {c:?}"
        );
    }

    #[test]
    fn value_key_field_allows_mixed_puts() {
        // Collection-shaped: free key field + eq-only method params → value.
        let c = codes(
            "\
% entry {
    $ key
    ~ value
}
% tab {
    $ put = (key, value) {
        $ e = entry { key: key, value: value }
        ? e.key == key {
            ^ .
        }
        ^ .
    }
}
$ main = () {
    $ t = tab {}
    t.put(1, 10)
    t.put(\"a\", 20)
    ^
}
",
        );
        assert!(
            !c.iter().any(|x| *x == "sem-type-mismatch"),
            "expected mixed put keys, got {c:?}"
        );
    }

    #[test]
    fn constrained_param_still_monomorphic() {
        // Arithmetic pins param to int — string call must fail.
        let c = codes(
            "\
$ add1 = (x) {
    ^ x + 1
}
$ main = () {
    $ a = add1(1)
    $ b = add1(\"no\")
    ^ a
}
",
        );
        assert!(
            c.iter().any(|x| *x == "sem-type-mismatch"),
            "expected string into int param to fail, got {c:?}"
        );
    }

    #[test]
    fn infer_int_float_mix_err() {
        let c = codes("$ x = 1 + 2.0\n");
        assert!(c.iter().any(|x| x == "sem-type-mismatch"), "{c:?}");
    }

    #[test]
    fn infer_bool_cond() {
        let c = codes("? 1 {\n    ^\n}\n");
        // 1 is not bool
        assert!(c.iter().any(|x| x == "sem-type-mismatch"), "{c:?}");
    }

    #[test]
    fn infer_not_callable() {
        let c = codes("$ x = 1\n$ y = x()\n");
        assert!(c.iter().any(|x| x == "sem-not-callable"), "{c:?}");
    }

    #[test]
    fn infer_width_mix_err() {
        // Two different non-default widths still do not mix.
        let c = codes("$ x = <i32>1 + <ui64>2\n");
        assert!(c.iter().any(|x| x == "sem-type-mismatch"), "{c:?}");
    }

    #[test]
    fn infer_width_default_ok() {
        let c = codes("$ x = <i64>1 + 2\n");
        assert!(
            !c.iter().any(|x| *x == "sem-type-mismatch"),
            "{c:?}"
        );
    }

    #[test]
    fn infer_default_i64_yields_to_i32() {
        // Untagged / default i64 adopts a more specific width.
        let c = codes("$ x = <i32>1 + 2\n$ y = <i32>1 + <i64>2\n");
        assert!(
            !c.iter().any(|x| *x == "sem-type-mismatch"),
            "{c:?}"
        );
    }

    #[test]
    fn field_width_from_default_allows_ops_without_cast() {
        // `~ v = <ui64> 0` → field is ui64; load/store ops need no re-tag.
        let c = codes(
            "% s {\n    ~ v = <ui64> 0\n}\n$ t = s {}\n~ t.v = t.v + <ui64> 1\n",
        );
        assert!(
            !c.iter().any(|x| *x == "sem-type-mismatch"),
            "expected field ui64 from default, got {c:?}"
        );
    }

    #[test]
    fn field_width_from_default_rejects_explicit_other_width_write() {
        // Untagged `1` yields to ui64; explicit non-default widths still clash.
        let c = codes("% s {\n    ~ v = <ui64> 0\n}\n$ t = s {}\n~ t.v = <i32> 1\n");
        assert!(
            c.iter().any(|x| *x == "sem-type-mismatch"),
            "expected mismatch writing i32 into ui64 field, got {c:?}"
        );
    }

    #[test]
    fn default_int_literal_yields_to_ui64_lane() {
        let c = codes(
            "% s {\n    ~ v = <ui64> 0\n}\n$ t = s {}\n~ t.v = t.v + 1\n~ t.v = t.v << 3\n",
        );
        assert!(
            !c.iter().any(|x| *x == "sem-type-mismatch"),
            "untagged lits should adopt ui64 field lane, got {c:?}"
        );
    }

    #[test]
    fn infer_duration_ok() {
        let c = codes("$ t = 5s\n$ u = 10ms\n");
        assert!(
            !c.iter().any(|x| *x == "sem-type-mismatch"),
            "{c:?}"
        );
    }
}
