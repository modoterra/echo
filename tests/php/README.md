# PHP Compatibility Fixtures

Each case is a directory containing:
- `program.php`: the PHP program under test.
- `stdin.txt`: bytes provided on standard input.
- `stdout.txt`: expected standard output.

These fixtures should grow from simple PHP programs toward full PHP compatibility. Until Echo has a runtime command, the Rust harness validates that every fixture is well-formed and accepted by the parser.
