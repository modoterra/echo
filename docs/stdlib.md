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

This layout separates public Echo source, packaged std metadata, Rust runtime implementation, resolver validation, and final ABI lowering.

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

This example is the preferred shape for pure stdlib helpers: public Echo types and functions express behavior without needing a runtime intrinsic.

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

This example keeps the user-facing socket API in Echo source while routing the actual I/O work through trusted Rust runtime primitives.

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

This module-style surface supports imported calls such as `net.listen(...)` when a class-oriented API would add friction.

Tests can use the tiny assertion stdlib module:

```php
namespace std assert

intrinsic function ok(bool $condition): bool
intrinsic function equals(mixed $actual, mixed $expected): bool
```

These assertions let Echo tests run through ordinary compiled programs while still reporting failures through the runtime.

Assertion failures are reported on stderr and make the process exit nonzero at
shutdown, so `xo test` can run Echo tests through the normal compiler/runtime
path.

Function reflection is exposed as an Echo strict-mode stdlib module so Echo
programs can inspect available functions without calling PHP reflection APIs
directly:

```php
namespace std reflect

intrinsic function exists(string $name): bool
intrinsic function params(string $name): string
intrinsic function returnType(string $name): string
intrinsic function typeOf(mixed $value): string
```

This reflection surface gives Echo strict-mode code an Echo-owned way to inspect function metadata without importing PHP reflection APIs.

`params()` returns the supported PHP parameter list as a string and
`returnType()` returns the supported PHP return type string. Unknown names
return `false` from `exists()` and an empty string from the string accessors.
`typeOf()` reflects the runtime category of an Echo value, such as `null`,
`bool`, `int`, `string`, `array`, `task`, resource-like std values, or `object`.
Generated IR registers PHP builtins, Echo std functions, and userland functions
with the runtime reflection registry during program startup. PHP globals are not
declared by an Echo source file and are not importable std symbols. Echo std
function metadata is derived from packaged `std/*.echo` module declarations,
and userland function metadata is derived from parsed function declarations.

The `namespace std ...` form declares the compiler's internal stdlib module identity. User code imports it with `from std use ...`; it is not a PHP namespace and does not reserve `std\...`, `Std\...`, `Echo\...`, or `EchoStd\...`.

This distinction is intentional:

```php
namespace std net
```

This declaration names trusted module identity for the compiler and is valid only in packaged stdlib source.

declares trusted stdlib module `std.net`, while:

```php
namespace std\Net
```

This declaration remains ordinary PHP namespace syntax and should not be captured by Echo's stdlib resolver.

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

The registry concept ties a trusted Echo declaration to one approved runtime ABI symbol, preventing user code from inventing arbitrary Rust calls.

Instance intrinsic methods pass the receiver as the first runtime argument. Static intrinsic methods do not.

```text
TcpServer::listen($address)
  -> echo_std_net_tcp_server_listen($address)

$server.accept()
  -> echo_std_net_tcp_server_accept($server)

$conn.write($bytes)
  -> echo_std_net_tcp_connection_write($conn, $bytes)
```

This lowering model explains where the receiver goes at the ABI boundary and why static and instance methods differ.

Intrinsic resource values should be opaque Echo values, not exposed integer handles or pointers. The runtime should reject using a `TcpConnection` where a `TcpServer` is required.

## Compiler Pipeline

For a user import:

```php
from std use net\TcpServer
```

This import is the compiler pipeline's input: a user asks for a stdlib item without naming an ABI symbol.

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
- `echo_php_count(...)` is PHP builtin ABI because `count()` is a PHP compatibility function.
- `echo_php_function_exists(...)` is PHP builtin ABI because `function_exists()` is a PHP compatibility function.
- `echo_php_gettype(...)` is PHP builtin ABI because `gettype()` is a PHP compatibility function.
- `echo_php_is_array(...)` is PHP builtin ABI because `is_array()` is a PHP compatibility function.
- `echo_php_is_null(...)`, `echo_php_is_bool(...)`, `echo_php_is_int(...)`, `echo_php_is_string(...)`, `echo_php_is_countable(...)`, `echo_php_is_iterable(...)`, `echo_php_is_numeric(...)`, and `echo_php_is_scalar(...)` are PHP builtin ABI because the corresponding `is_*()` functions are PHP compatibility functions.
- `echo_php_strval(...)` is PHP builtin ABI because `strval()` is a PHP compatibility function.
- `echo_php_boolval(...)` is PHP builtin ABI because `boolval()` is a PHP compatibility function.
- `echo_php_intval(...)` is PHP builtin ABI because `intval()` is a PHP compatibility function.
- `echo_php_strtoupper(...)` and `echo_php_strtolower(...)` are PHP builtin ABI because `strtoupper()` and `strtolower()` are PHP compatibility functions.
- `echo_php_ucwords(...)` is PHP builtin ABI because `ucwords()` is a PHP compatibility function.
- `echo_php_strrev(...)`, `echo_php_ucfirst(...)`, and `echo_php_lcfirst(...)` are PHP builtin ABI because `strrev()`, `ucfirst()`, and `lcfirst()` are PHP compatibility functions.
- `echo_php_ord(...)` and `echo_php_str_rot13(...)` are PHP builtin ABI because `ord()` and `str_rot13()` are PHP compatibility functions.
- `echo_php_chr(...)`, `echo_php_bin2hex(...)`, and `echo_php_hex2bin(...)` are PHP builtin ABI because `chr()`, `bin2hex()`, and `hex2bin()` are PHP compatibility functions.
- `echo_php_bindec(...)`, `echo_php_hexdec(...)`, `echo_php_octdec(...)`, and `echo_php_base_convert(...)` are PHP builtin ABI because PHP exposes explicit binary, hexadecimal, octal, and arbitrary-base conversion functions.
- `echo_php_base64_encode(...)` and `echo_php_base64_decode(...)` are PHP builtin ABI because `base64_encode()` and `base64_decode()` are PHP compatibility functions.
- `echo_php_rawurlencode(...)`, `echo_php_rawurldecode(...)`, `echo_php_urlencode(...)`, and `echo_php_urldecode(...)` are PHP builtin ABI because PHP exposes separate raw URL and form/query URL encoding functions.
- `echo_php_deg2rad(...)` and `echo_php_rad2deg(...)` are PHP builtin ABI because `deg2rad()` and `rad2deg()` are PHP compatibility functions.
- `echo_php_trim(...)`, `echo_php_ltrim(...)`, and `echo_php_rtrim(...)` are PHP builtin ABI because `trim()`, `ltrim()`, and `rtrim()` are PHP compatibility functions.
- `echo_php_addslashes(...)`, `echo_php_stripslashes(...)`, and `echo_php_quotemeta(...)` are PHP builtin ABI because `addslashes()`, `stripslashes()`, and `quotemeta()` are PHP compatibility functions.
- `echo_php_str_contains(...)`, `echo_php_str_starts_with(...)`, and `echo_php_str_ends_with(...)` are PHP builtin ABI because `str_contains()`, `str_starts_with()`, and `str_ends_with()` are PHP compatibility functions.
- `echo_php_str_repeat(...)` is PHP builtin ABI because `str_repeat()` is a PHP compatibility function.
- `echo_php_substr(...)` is PHP builtin ABI because `substr()` is a PHP compatibility function.
- `echo_php_strpos(...)` is PHP builtin ABI because `strpos()` is a PHP compatibility function.
- `echo_php_stripos(...)` is PHP builtin ABI because `stripos()` is a PHP compatibility function.
- `echo_php_strrpos(...)` is PHP builtin ABI because `strrpos()` is a PHP compatibility function.
- `echo_php_strripos(...)` is PHP builtin ABI because `strripos()` is a PHP compatibility function.
- `echo_php_strstr(...)` is PHP builtin ABI because `strstr()` is a PHP compatibility function.
- `strchr()` is mapped to `echo_php_strstr(...)` because PHP defines it as an alias of `strstr()`.
- `echo_php_stristr(...)` is PHP builtin ABI because `stristr()` is a PHP compatibility function.
- `echo_php_strrchr(...)` is PHP builtin ABI because `strrchr()` is a PHP compatibility function.
- `echo_php_strpbrk(...)` is PHP builtin ABI because `strpbrk()` is a PHP compatibility function.
- `echo_php_strspn(...)` is PHP builtin ABI because `strspn()` is a PHP compatibility function.
- `echo_php_strcspn(...)` is PHP builtin ABI because `strcspn()` is a PHP compatibility function.
- `echo_php_substr_count(...)` is PHP builtin ABI because `substr_count()` is a PHP compatibility function.
- `echo_php_substr_compare(...)` is PHP builtin ABI because `substr_compare()` is a PHP compatibility function.
- `echo_php_strcmp(...)` is PHP builtin ABI because `strcmp()` is a PHP compatibility function.
- `echo_php_strcasecmp(...)` is PHP builtin ABI because `strcasecmp()` is a PHP compatibility function.
- `echo_php_is_readable(...)`, `echo_php_is_writable(...)`, `echo_php_is_executable(...)`, `echo_php_filesize(...)`, and `echo_php_realpath(...)` are PHP builtin ABI because the corresponding filesystem metadata functions are PHP compatibility functions.

Filesystem metadata helpers can be combined to validate a user-provided path before using it in a generated response:

```php
let $report = realpath(__DIR__ . "/../data/report.csv")

echo "download:" . basename($report) . "\n"
echo "readable:" . is_readable($report) . "\n"
echo "bytes:" . filesize($report) . "\n"
```

This workflow uses `realpath()` to collapse `..` segments before display or logging, then uses `basename()` when only the final filename should leave the server boundary. That is useful for generated download names, audit log labels, or UI messages where callers need `report.csv` but should not see the full canonical path. `is_readable()` and `filesize()` provide the metadata a caller would normally check before linking or serving the file.

URL encoding helpers are split by the part of the URL being built:

```php
let $department = rawurlencode("sales and marketing/Miami")
let $query = "department=" . urlencode("sales and marketing/Miami")

echo "/teams/" . $department . "?" . $query . "\n"
```

Use `rawurlencode()` for path segments so spaces become `%20` and embedded slashes are protected as `%2F`. Use `urlencode()` for form-style query values where spaces are conventionally written as `+`; decoding mirrors that distinction with `rawurldecode()` preserving literal plus signs and `urldecode()` turning them back into spaces.

Angle conversion helpers let code accept human-readable degree settings while passing radians to lower-level math or geometry code:

```php
let $heading = 90
let $radians = deg2rad($heading)

echo "heading:" . intval(rad2deg($radians)) . "\n"
```

Use `deg2rad()` at input boundaries when configuration, UI controls, or map headings are written in degrees but the next calculation expects radians. Use `rad2deg()` when reporting a computed angle back to people or storing it in degree-based settings; the conversion keeps those boundary choices explicit instead of mixing units in intermediate code.

Base conversion helpers are useful when importing identifiers, permissions, or protocol fields that arrive as text in a fixed base:

```php
let $packet_flags = bindec("1101")
let $color = hexdec("ff8800")
let $mode = octdec("0755")
let $binary_id = base_convert("a37334", 16, 2)

echo "flags:" . $packet_flags . "\n"
echo "color-id:" . $color . "\n"
echo "mode:" . $mode . "\n"
echo "binary-id:" . $binary_id . "\n"
```

Use `bindec()` for bit flags serialized as binary text, `hexdec()` for compact identifiers such as color or protocol fields, and `octdec()` for Unix-style permission strings. Use `base_convert()` when a value needs to stay textual but move between bases, such as storing a hexadecimal upstream identifier as binary text for a lower-level protocol. These helpers keep the parsing or rewriting step at the input boundary so later code can work with the representation it actually needs.

- `std.http.Response::text(...)` belongs in Echo stdlib source because it is an Echo standard library API.
- Low-level socket polling belongs inside `echo_runtime`, with Mio hidden as an implementation detail.
- A future image-processing package could use `echo_ext_*` if it is not part of the standard library.

## HTTP Direction

Echo should be able to run a small HTTP server as an Echo program using `echo_std`, not as an `xo serve` command.

Initial target direction:

```php
<?php

namespace App\Http

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
    let $conn = join run { return net.accept($server) }

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

This target program shows the intended end-to-end standard library role: Echo code owns the server shape, while stdlib modules provide networking and HTTP primitives.

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
