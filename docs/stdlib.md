# Echo Standard Library

`echo_std` is the Echo-facing standard library layer. It is where user-facing APIs such as networking and HTTP should live.

## Layering

- `echo_runtime`: low-level implementation primitives for values, output, tasks, I/O, networking, scheduling, and process integration.
- `echo_std`: standard library APIs exposed to Echo programs, built on top of runtime primitives.
- `echo_php_*`: PHP builtin compatibility exports, such as `ob_start()` and future builtins like `strlen()`.
- `echo_ext_*`: future extension/module ABI for optional modules.
- `echo_internal_*`: runtime-private implementation details, never emitted by codegen.

## Ownership Rules

| Layer | Owns | Must Not Own |
| --- | --- | --- |
| `echo_runtime` | Low-level execution machinery: values, allocation, output, dynamic calls, task scheduling, polling, sockets, files, timers, process handles. | User-facing library design, PHP compatibility naming, high-level HTTP server APIs. |
| `echo_std` | Echo-facing standard APIs such as `net_listen`, `http_response_text`, and `http_serve`. | Runtime scheduler internals, PHP builtin compatibility shims, optional extension packaging. |
| `echo_php_*` | PHP builtin compatibility functions with PHP source-level names and behavior, such as `ob_start()` and `strlen()`. | Echo-native standard library APIs or runtime-private helpers. |
| `echo_ext_*` | Future optional extension/module ABI for features that are not part of the core runtime or standard library. | Core language semantics, PHP builtins that are required for compatibility. |
| `echo_internal_*` | Private runtime implementation details. | Anything generated LLVM IR calls directly. |

The default decision rule is:

- If generated code needs a primitive for language semantics, put a small stable ABI in `echo_runtime` under `echo_*`.
- If Echo users should call it as part of the standard library, put the public API in `echo_std`.
- If PHP code expects a PHP builtin function, put the compatibility entry under `echo_php_*`.
- If it is optional/module-like, reserve it for `echo_ext_*`.
- If it is only an implementation detail, keep it private and never emit it from codegen.

Examples:

- `echo_write(ptr, len)` is core runtime ABI because `echo` syntax needs output semantics.
- `echo_php_strlen(...)` is PHP builtin ABI because `strlen()` is a PHP compatibility function.
- `http_serve(...)` belongs in `echo_std` because it is an Echo standard library API.
- Low-level socket polling belongs inside `echo_runtime`, with Mio hidden as an implementation detail.
- A future image-processing package could use `echo_ext_*` if it is not part of the standard library.

## HTTP Direction

Echo should be able to run a small HTTP server as an Echo program using `echo_std`, not as an `xo serve` command.

Initial target direction:

```php
<?php

namespace App\Http

use Echo\Net\TcpServer
use Echo\Http\Response

let $server = TcpServer::listen("127.0.0.1:8080")

while (true) {
    let $conn = join run {
        return $server.accept()
    }

    run {
        let $request = $conn.readRequest()
        $conn.write(Response::text("hello " . $request.path . "\n"))
        $conn.close()
    }
}
```

The standard library should provide the HTTP API. The runtime should provide lower-level networking and scheduling primitives.

`echo_std` is Echo-facing, but it does not need to be implemented in Echo source by default. Echo should take advantage of fast, correct, well-maintained Rust crates where they improve correctness, performance, security, or implementation velocity. The boundary that matters is the user-facing Echo API, not the implementation language.

Pure Echo implementations are still useful when they make semantics easier to audit or when the implementation is naturally expressed in Echo. Runtime-backed Rust implementations are preferred for low-level I/O, parsing, protocol handling, cryptography, compression, time, filesystem, process, and other areas where mature Rust crates provide better foundations.

Strict Echo's array/list/object/tuple/type model is documented in [Strict-Mode Type System](strict-mode-type-system.md).

## First Slices

- Establish the `echo_std` crate/package boundary.
- Add low-level runtime networking primitives.
- Add `echo_std` networking wrappers.
- Prove a one-request raw TCP program can compile and run.
- Build HTTP response formatting in `echo_std`.
- Add request parsing and named-handler invocation.

The concurrency runtime model remains: Mio wakes sockets, Echo wakes tasks.
