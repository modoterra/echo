# Echo Parser Fixtures

Each case is a directory containing:

- `program.echo`: the Echo program under test.

- `stdin.txt`
- `stdout.txt`
- `unsupported.txt` only when the syntax is intentionally not executable yet.

The parser fixture harness parses every Echo fixture. The `xo` fixture harness also invokes `ast`, `ir`, `run`, and `build` for every Echo fixture and writes artifacts under `test-results/echo/<fixture>/`. Fixtures without `unsupported.txt` are expected to run and produce exactly `stdout.txt`. Fixtures with `unsupported.txt` may cover syntax that is not executable yet; their command outputs are still recorded as artifacts.

Run the ignored benchmark with:

```sh
cargo test -p xo --test echo_bench -- --ignored --nocapture
```

Use `ECHO_BENCH_ITERATIONS=2` for a quick smoke check. The benchmark includes both PHP compatibility fixtures and Echo fixtures, skips fixtures with `unsupported.txt`, validates built Echo binary stdout, times only the built binary, and writes reports under `test-results/echo/`.
