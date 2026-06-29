# AGENTS.md

## Workspace
- Rust workspace with resolver `2`; all crates use edition `2024`.
- Crate flow is `echo_source`/`echo_diagnostics` -> `echo_lexer` -> `echo_ast` -> `echo_parser` -> `echo_semantics` -> `echo_codegen`; `xo` is the CLI entrypoint.
- `echo_codegen` is the LLVM backend using `inkwell` with feature `llvm22-1`; clean environments need LLVM 22 available for full workspace builds.

## Product Direction
- Echo is a Rust implementation of a PHP superset: existing PHP programs should remain valid while Echo adds modern runtime and language features.
- When changing syntax, parsing, diagnostics, or runtime behavior, preserve PHP compatibility unless the task explicitly says otherwise.
- Output buffering semantics are tracked in `docs/output-buffering.md`; consult it before changing `echo_runtime` or `ob_*` codegen.
- Echo source files should use `snake_case.echo` file names and directories should use lowercase names with underscores when needed. Keep PHP/Composer package directory names as required by their ecosystem, but do not carry PHP class-file naming into Echo source. Use `PascalCase` for class/type/enum/trait/interface names and `snake_case` for functions, variables, modules, and file/module identifiers.
- Echo package code should declare and import Echo modules with lower-dot names such as `module modoterra.laravel_echo.console` and `from illuminate.console use Command`. Echo module declarations and PHP namespace declarations that denote the same package boundary should resolve to the same internal identity and lower through the same compiler path; package segments use lowercase/snake_case in Echo and PascalCase namespace segments in PHP.
- Canonical Echo imports have two forms: `use illuminate.console.Command` for a single direct import, and `from illuminate.console use Command, AndThis, AndThat` for multiple imports from the same module. `from illuminate.console use Command` remains legal for one item, but prefer direct `use ...` when importing exactly one symbol.
- Echo imports may alias imported symbols with `as`: `use illuminate.console.Command as LaravelCommand` and `from illuminate.console use Command, AndThis as ThisThing, AndThat`. Use aliases for name conflicts or clearer local names. Do not use module aliases such as `use illuminate.console as console` yet; reserve that for a later module-import design.
- Whole-module imports use `use some.long.module` and bind the module under its final segment, so `use std.process` enables `process.run(...)` and `use modoterra.laravel_echo.console` enables `console.EchoStartCommand`. Whole-module imports may be aliased with `as`, for example `use std.process as proc` enables `proc.run(...)`. The same module aliasing rule applies to std imports through the grouped form, for example `from std use process as proc`.
- Echo-native files may optionally start with one module declaration such as `module modoterra.laravel_echo.console`. If present, `module` must be the first declaration before imports and other statements; `use std.time` followed by `module app.console` is invalid. Only one module declaration is allowed per file. If omitted, the file belongs to the package root or to an anonymous script module. An Echo package file can be imported only when it declares a module; PHP-compat files can be imported through their PHP namespace instead.
- Echo module names use dot paths such as `module modoterra.laravel_echo.console`, `module std.time`, and `module app.http.router`. Each path part must be a simple identifier containing letters, numbers, and underscores, and may not start with a number. Valid: `module app.http_v2.router`. Invalid: `module app.http-router`, `module app.2http.router`, and `module app..router`. Module paths are case-sensitive; lowercase is the package lint convention, not a parser error, so `module App.Http.Router` may parse but should warn.
- The canonical `std` root is reserved for Echo's compiler-owned standard library after module/namespace canonicalization. Packaged stdlib source declares `module std.net`-style modules; user/package code must not declare `module std.*`, `namespace std\*`, or `namespace Std\*`.

## Documentation Quality
- Documentation for code behavior, built-ins, APIs, CLI commands, or terminal workflows must include at least one useful snippet when a snippet can make the behavior concrete.
- Snippets should be bounded in a realistic use case and show why the feature is useful, not only prove that the function or command exists. Avoid toy probes such as assigning a function name to a variable and printing `function_exists`; prefer examples that validate input, transform real data, handle an edge case, or fit into a small workflow.
- In website content source files, each user-facing code snippet should be followed by short commentary explaining the purpose of the example and how the code can be applied. Keep factual API behavior above the snippet; use the commentary after the snippet for applied guidance and tradeoffs. This commentary rule does not apply to every code block in every Markdown document.
- Website documentation for Echo standard library packages belongs under Language -> Standard Library. Add or update that page when introducing or changing public `std.*` packages, and document each package with the same useful-snippet-plus-commentary standard used for PHP built-ins.
- Echo code snippets should use current Echo style: rely on inference with `let`, avoid invalid typed variable declarations, and omit semicolons unless the documented mode specifically requires PHP syntax.

## Module Ownership Invariants
- Global domain vocabulary and module ownership are defined in `CONTEXT.md`; read it before changing compiler, runtime, or REPL behavior.
- REPL examples are language-development inputs. Do not solve them with REPL-only lookup tables, evaluators, type environments, or ad hoc value semantics; implement behavior in the shared language pipeline first.
- Semantic facts such as variable bindings, expression types, scope rules, and undefined-variable diagnostics belong in `echo_semantics`; `xo`, `echo_codegen`, future LLVM JIT code, and future LSP code should consume that shared analysis rather than reimplementing it.
- Project-wide import/name resolution policy belongs in the planned `echo_resolver` crate. `echo_index` stores extracted facts; `echo_resolver` should own module/namespace canonicalization, Composer/Echo package lookup, reserved `std` enforcement, ambiguity diagnostics, and resolved symbol artifacts. Existing local lookup in `xo`, `echo_lsp`, or codegen is transitional and should move to `echo_resolver` when touched.
- Diagnostics belong in `echo_diagnostics` as a shared compiler contract. Presentation layers may format diagnostics, but new diagnostic categories, stable codes, severities, related spans, or owning-layer metadata should be added to the shared diagnostic model rather than invented locally in `xo`, LSP, parser, resolver, semantics, HIR, MIR, or codegen.
- AST shape must truthfully represent source semantics. Do not add PHP-compatible parse support by smuggling syntax into an unrelated existing AST shape, such as representing `elseif`/`else` as statements appended to an `if` true-body. Add the proper AST structure and lower it through semantics/MIR/codegen instead.
- Keep AST node spans local. `Program` carries optional source identity for the parsed source; do not add `SourceId` to every AST expression or statement unless the source model itself changes to support true multi-source AST nodes.
- Keep collection kinds distinct: `[]` is PHP array syntax, `{}` is an Echo list, `{ field: value }` is an Echo structural object, `()` is reserved for tuples, and fixed-size arrays are their own array form. PHP `$value[] = item` append syntax may grow non-fixed arrays only; do not use it for Echo lists or fixed-size arrays.
- Runtime and executable semantics should be owned by Rust code in this workspace. Do not add C/C++ runtime implementations, `libm`/`-lm`, libc math calls, or new non-Rust link dependencies for language behavior. The current `clang` native-link driver is a bootstrap path, not a license to add C runtime semantics; replacing it with a Rust-owned link path is preferred when touching build plumbing.

## Agent skills

### Issue tracker

Issues live as GitHub issues via the `gh` CLI; external PRs are also a triage surface. See `docs/agents/issue-tracker.md`.

### Triage labels

Five canonical roles use their default label strings (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context layout: one `CONTEXT.md` at the root plus `docs/adr/` for ADRs. See `docs/agents/domain.md`.

## Commands
- Fast agent checks: `scripts/check-fast changed --list`, `scripts/check-fast changed`, `scripts/check-fast source`, `scripts/check-fast diagnostics`, `scripts/check-fast lexer`, `scripts/check-fast ast`, `scripts/check-fast runtime`, `scripts/check-fast runtime-collections`, `scripts/check-fast runtime-execution`, `scripts/check-fast runtime-output`, `scripts/check-fast runtime-reflection`, `scripts/check-fast runtime-math`, `scripts/check-fast runtime-encoding`, `scripts/check-fast parser <fixture-filter>`, `scripts/check-fast parser-concurrency`, `scripts/check-fast parser-echo-surface`, `scripts/check-fast parser-strings`, `scripts/check-fast semantics`, `scripts/check-fast hir`, `scripts/check-fast mir`, `scripts/check-fast pipeline`, `scripts/check-fast index`, `scripts/check-fast lsp`, `scripts/check-fast std`, `scripts/check-fast reflection`, `scripts/check-fast codegen`, `scripts/check-fast xo`, `scripts/check-fast repl`, `scripts/check-fast jit <fixture-filter>`, `scripts/check-fast fixture <fixture-filter>`, `scripts/check-fast bench-echo <fixture-filter>`, `scripts/check-fast bench-php <fixture-filter>`, `scripts/check-fast fmt`, `scripts/check-fast script`, `scripts/check-fast workspace`, and `scripts/check-fast web`.
- Check all crates: `cargo check --workspace`.
- Run all tests/doc-tests: `cargo test --workspace`.
- Check formatting: `cargo fmt-check` or `scripts/fmt --check`.
- Focus formatting specific Rust files with `scripts/fmt <file>...`; direct `rustfmt` defaults can ignore the workspace edition unless `--edition 2024` is passed.
- Quiet final verification: `scripts/check-fast workspace` runs `scripts/fmt --check`, `cargo check --workspace`, and `cargo test --workspace` with successful output suppressed.
- Focus one crate: `cargo test -p echo_parser` or `cargo check -p xo`.
- Benchmark PHP fixtures against system PHP: `cargo test -p xo --test php_bench -- --ignored --nocapture`; use `ECHO_BENCH_ITERATIONS=2 cargo test -p xo --test php_bench -- --ignored --nocapture` for intermediate smoke checks, and larger counts such as 100 for final benchmark reports.
- Run CLI examples: `cargo run -p xo -- ast examples/hello.php`, `cargo run -p xo -- ir examples/hello.php`, `cargo run -p xo -- run examples/hello.php`, and `cargo run -p xo -- build examples/hello.php -o /tmp/hello`.
- Benchmark PHP and Echo fixtures through Echo: `cargo test -p xo --test echo_bench -- --ignored --nocapture`; use `ECHO_BENCH_ITERATIONS=2 cargo test -p xo --test echo_bench -- --ignored --nocapture` for intermediate smoke checks.

## Fast Development Checks
- Prefer `scripts/check-fast` while iterating. It suppresses successful Rust test output and uses fixture filters so agents get targeted pass/fail signals without dumping the full fixture corpus; failing commands replay a bounded captured-output preview and retain the full temp log path.
- `scripts/check-fast` also suppresses successful command trace lines by default; set `CHECK_FAST_ECHO_COMMANDS=1` only when you need to see each underlying command.
- Set `CHECK_FAST_MAX_OUTPUT_LINES=0` to replay full failing command output, or set it to a positive line count to adjust the preview size.
- Use `scripts/check-fast changed --list` to see the checks implied by the current dirty worktree, then `scripts/check-fast changed` to run that set when the list looks right.
- Use `scripts/check-fast changed --take N` to run only the first N derived checks when the dirty worktree is broad and you want a bounded first signal.
- `scripts/check-fast changed` batches dirty fixture directories into comma-separated filters, avoids redundant filtered parser/JIT checks when broad parser or JIT checks are already selected, and uses runtime fixture-only variants when broad runtime tests are already selected.
- Use `scripts/check-fast script` after changing `scripts/check-fast` itself.
- Use `scripts/check-fast workspace` for quiet final verification before committing.
- Use `scripts/check-fast source`, `scripts/check-fast diagnostics`, `scripts/check-fast lexer`, or `scripts/check-fast ast` for foundational crate changes before checking parser behavior.
- Use `scripts/check-fast runtime` after runtime-only built-in or value-behavior changes.
- Use `scripts/check-fast runtime-collections` after PHP array, Echo list, collection key coercion, or index lookup changes.
- Use `scripts/check-fast runtime-execution` after stdout capture, REPL inspection, or runtime execution-state changes.
- Use `scripts/check-fast runtime-output` after output buffering or `ob_*` runtime changes.
- Use `scripts/check-fast runtime-reflection` after runtime reflection registry, `function_exists`, `is_callable`, or `std.reflect.*` changes.
- Use `scripts/check-fast runtime-math` after runtime numeric, float, trigonometric, rounding, exponent, or logarithm changes.
- Use `scripts/check-fast runtime-encoding` after runtime hex, base64, URL encoding, or string rewrite changes.
- Use `scripts/check-fast runtime-tests-string-search-position` or `scripts/check-fast runtime-tests-string-search-matches` for focused runtime string search tests.
- Use `scripts/check-fast parser` after parser/AST/HIR/MIR shape changes; pass a fixture filter when a parser failure is local to one fixture. Use `scripts/check-fast parser-strings` for PHP string literal parsing, `scripts/check-fast parser-concurrency` for `run`/`fork`/`spawn`/`join` parser syntax, and `scripts/check-fast parser-echo-surface` for Echo-only parser surface tests such as typed `let`, `fn`, dot receiver calls, and target server syntax.
- Use `scripts/check-fast semantics`, `scripts/check-fast hir`, or `scripts/check-fast mir` for focused checks in the shared language pipeline.
- Use `scripts/check-fast pipeline` before codegen when a change crosses parser, semantic analysis, HIR, or MIR lowering.
- Use `scripts/check-fast index`, `scripts/check-fast lsp`, `scripts/check-fast std`, or `scripts/check-fast reflection` for support-layer changes. Keep `lsp` opt-in rather than part of `smoke` because it is slower than the core pipeline checks.
- Use `scripts/check-fast codegen` after LLVM lowering or codegen ABI metadata changes; use `scripts/check-fast jit <fixture-filter>` after in-process JIT engine changes.
- Use `scripts/check-fast xo` after CLI, build plumbing, REPL presentation, or source-loading changes.
- Use `scripts/check-fast repl` after REPL session, prompt, expression display, or piped input changes.
- Use `scripts/check-fast jit <fixture-filter>` for executable behavior that can be proven through the in-process LLVM JIT.
- Use `scripts/check-fast fixture <fixture-filter>` only when the full `xo ast`/`ir`/`run`/`build` path matters for a specific fixture.
- Use `scripts/check-fast bench-echo <fixture-filter>` or `scripts/check-fast bench-php <fixture-filter>` for a two-iteration benchmark smoke check before running larger reports.
- Fast benchmark shortcuts are quiet on success; inspect `test-results/echo/` or `test-results/php/` for the benchmark reports.
- Use `scripts/check-fast web` for a quiet TypeScript plus Vite production build; successful runs suppress Vite's asset table.

```sh
scripts/check-fast changed --list
scripts/check-fast changed --take 3
scripts/check-fast runtime
scripts/check-fast runtime-collections
scripts/check-fast runtime-execution
scripts/check-fast runtime-output
scripts/check-fast runtime-reflection
scripts/check-fast runtime-math
scripts/check-fast runtime-encoding
scripts/check-fast parser
scripts/check-fast parser 017_object_append_count
scripts/check-fast pipeline
scripts/check-fast codegen
scripts/check-fast xo
scripts/check-fast repl
scripts/check-fast jit 017_object_append_count
scripts/check-fast fixture string_escape
scripts/check-fast bench-echo 017_object_append_count
```

The optional fixture filter sets `ECHO_FIXTURE` for that command. Filters are comma-separated substrings matched against fixture directory names, which keeps failure output local to the changed behavior.
Benchmark shortcuts default `ECHO_BENCH_ITERATIONS` to `2`; set the variable explicitly for larger runs.

## Slice Workflow
- Keep git hygiene visible while working: check `git status --short` before broad edits, keep ADR changes synchronized with `CONTEXT.md`, `AGENTS.md`, and `www/` when the decision is user-facing, and do not mix unrelated cleanup into a slice. When website docs change direction, prefer clean current URLs and labels over legacy aliases. If the worktree already has unrelated user changes, leave them alone and call out any verification limits they create.
- Commit completed work in logical slices with conventional commit subjects such as `docs: record single language mode` or `feat(parser): remove source modes`. Commit bodies should be rich enough for later review: explain the architectural decision or behavior change, list the important implementation/docs surfaces touched, and mention the focused verification that passed. Avoid vague subjects such as `update docs`, mixed grab-bag commits, and commits that combine unrelated refactors with user-visible behavior.
- Use a separate git worktree for each implementation slice. Keep the main checkout clean and use a descriptive branch/worktree name for the slice.
- Make slices vertical and observable: syntax/parser, AST, codegen/runtime, fixtures, docs, tests, and commit should land together when the behavior needs the full path.
- Do not stop at a parser-only or runtime-only partial unless the user explicitly asks for that boundary; prefer a coherent end-to-end slice that proves the behavior through `xo ast`, `xo ir`, `xo run`, and `xo build`.
- Start with the fixture whenever behavior is user-observable. PHP compatibility behavior belongs under `tests/php/<number>_<name>/`; Echo-only behavior belongs under `tests/echo/<number>_<name>/` with `program.echo`, `stdin.txt`, and `stdout.txt`.
- Ground PHP compatibility slices in `php.net` before implementation. Prefer adding the relevant manual URL to nearby docs/spec notes; add a concise source comment only when the PHP behavior is non-obvious in code.
- Keep implementation comments rare and useful. Comments should explain compatibility rules, runtime invariants, or surprising behavior, not restate straightforward code.
- Run focused tests while developing, then run quiet full verification before committing: `scripts/check-fast workspace`; run the relevant fixture harness separately when a slice needs a specific fixture path.
- Run ignored benchmarks when a slice affects executable behavior, runtime performance, fixture benchmark reports, or PHP/Echo parity timing: `ECHO_BENCH_ITERATIONS=2 cargo test -p xo --test php_bench -- --ignored --nocapture` for PHP parity and `ECHO_BENCH_ITERATIONS=2 cargo test -p xo --test echo_bench -- --ignored --nocapture` for Echo runtime behavior.
- Commit each completed slice with a small meaningful conventional commit after verification. Do not push unless the user explicitly asks.

## PHP Compatibility Fixtures
- Add compatibility cases under `tests/php/<number>_<name>/` with `program.php`, `stdin.txt`, and `stdout.txt`.
- `crates/echo_parser/tests/php_fixtures.rs` checks every fixture is well-formed and parseable.
- `crates/xo/tests/php_fixtures.rs` exercises `ast`, `ir`, `run`, and `build`; `run` and the built binary must match `stdout.txt` with `stdin.txt` piped in.
- The `xo` fixture test overwrites latest stable artifacts in `test-results/php/<fixture>/`: `ast.txt`, `ir.ll`, `run.stdout`, `run.stderr`, `binary.stdout`, and `binary.stderr`.
- Built executables use temporary per-process paths under `test-results/php/.runs/<pid>/<fixture>/` to avoid concurrent test runs touching the same binary; fixture and benchmark harnesses should clean `.runs` automatically, while stable stdout/stderr/IR artifacts still live under `test-results/php/<fixture>/`.
- The ignored `crates/xo/tests/php_bench.rs` benchmark requires `php` on `PATH`, builds each fixture, compares PHP/Echo stdout, prints timing, and writes `benchmark.txt` under the same artifact directory.
- The ignored `crates/xo/tests/echo_bench.rs` benchmark requires `clang` on `PATH`, covers both `tests/php` and `tests/echo`, skips fixtures with `unsupported.txt`, validates built Echo binary stdout, prints binary timing, and writes reports under `test-results/echo/`.

## Parity Loop
- Work in focused PHP slices: one fixture should introduce one new language/runtime behavior, but the slice should still be large enough to prove the behavior end-to-end.
- Before implementing, ground the intended behavior in the PHP manual on `php.net`; prefer a direct manual quote or URL over assumptions.
- Generate fixture `stdout.txt` from system PHP whenever possible: `php tests/php/<fixture>/program.php < tests/php/<fixture>/stdin.txt > tests/php/<fixture>/stdout.txt`. This preserves exact bytes, including intentionally absent trailing newlines.
- Before implementing, run the fixture with system `php` when behavior is not obvious; make `stdout.txt` match PHP, not assumptions or hand-written formatting.
- TDD order: add `program.php`/`stdin.txt` -> generate `stdout.txt` with PHP -> confirm `cargo test -p xo --test php_fixtures` fails -> implement the smallest parser/AST/codegen/runtime change -> make the fixture pass.
- Each slice must keep the whole path working: `xo ast`, `xo ir`, `xo run`, and `xo build`, with run and built binary stdout matching PHP.
- After green, run `scripts/check-fast workspace`; run the ignored PHP/Echo benchmark that matches the affected behavior when the slice affects executable behavior, runtime behavior, performance, or benchmark reports.
- Before committing, re-check the implemented behavior against the relevant `php.net` manual page and update nearby docs/spec notes with the source URL; include a concise source comment in code only when it clarifies non-obvious PHP compatibility behavior.
- After verifying a completed slice, create a small meaningful conventional commit with an explanatory body; do not push unless the user explicitly asks.
- Keep unsupported PHP behavior explicit with diagnostics rather than silently approximating semantics.

## Current Gotchas
- Parser entrypoints accept registered `echo_source::SourceFile` values through `echo_parser::parse_source_file`; low-level raw string parsing remains for parser tests and compatibility scaffolding. Parsing still uses Chumsky after first calling `echo_lexer::lex(source)?` to surface lexer errors.
- `xo run` and `xo build` share the same binary build path: generated LLVM IR is linked with `target/debug/libecho_runtime.a` via `clang -x ir`. This is transitional build plumbing; avoid adding any C/C++ runtime libraries or language semantics through that path. Full end-to-end tests need `clang` on `PATH`; PHP benchmarks also need `php` on `PATH`.

## Source Notes
- `echo_source::SourceFile::new` stores source text and path metadata; file extension does not select a parser or semantic mode.
- `echo_source` owns source identity. Use `SourceMap`/`SourceId` for registered files, REPL snippets, std modules, and anonymous test sources instead of adding layer-local path/text registries. `Span` remains a byte-offset range within one source; migrate cross-file APIs toward `SourceSpan` as that type lands.
- `examples/hello.echo` has no PHP open tag; `examples/hello.php` includes `<?php`. The parser accepts both forms.
- Parser currently accepts `echo` statements, no-argument function-call statements, string/number literals, and `.` concat expressions for the supported fixture subset.
