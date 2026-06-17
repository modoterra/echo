# Echo Parser Fixtures

Each case is a directory containing:

- `program.echo`: the Echo program under test.

Optional executable fixtures may also include:

- `stdin.txt`
- `stdout.txt`

The parser fixture harness parses every Echo fixture. The `xo` fixture harness also invokes `ast`, `ir`, `run`, and `build` for every Echo fixture and writes artifacts under `test-results/echo/<fixture>/`. Fixtures with `stdout.txt` are expected to run and produce exactly that output. Fixtures without `stdout.txt` may cover syntax that is not executable yet; their command outputs are still recorded as artifacts.
