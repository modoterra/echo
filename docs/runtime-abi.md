# Runtime ABI

Echo's generated LLVM IR may declare many runtime symbols as PHP compatibility grows. The size of that declaration set is acceptable only when the symbols remain separated by role.

## Symbol Layers

- `echo_*`: core compiler/runtime ABI for language semantics such as output writes, value construction, dynamic calls, and shutdown.
- `echo_std_*`: approved intrinsic ABI used by trusted Echo standard library declarations.
- `echo_php_*`: PHP builtin ABI for known PHP function implementations such as `ob_start()` and `ob_flush()`.
- `echo_ext_*`: reserved for a future extension/module ABI.
- `echo_internal_*`: runtime-private implementation details. Codegen must not emit declarations or calls to these symbols.

The core ABI should stay small and stable. PHP builtin coverage and standard-library intrinsic coverage may become large, but they are routed through registries rather than ad hoc codegen symbol construction.

## Static Builtin Calls

When source code names a known PHP builtin directly, codegen may lower it to the PHP builtin ABI through the compile-time builtin registry.

Example:

```php
ob_start();
echo "hello";
ob_flush();
```

Expected shape:

```llvm
call i1 @echo_php_ob_start()
call void @echo_write(ptr @echo_str_0, i64 5)
call i1 @echo_php_ob_flush()
```

`echo` remains syntax, not a PHP function call, so it uses the core output ABI rather than an `echo_php_echo` builtin.

## Dynamic Function Calls

Variable function calls are runtime operations in PHP. They must not be rewritten to direct builtin calls just because a local variable currently holds a string literal.

Example:

```php
$fn = "ob_start";
$fn();
```

Expected shape:

```llvm
call %EchoValue @echo_call_function(ptr @echo_str_0, i64 8)
```

The runtime dispatcher resolves the string and may fail at runtime if the callable is undefined or invalid. This preserves PHP-compatible behavior for `.php` inputs.

## Shared Value ABI

Compiler-facing calls should converge on `%EchoValue = type { i32, i64 }` for values crossing runtime/function boundaries. The current discriminants are:

- `-1`: error sentinel.
- `0`: null.
- `1`: bool, with payload `0` for false and `1` for true.
- `2`: signed integer, stored in the payload bits.
- `3`: runtime string handle.
- `4`: array handle, reserved for the upcoming array representation.

Direct `echo "literal"` may still use `echo_write(ptr, i64)` as a core output fast path. Function-call returns should prefer `%EchoValue`; generated code can print values through `echo_write_value(%EchoValue)`. This avoids ad hoc return ABIs such as raw `ptr` for strings, `i64` for ints, and sentinel integers for `int|false`.

## Registry Boundaries

Echo has two distinct lookup concepts:

- Codegen builtin registry: maps static PHP source-level function names to direct `echo_php_*` symbols.
- Runtime function dispatcher: resolves dynamic callables such as `$fn()` and reports runtime failures.

The codegen registry is an ABI-routing table, not a compile-time proof that every possible call is safe. Compile-time safety checks belong in a later semantic resolver, not in ABI declaration code.

## Standard Library Boundary

`echo_std` is the Echo-facing standard library layer. It should expose APIs such as networking and HTTP to Echo programs while depending on lower-level runtime primitives. PHP compatibility builtins remain in the `echo_php_*` ABI, and future optional modules should use the `echo_ext_*` ABI.

The first HTTP server should be written as an Echo program using `echo_std`, not as an `xo serve` command.

Ownership rules are documented in [Echo Standard Library](stdlib.md). In short: codegen depends on the small core runtime ABI, PHP-compatible functions use `echo_php_*`, Echo-native library APIs live in `echo_std`, optional modules use `echo_ext_*`, and runtime internals stay private.

Trusted stdlib Echo source may declare intrinsic functions and methods. Those declarations lower through a compiler-owned intrinsic binding registry to `echo_std_*` ABI symbols.

Trusted stdlib source declares modules with `namespace std ...`, for example `namespace std net`. This is a stdlib module declaration, not a PHP namespace declaration.

Example:

```php
from std use net\TcpServer

let $server = TcpServer::listen("127.0.0.1:8080")
```

Expected intrinsic binding shape:

```text
std.net.TcpServer::listen(string): TcpServer
  -> echo_std_net_tcp_server_listen
```

`echo_std_*` symbols are not looked up from arbitrary user source. User code cannot name Rust symbols, and non-stdlib files cannot declare `intrinsic` bindings.

## Compatibility And Safety Modes

Echo should support both always-on Echo superset behavior and stricter safety over time.

Strict Echo's type-system direction is documented in [Strict-Mode Type System](strict-mode-type-system.md).

Default direction:

- `.php`: Echo mode by default. Valid PHP stays valid, and Echo language features are still available.
- `.xo` and `.echo`: Strict mode by default. Echo features are available, but unsafe or ambiguous PHP compatibility patterns can become compile-time diagnostics.

The CLI can override the extension default:

```sh
xo run --strict file.php
xo run --unsafe file.echo
```

`--strict` forces strict mode. `--unsafe` forces Echo superset mode, allowing PHP compatibility patterns. `--unsafe` does not disable Echo features; it only disables strict-mode safety rejections. The important design point is that compatibility and safety are policy inputs to semantic analysis, not ABI naming rules.

In strict mode, Echo may diagnose cases such as:

```php
$fn = "ob__start";
$fn();
```

when the compiler can prove the target is a literal and no known builtin or userland function exists in the compilation unit. In Echo mode, this remains a runtime call and should fail only if executed.

## Current Output-Buffering ABI

Current PHP-facing output-buffering builtins use `echo_php_*` symbols:

```llvm
declare i1 @echo_php_ob_start()
declare i1 @echo_php_ob_start_value(%EchoValue)
declare i1 @echo_php_ob_clean()
declare i1 @echo_php_ob_flush()
declare i1 @echo_php_ob_end_flush()
declare i1 @echo_php_ob_end_clean()
declare %EchoValue @echo_php_ob_get_clean()
declare %EchoValue @echo_php_ob_get_contents()
declare %EchoValue @echo_php_ob_get_flush()
declare %EchoValue @echo_php_ob_get_length()
declare %EchoValue @echo_php_ob_get_level()
```

Current PHP-facing string builtins use unary `%EchoValue` calls so PHP scalar coercion
stays centralized in the runtime value layer:

```llvm
declare %EchoValue @echo_php_strlen(%EchoValue)
declare %EchoValue @echo_php_strtoupper(%EchoValue)
declare %EchoValue @echo_php_strtolower(%EchoValue)
declare %EchoValue @echo_php_strrev(%EchoValue)
declare %EchoValue @echo_php_ucfirst(%EchoValue)
declare %EchoValue @echo_php_lcfirst(%EchoValue)
declare %EchoValue @echo_php_ord(%EchoValue)
declare %EchoValue @echo_php_str_rot13(%EchoValue)
declare %EchoValue @echo_php_chr(%EchoValue)
declare %EchoValue @echo_php_bin2hex(%EchoValue)
declare %EchoValue @echo_php_hex2bin(%EchoValue)
declare %EchoValue @echo_php_trim(%EchoValue)
declare %EchoValue @echo_php_ltrim(%EchoValue)
declare %EchoValue @echo_php_rtrim(%EchoValue)
declare %EchoValue @echo_php_addslashes(%EchoValue)
declare %EchoValue @echo_php_stripslashes(%EchoValue)
declare %EchoValue @echo_php_quotemeta(%EchoValue)
declare %EchoValue @echo_php_str_contains(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_str_starts_with(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_str_ends_with(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_str_repeat(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_substr(%EchoValue, %EchoValue)
```

Core output behavior remains under `echo_*`:

```llvm
declare void @echo_write(ptr, i64)
declare void @echo_write_value(%EchoValue)
declare void @echo_shutdown()
declare %EchoValue @echo_call_function(ptr, i64)
```
