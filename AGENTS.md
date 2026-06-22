# AGENTS.md

## Workspace
- Rust workspace with resolver `2`; all crates use edition `2024`.
- Crate flow is `echo_source`/`echo_diagnostics` -> `echo_lexer` -> `echo_ast` -> `echo_parser` -> `echo_semantics` -> `echo_codegen`; `xo` is the CLI entrypoint.
- `echo_codegen` is a separate LLVM backend stub using `inkwell` with feature `llvm22-1`; clean environments need LLVM 22 available for full workspace builds.

## Product Direction
- Echo is a Rust implementation of a PHP superset: existing PHP programs should remain valid while Echo adds modern runtime and language features.
- When changing syntax, parsing, diagnostics, or runtime behavior, preserve PHP compatibility unless the task explicitly says otherwise.
- Output buffering semantics are tracked in `docs/output-buffering.md`; consult it before changing `echo_runtime` or `ob_*` codegen.

## Documentation Quality
- Documentation for code behavior, built-ins, APIs, CLI commands, or terminal workflows must include at least one useful snippet when a snippet can make the behavior concrete.
- Snippets should be bounded in a realistic use case and show why the feature is useful, not only prove that the function or command exists. Avoid toy probes such as assigning a function name to a variable and printing `function_exists`; prefer examples that validate input, transform real data, handle an edge case, or fit into a small workflow.
- In website content source files, each user-facing code snippet should be followed by short commentary explaining the purpose of the example and how the code can be applied. Keep factual API behavior above the snippet; use the commentary after the snippet for applied guidance and tradeoffs. This commentary rule does not apply to every code block in every Markdown document.
- Website documentation for Echo standard library packages belongs under Language -> Standard Library. Add or update that page when introducing or changing public `std.*` packages, and document each package with the same useful-snippet-plus-commentary standard used for PHP built-ins.
- Echo code snippets should use current Echo style: rely on inference with `let`, avoid invalid typed variable declarations, and omit semicolons unless the documented mode specifically requires PHP syntax.

## Module Ownership Invariants
- Global domain vocabulary and module ownership are defined in `CONTEXT.md`; read it before changing compiler, runtime, or REPL behavior.
- REPL examples are language-development inputs. Do not solve them with REPL-only lookup tables, evaluators, type environments, or ad hoc value semantics; implement behavior in the shared language pipeline first.
- Semantic facts such as variable bindings, expression types, scope rules, and undefined-variable diagnostics belong in `echo_semantics`; `xo`, `echo_codegen`, future VM code, and future LSP code should consume that shared analysis rather than reimplementing it.
- Keep collection kinds distinct: `[]` is PHP array syntax, `{}` is an Echo list, `{ field: value }` is an Echo structural object, `()` is reserved for tuples, and fixed-size arrays are their own array form. PHP `$value[] = item` append syntax may grow non-fixed arrays only; do not use it for Echo lists or fixed-size arrays.
- Runtime and executable semantics should be owned by Rust code in this workspace. Do not add C/C++ runtime implementations, `libm`/`-lm`, libc math calls, or new non-Rust link dependencies for language behavior. The current `clang` native-link driver is a bootstrap path, not a license to add C runtime semantics; replacing it with a Rust-owned link path is preferred when touching build plumbing.

## Agent skills

### Domain docs

This repo uses a single-context domain documentation layout. See `docs/agents/domain.md`.

## Commands
- Fast agent checks: `scripts/check-fast changed --list`, `scripts/check-fast changed`, `scripts/check-fast source`, `scripts/check-fast diagnostics`, `scripts/check-fast lexer`, `scripts/check-fast ast`, `scripts/check-fast runtime`, `scripts/check-fast runtime-collections`, `scripts/check-fast runtime-execution`, `scripts/check-fast runtime-output`, `scripts/check-fast runtime-reflection`, `scripts/check-fast runtime-math`, `scripts/check-fast runtime-encoding`, `scripts/check-fast parser <fixture-filter>`, `scripts/check-fast semantics`, `scripts/check-fast hir`, `scripts/check-fast mir`, `scripts/check-fast pipeline`, `scripts/check-fast index`, `scripts/check-fast lsp`, `scripts/check-fast std`, `scripts/check-fast reflection`, `scripts/check-fast codegen`, `scripts/check-fast xo`, `scripts/check-fast repl`, `scripts/check-fast jit <fixture-filter>`, `scripts/check-fast fixture <fixture-filter>`, `scripts/check-fast bench-echo <fixture-filter>`, `scripts/check-fast bench-php <fixture-filter>`, `scripts/check-fast fmt`, `scripts/check-fast script`, and `scripts/check-fast web`.
- Check all crates: `cargo check --workspace`.
- Run all tests/doc-tests: `cargo test --workspace`.
- Check formatting: `cargo fmt --all -- --check`.
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
- Use `scripts/check-fast source`, `scripts/check-fast diagnostics`, `scripts/check-fast lexer`, or `scripts/check-fast ast` for foundational crate changes before checking parser behavior.
- Use `scripts/check-fast runtime` after runtime-only built-in or value-behavior changes.
- Use `scripts/check-fast runtime-collections` after PHP array, Echo list, collection key coercion, or index lookup changes.
- Use `scripts/check-fast runtime-execution` after stdout capture, REPL inspection, or runtime execution-state changes.
- Use `scripts/check-fast runtime-output` after output buffering or `ob_*` runtime changes.
- Use `scripts/check-fast runtime-reflection` after runtime reflection registry, `function_exists`, `is_callable`, or `std.reflect.*` changes.
- Use `scripts/check-fast runtime-math` after runtime numeric, float, trigonometric, rounding, exponent, or logarithm changes.
- Use `scripts/check-fast runtime-encoding` after runtime hex, base64, URL encoding, or string rewrite changes.
- Use `scripts/check-fast parser` after parser/AST/HIR/MIR shape changes; pass a fixture filter when a parser failure is local to one fixture.
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
- Use a separate git worktree for each implementation slice. Keep the main checkout clean and use a descriptive branch/worktree name for the slice.
- Make slices vertical and observable: syntax/parser, AST, codegen/runtime, fixtures, docs, tests, and commit should land together when the behavior needs the full path.
- Do not stop at a parser-only or runtime-only partial unless the user explicitly asks for that boundary; prefer a coherent end-to-end slice that proves the behavior through `xo ast`, `xo ir`, `xo run`, and `xo build`.
- Start with the fixture whenever behavior is user-observable. PHP compatibility behavior belongs under `tests/php/<number>_<name>/`; Echo-only behavior belongs under `tests/echo/<number>_<name>/` with `program.echo`, `stdin.txt`, and `stdout.txt`.
- Ground PHP compatibility slices in `php.net` before implementation. Prefer adding the relevant manual URL to nearby docs/spec notes; add a concise source comment only when the PHP behavior is non-obvious in code.
- Keep implementation comments rare and useful. Comments should explain compatibility rules, runtime invariants, or surprising behavior, not restate straightforward code.
- Run focused tests while developing, then run the full verification set before committing: `cargo fmt --all -- --check`, `cargo test --workspace`, and the relevant fixture harness.
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
- After green, run `cargo test --workspace` and `cargo fmt --all -- --check`; run the ignored PHP/Echo benchmark that matches the affected behavior when the slice affects executable behavior, runtime behavior, performance, or benchmark reports.
- Before committing, re-check the implemented behavior against the relevant `php.net` manual page and update nearby docs/spec notes with the source URL; include a concise source comment in code only when it clarifies non-obvious PHP compatibility behavior.
- After verifying a completed slice, create a small meaningful conventional commit with an explanatory body; do not push unless the user explicitly asks.
- Keep unsupported PHP behavior explicit with diagnostics rather than silently approximating semantics.

## Current Gotchas
- Parser currently parses source text directly with Chumsky after first calling `echo_lexer::lex(source)?` only to surface lexer errors.
- `xo run` and `xo build` share the same binary build path: generated LLVM IR is linked with `target/debug/libecho_runtime.a` via `clang -x ir`. This is transitional build plumbing; avoid adding any C/C++ runtime libraries or language semantics through that path. Full end-to-end tests need `clang` on `PATH`; PHP benchmarks also need `php` on `PATH`.

## Source Notes
- `echo_source::SourceFile::new` classifies `.echo` and `.xo` as strict mode; every other extension defaults to Echo superset mode.
- `examples/hello.echo` has no PHP open tag; `examples/hello.php` includes `<?php`. The parser accepts both forms.
- Parser currently accepts `echo` statements, no-argument function-call statements, string/number literals, and `.` concat expressions for the supported fixture subset.
