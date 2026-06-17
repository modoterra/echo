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

## Implementation Model

The standard library should be a real Echo library wherever Echo can express the behavior clearly.

```text
std/                 Echo-facing source files
  net.echo           public types/classes and intrinsic declarations for networking
  http.echo          request/response types and pure HTTP helpers
  task.echo          task-facing API surface

echo_std             Rust crate that packages/embeds std source and exposes std metadata
echo_runtime         Rust crate that implements low-level runtime primitives
compiler resolver    imports std symbols and validates intrinsic bindings
codegen              lowers resolved intrinsic calls to approved ABI symbols
```

Pure value APIs should be written in Echo source:

```php
namespace std http

type Response = {
    status: int
    headers: Headers
    body: bytes
}

function text(string $body): Response {
    return Response {
        status: 200
        headers: Headers { contentType: "text/plain" }
        body: bytes($body)
    }
}
```

Resource and syscall APIs should be declared in trusted stdlib Echo source as intrinsics and implemented by Rust runtime primitives:

```php
namespace std net

class TcpServer {
    intrinsic static function listen(string $address): TcpServer
    intrinsic function accept(): TcpConnection
}

class TcpConnection {
    intrinsic function read(int $maxBytes): bytes
    intrinsic function write(bytes|string $data): int
    intrinsic function close(): void
}
```

Lowercase stdlib modules may also expose module-style intrinsic functions for value-like APIs:

```php
namespace std net

intrinsic function listen(string $address): TcpServer
intrinsic function connect(string $address): TcpConnection
intrinsic function accept(TcpServer $server): TcpConnection
intrinsic function read(TcpConnection $connection, int $maxBytes): bytes
intrinsic function write(TcpConnection $connection, bytes|string $data): int
intrinsic function close(TcpConnection $connection): void
```

The `namespace std ...` form declares the compiler's internal stdlib module identity. User code imports it with `from std use ...`; it is not a PHP namespace and does not reserve `std\...`, `Std\...`, `Echo\...`, or `EchoStd\...`.

This distinction is intentional:

```php
namespace std net
```

declares trusted stdlib module `std.net`, while:

```php
namespace std\Net
```

declares an ordinary PHP namespace named `std\Net`.

Only trusted stdlib files may use `namespace std ...`. Ordinary user files that write `namespace std net` should receive a diagnostic. Ordinary user files may still use `namespace std\Net` for PHP compatibility.

## Intrinsic Binding Rules

`intrinsic` is privileged. It means the declaration is implemented by a compiler/runtime-known operation. It is not a general userland FFI feature.

- Built-in stdlib files may declare `intrinsic` functions and methods.
- Ordinary project files cannot declare `intrinsic`; semantic analysis must reject them with a clear diagnostic.
- Intrinsic declarations must have explicit parameter and return types.
- Echo source never names arbitrary Rust symbols.
- The compiler owns an intrinsic binding registry.
- Every trusted intrinsic declaration must match a registry entry exactly.
- Codegen emits only registry-approved ABI calls.
- Runtime functions validate opaque resource kinds at the intrinsic boundary.

Example registry concept:

```text
std.net.TcpServer::listen(string): TcpServer
  intrinsic: std.net.tcp_server.listen
  abi: echo_std_net_tcp_server_listen

std.net.TcpServer#accept(): TcpConnection
  intrinsic: std.net.tcp_server.accept
  abi: echo_std_net_tcp_server_accept

std.net.TcpConnection#write(bytes|string): int
  intrinsic: std.net.tcp_connection.write
  abi: echo_std_net_tcp_connection_write
```

Instance intrinsic methods pass the receiver as the first runtime argument. Static intrinsic methods do not.

```text
TcpServer::listen($address)
  -> echo_std_net_tcp_server_listen($address)

$server.accept()
  -> echo_std_net_tcp_server_accept($server)

$conn.write($bytes)
  -> echo_std_net_tcp_connection_write($conn, $bytes)
```

Intrinsic resource values should be opaque Echo values, not exposed integer handles or pointers. The runtime should reject using a `TcpConnection` where a `TcpServer` is required.

## Compiler Pipeline

For a user import:

```php
from std use net\TcpServer
```

The compiler should:

1. Recognize `from std use ...` as an Echo-owned import, not a PHP namespace import.
2. Load or reference the built-in stdlib module graph from `echo_std`.
3. Resolve `net\TcpServer` to the stdlib item `std.net.TcpServer`.
4. Make the local name `TcpServer` available in the importing file.
5. Type-check calls against the stdlib Echo declarations.
6. Lower pure Echo stdlib calls like ordinary Echo code.
7. Lower resolved intrinsic stdlib calls through the intrinsic binding registry.

This lets Echo dogfood its own syntax for types and pure helpers while keeping sockets, files, processes, timers, and scheduler operations backed by Rust.

Examples:

- `echo_write(ptr, len)` is core runtime ABI because `echo` syntax needs output semantics.
- `echo_php_strlen(...)` is PHP builtin ABI because `strlen()` is a PHP compatibility function.
- `echo_php_strtoupper(...)` and `echo_php_strtolower(...)` are PHP builtin ABI because `strtoupper()` and `strtolower()` are PHP compatibility functions.
- `echo_php_strrev(...)`, `echo_php_ucfirst(...)`, and `echo_php_lcfirst(...)` are PHP builtin ABI because `strrev()`, `ucfirst()`, and `lcfirst()` are PHP compatibility functions.
- `echo_php_ord(...)` and `echo_php_str_rot13(...)` are PHP builtin ABI because `ord()` and `str_rot13()` are PHP compatibility functions.
- `echo_php_chr(...)` and `echo_php_bin2hex(...)` are PHP builtin ABI because `chr()` and `bin2hex()` are PHP compatibility functions.
- `echo_php_trim(...)`, `echo_php_ltrim(...)`, and `echo_php_rtrim(...)` are PHP builtin ABI because `trim()`, `ltrim()`, and `rtrim()` are PHP compatibility functions.
- `std.http.Response::text(...)` belongs in Echo stdlib source because it is an Echo standard library API.
- Low-level socket polling belongs inside `echo_runtime`, with Mio hidden as an implementation detail.
- A future image-processing package could use `echo_ext_*` if it is not part of the standard library.

## HTTP Direction

Echo should be able to run a small HTTP server as an Echo program using `echo_std`, not as an `xo serve` command.

Initial target direction:

```php
<?php

namespace App\Http

from std use net\TcpServer
from std use http\Response

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

Echo-owned imports, including `from std use ...`, are documented in [Imports](imports.md).

## First Slices

- Establish the `echo_std` crate/package boundary.
- Add low-level runtime networking primitives.
- Add `echo_std` networking wrappers.
- Prove a one-request raw TCP program can compile and run.
- Build HTTP response formatting in `echo_std`.
- Add request parsing and named-handler invocation.

The first HTTP runtime slice formats text responses with the Rust `http` crate and writes those bytes over `std.net`. Request parsing is intentionally left as the next layer; do not couple Echo's public HTTP API to a Rust parser crate choice.

The concurrency runtime model remains: Mio wakes sockets, Echo wakes tasks.
