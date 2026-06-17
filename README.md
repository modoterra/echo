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

namespace App\Http

from std use net\TcpServer
from std use http\Response

type User = {
    const id: int
    email: string
    displayName?: string
}

extend list<User> as $users {
    function active(): list<User> {
        return $users.filter(fn ($user): bool => $user.displayName is not null)
    }
}

let $address = "127.0.0.1:8080"
let list<User> $users = {}

let $server = TcpServer::listen($address)

while (true) {
    let $conn = join run {
        return $server.accept()
    }

    run {
        let $request = $conn.readRequest()
        $users[] = User {
            id: count($users) + 1
            email: "visitor" . count($users) . "@echo.local"
        }

        let $body = "Hello from Echo at " . $request.path . "\n"
        $body = $body . "Users seen: " . count($users) . "\n"

        $conn.write(Response::text($body))
        $conn.close()
    }
}
```

The exact syntax will evolve, but the design goals are stable: PHP compatibility in Echo mode, stricter safety in strict mode, first-class `echo_std`, explicit receiver extensions, and one lazy Echo event loop per thread.

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

Check the Rust workspace.

```bash
cargo check --workspace
```

Run all Rust tests.

```bash
cargo test --workspace
```

Check formatting.

```bash
cargo fmt --all -- --check
```

Run the CLI against an example.

```bash
cargo run -p xo -- run examples/hello.php
```

Mode defaults:

- `.php` files use Echo superset mode by default.
- `.echo` and `.xo` files use strict mode by default.
- `--strict` forces strict mode.
- `--unsafe` forces Echo superset mode and still keeps Echo language features enabled.

Build the website.

```bash
cd www
npm install
npm run build
```

## PHP Compatibility Fixtures

Compatibility fixtures live under `tests/php/<number>_<name>/`.

Each fixture includes these files.

- `program.php`
- `stdin.txt`
- `stdout.txt`

Generate expected output with system PHP when possible.

```bash
php tests/php/<fixture>/program.php < tests/php/<fixture>/stdin.txt > tests/php/<fixture>/stdout.txt
```

The parser fixtures and CLI fixtures exercise the supported path through `ast`, `ir`, `run`, and `build`.

## License

Echo is licensed under the MIT License. See `LICENSE`.
