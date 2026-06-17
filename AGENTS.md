# AGENTS.md

## Workspace
- Rust workspace with resolver `2`; all crates use edition `2024`.
- Crate flow is `echo_source`/`echo_diagnostics` -> `echo_lexer` -> `echo_ast` -> `echo_parser`; `xo` is the CLI entrypoint.
- `echo_codegen` is a separate LLVM backend stub using `inkwell` with feature `llvm22-1`; clean environments need LLVM 22 available for full workspace builds.

## Product Direction
- Echo is a Rust implementation of a PHP superset: existing PHP programs should remain valid while Echo adds modern runtime and language features.
- When changing syntax, parsing, diagnostics, or runtime behavior, preserve PHP compatibility unless the task explicitly says otherwise.
- Output buffering semantics are tracked in `docs/output-buffering.md`; consult it before changing `echo_runtime` or `ob_*` codegen.

## Agent skills

### Domain docs

This repo uses a single-context domain documentation layout. See `docs/agents/domain.md`.

## Commands
- Check all crates: `cargo check --workspace`.
- Run all tests/doc-tests: `cargo test --workspace`.
- Check formatting: `cargo fmt --all -- --check`.
- Focus one crate: `cargo test -p echo_parser` or `cargo check -p xo`.
- Benchmark PHP fixtures against system PHP: `cargo test -p xo --test php_bench -- --ignored --nocapture`; use `ECHO_BENCH_ITERATIONS=2 cargo test -p xo --test php_bench -- --ignored --nocapture` for intermediate smoke checks, and larger counts such as 100 for final benchmark reports.
- Run CLI examples: `cargo run -p xo -- ast examples/hello.php`, `cargo run -p xo -- ir examples/hello.php`, `cargo run -p xo -- run examples/hello.php`, and `cargo run -p xo -- build examples/hello.php -o /tmp/hello`.
- Benchmark PHP and Echo fixtures through Echo: `cargo test -p xo --test echo_bench -- --ignored --nocapture`; use `ECHO_BENCH_ITERATIONS=2 cargo test -p xo --test echo_bench -- --ignored --nocapture` for intermediate smoke checks.

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
- Built executables use per-process paths under `test-results/php/.runs/<pid>/<fixture>/` to avoid concurrent test runs touching the same binary; stable stdout/stderr/IR artifacts still live under `test-results/php/<fixture>/`.
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
- `xo run` and `xo build` share the same binary build path: generated LLVM IR is linked with `target/debug/libecho_runtime.a` via `clang -x ir`. Full end-to-end tests need `clang` on `PATH`; PHP benchmarks also need `php` on `PATH`.

## Source Notes
- `echo_source::SourceFile::new` classifies `.echo` and `.xo` as strict mode; every other extension defaults to Echo superset mode.
- `examples/hello.echo` has no PHP open tag; `examples/hello.php` includes `<?php`. The parser accepts both forms.
- Parser currently accepts `echo` statements, no-argument function-call statements, string/number literals, and `.` concat expressions for the supported fixture subset.
