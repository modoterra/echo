# Echo

Echo is a Rust implementation of a PHP superset. Existing PHP should stay familiar, while Echo adds a modern runtime, compiler tooling, native concurrency, parallel execution, and a path toward compiled binaries with predictable performance gains.

The command line entrypoint is `xo`.

## Status

Echo is early-stage software. The current implementation supports a small but growing PHP-compatible slice across parsing, AST generation, LLVM IR codegen, runtime behavior, and CLI execution.

Unsupported PHP behavior should fail explicitly rather than silently approximate semantics.

## Direction

Echo is intended to feel like PHP if PHP had a modern compiler, a standard library with native networking, and an owned concurrency runtime.

Future Echo should support programs shaped like this:

```php
<?php

namespace app\http

from std use net
from std use http

type User = {
    const id: int
    email: string
    displayName?: string
}

fn responseBody($request, list<User> $users): string {
    let $body = "Hello from Echo at " . $request.path . "\n"
    return $body . "Users seen: " . count($users) . "\n"
}

let list<User> $users = {}
let $server = net.listen("127.0.0.1:8080")

loop {
    let $conn = join run {
        return net.accept($server)
    }

    run {
        let $request = http.readRequest($conn)

        $users[] = User {
            id: count($users) + 1
            email: "visitor" . count($users) . "@echo.local"
        }

        net.write($conn, http.responseText(responseBody($request, $users)))
        net.close($conn)
    }
}
```

The exact syntax will evolve, but the design goals are stable: PHP compatibility in Echo mode, stricter safety in strict mode, first-class `echo_std`, Echo-owned `loop`/`run` concurrency, and one lazy Echo event loop per thread.

## Workspace

This repository is a Rust workspace.

- `echo_source` handles source file classification and source text.
- `echo_diagnostics` provides diagnostics.
- `echo_lexer` tokenizes input.
- `echo_ast` defines syntax tree structures.
- `echo_parser` parses PHP/Echo source.
- `echo_codegen` lowers supported programs through LLVM.
- `echo_runtime` provides runtime support.
- `xo` is the CLI entrypoint.
- `www` contains the Vite React site for `xo.run`.

## Requirements

- Rust with edition 2024 support.
- LLVM 22 for full `echo_codegen` workspace builds.
- `clang` for end-to-end CLI build and run tests.
- `php` for compatibility fixture generation and PHP benchmark comparisons.
- Node.js and npm for the `www` site.

## Build And Test

Check the Rust workspace before sending a compiler/runtime change.

```bash
cargo check --workspace
cargo test --workspace
cargo fmt --all -- --check
```

This is the normal pre-commit loop for compiler and runtime edits: typecheck the whole workspace, run tests, then make sure formatting is stable.

Run and compile the same example when validating the CLI path.

```bash
cargo run -p xo -- run examples/hello.php
cargo run -p xo -- build examples/hello.php -o /tmp/hello
/tmp/hello
```

This checks both executable paths: interpreted-style `xo run` behavior and the native binary produced by `xo build`.

Use the REPL for quick expression checks before turning behavior into a fixture.

```bash
cargo run -p xo -- repl
5 + 3
count(["red", "blue"])
```

Use the REPL for quick feedback only; once the behavior matters, capture it in a fixture so the shared parser, semantics, codegen, and runtime path stay covered.

Bare expressions such as `5` or `5+3` print their resulting value.
Interactive sessions support arrow-key history saved in `~/.xo_history`.

Run Echo assertion tests in a file or directory after changing language behavior.

```bash
cargo run -p xo -- test tests/echo/031_reflection_assertions/program.echo
cargo run -p xo -- test tests/echo
```

The single-file command is useful while iterating on one behavior. The directory command checks the same assertion harness across every Echo fixture.

Mode defaults:

- `.php` files use Echo superset mode by default.
- `.echo` and `.xo` files use strict mode by default.
- `--strict` forces strict mode.
- `--unsafe` forces Echo superset mode and still keeps Echo language features enabled.

Build the website after documentation or UI changes.

```bash
cd www
npm install
npm run lint
npm run format
npm run build
```

This mirrors the documentation-site gate: install dependencies, lint, verify formatting, and build the static bundle.

## PHP Compatibility Fixtures

Compatibility fixtures live under `tests/php/<number>_<name>/`.

Each fixture includes these files.

- `program.php`
- `stdin.txt`
- `stdout.txt`

Generate expected output with system PHP when adding a compatibility fixture.

```bash
php tests/php/<fixture>/program.php < tests/php/<fixture>/stdin.txt > tests/php/<fixture>/stdout.txt
cargo test -p xo --test php_fixtures
```

The PHP command records the authoritative expected bytes. The fixture test then proves Echo parses, lowers, runs, and builds the same case.

The parser fixtures and CLI fixtures exercise the supported path through `ast`, `ir`, `run`, and `build`.

## License

Echo is licensed under the MIT License. See `LICENSE`.
