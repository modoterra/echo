# Echo Parser Fixtures

Each case is a directory containing:

- `program.echo`: the Echo program under test.

- `stdin.txt`
- `stdout.txt`
- `unsupported.txt` only when the syntax is intentionally not part of the valid
  executable strict language yet.

The parser fixture harness parses every supported Echo fixture. The `xo` fixture harness also invokes `ast`, `ir`, `run`, and `build` for every supported Echo fixture and writes artifacts under `test-results/echo/<fixture>/`. Fixtures without `unsupported.txt` are expected to run and produce exactly `stdout.txt`. Fixtures with `unsupported.txt` may cover invalid or not-yet-executable syntax; their command outputs are still recorded as artifacts when useful.

For fast development loops, target one fixture through the quiet check script:

```sh
scripts/check-fast jit 017_object_append_count
scripts/check-fast fixture 017_object_append_count
scripts/check-fast bench-echo 017_object_append_count
```

The first command checks the faster JIT execution path. The second command runs the full `xo ast`/`ir`/`run`/`build` fixture path for only matching fixture directories.
The benchmark shortcut defaults to two iterations and is intended as a quick regression smoke check, not a final benchmark report.

Run the ignored benchmark with:

```sh
cargo test -p xo --test echo_bench -- --ignored --nocapture
```

Use `ECHO_BENCH_ITERATIONS=2` for a quick smoke check, or `scripts/check-fast bench-echo <fixture-filter>` for a filtered benchmark smoke run. The benchmark includes both PHP compatibility fixtures and Echo fixtures, skips fixtures with `unsupported.txt`, validates built Echo binary stdout, times only the built binary, and writes reports under `test-results/echo/`.
