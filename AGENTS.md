# AGENTS.md

## Workspace
- Rust workspace with resolver `2`; all crates use edition `2024`.
- Crate flow is `echo_source`/`echo_diagnostics` -> `echo_lexer` -> `echo_ast` -> `echo_parser`; `xo` is the CLI entrypoint.
- `echo_codegen` is a separate LLVM backend stub using `inkwell` with feature `llvm22-1`; clean environments need LLVM 22 available for full workspace builds.

## Product Direction
- Echo is a Rust implementation of a PHP superset: existing PHP programs should remain valid while Echo adds modern runtime and language features.
- When changing syntax, parsing, diagnostics, or runtime behavior, preserve PHP compatibility unless the task explicitly says otherwise.
- Output buffering semantics are tracked in `docs/runtime/output-buffering.md`; consult it before changing `echo_runtime` or `ob_*` codegen.

## Commands
- Check all crates: `cargo check --workspace`.
- Run all tests/doc-tests: `cargo test --workspace`.
- Check formatting: `cargo fmt --all -- --check`.
- Focus one crate: `cargo test -p echo_parser` or `cargo check -p xo`.
- Benchmark PHP fixtures against system PHP: `cargo test -p xo --test php_bench -- --ignored --nocapture`.
- Run CLI examples: `cargo run -p xo -- ast examples/hello.php`, `cargo run -p xo -- ir examples/hello.php`, `cargo run -p xo -- run examples/hello.php`, and `cargo run -p xo -- build examples/hello.php -o /tmp/hello`.

## PHP Compatibility Fixtures
- Add compatibility cases under `tests/php/<number>_<name>/` with `program.php`, `stdin.txt`, and `stdout.txt`.
- `crates/echo_parser/tests/php_fixtures.rs` checks every fixture is well-formed and parseable.
- `crates/xo/tests/php_fixtures.rs` exercises `ast`, `ir`, `run`, and `build`; `run` and the built binary must match `stdout.txt` with `stdin.txt` piped in.
- The `xo` fixture test overwrites latest artifacts in `test-results/php/<fixture>/`: `ast.txt`, `ir.ll`, `run.stdout`, `run.stderr`, `program`, `binary.stdout`, and `binary.stderr`.
- The ignored `crates/xo/tests/php_bench.rs` benchmark requires `php` on `PATH`, builds each fixture, compares PHP/Echo stdout, prints timing, and writes `benchmark.txt` under the same artifact directory.

## Parity Loop
- Work in extremely thin PHP slices: one fixture should introduce one new language/runtime behavior.
- Before implementing, run the fixture with system `php` when behavior is not obvious; make `stdout.txt` match PHP, not assumptions.
- TDD order: add fixture -> confirm `cargo test -p xo --test php_fixtures` fails -> implement the smallest parser/AST/codegen/runtime change -> make the fixture pass.
- Each slice must keep the whole path working: `xo ast`, `xo ir`, `xo run`, and `xo build`, with run and built binary stdout matching PHP.
- After green, run `cargo test --workspace` and `cargo fmt --all -- --check`; run the ignored PHP benchmark when the slice affects executable behavior or benchmark reports.
- After verifying a completed slice, create a small meaningful conventional commit with an explanatory body; do not push unless the user explicitly asks.
- Keep unsupported PHP behavior explicit with diagnostics rather than silently approximating semantics.

## Current Gotchas
- Parser currently parses source text directly with Chumsky after first calling `echo_lexer::lex(source)?` only to surface lexer errors.
- `xo run` and `xo build` share the same binary build path: generated LLVM IR is linked with `target/debug/libecho_runtime.a` via `clang -x ir`. Full end-to-end tests need `clang` on `PATH`; PHP benchmarks also need `php` on `PATH`.

## Source Notes
- `echo_source::SourceFile::new` classifies `.echo` and `.xo` as `EchoFile`; every other extension is `PhpFile`.
- `examples/hello.echo` has no PHP open tag; `examples/hello.php` includes `<?php`. The parser accepts both forms.
- Parser currently accepts `echo` statements, no-argument function-call statements, string/number literals, and `.` concat expressions for the supported fixture subset.
