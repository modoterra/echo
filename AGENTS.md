# AGENTS.md

## Workspace
- Rust workspace with resolver `2`; all crates use edition `2024`.
- Crate flow is `echo_source`/`echo_diagnostics` -> `echo_lexer` -> `echo_ast` -> `echo_parser`; `xo` is the CLI entrypoint.
- `echo_codegen` is a separate LLVM backend stub using `inkwell` with feature `llvm22-1`; clean environments need LLVM 22 available for full workspace builds.

## Product Direction
- Echo is a Rust implementation of a PHP superset: existing PHP programs should remain valid while Echo adds modern runtime and language features.
- When changing syntax, parsing, diagnostics, or runtime behavior, preserve PHP compatibility unless the task explicitly says otherwise.

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

## Current Gotchas
- Parser currently parses source text directly with Chumsky after first calling `echo_lexer::lex(source)?` only to surface lexer errors.
- `xo run` shells out to `lli`; `xo build` shells out to `clang -x ir`, so full end-to-end tests need those tools on `PATH`. PHP benchmarks also need `php` on `PATH`.

## Source Notes
- `echo_source::SourceFile::new` classifies `.echo` and `.xo` as `EchoFile`; every other extension is `PhpFile`.
- `examples/hello.echo` has no PHP open tag; `examples/hello.php` includes `<?php`. The parser accepts both forms.
- AST currently models numbers, variables, and binary expressions, but the parser path only accepts `echo` statements containing comma-separated string literals.
