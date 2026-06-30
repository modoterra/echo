# Runtime ABI

Echo's generated LLVM IR may declare many runtime symbols as PHP compatibility grows. The size of that declaration set is acceptable only when the symbols remain separated by role.

## Symbol Layers

- `echo_*`: core compiler/runtime ABI for language semantics such as output writes, value construction, dynamic calls, and shutdown.
- `echo_std_*`: approved intrinsic ABI used by trusted Echo standard library declarations.
- `echo_php_*`: PHP builtin ABI for known PHP function implementations such as `ob_start()` and `ob_flush()`.
- `echo_ext_*`: reserved for a future extension/module ABI.
- `echo_internal_*`: runtime-private implementation details. Codegen must not emit declarations or calls to these symbols.

The core ABI should stay small and stable. PHP builtin coverage and standard-library intrinsic coverage may become large, but they are routed through registries rather than ad hoc codegen symbol construction.

## Rust Runtime Ownership

Echo runtime and executable semantics should be implemented in Rust crates owned by
this workspace. Generated code may call stable `echo_*`, `echo_php_*`,
`echo_std_*`, and future `echo_ext_*` ABI symbols, but those symbols should be
backed by Rust implementation code.

Do not add C/C++ runtime implementations, `libm`/`-lm`, libc math entry points,
or other non-Rust link dependencies to implement language behavior. The current
native build path uses `clang` as a bootstrap linker driver for generated LLVM IR
and `target/debug/libecho_runtime.a`; that driver is not part of the language
semantics and should not be used to smuggle in C runtime behavior. Future build
plumbing should move toward a Rust-owned link path where practical.

## Static Builtin Calls

When source code names a known PHP builtin directly, codegen may lower it to the PHP builtin ABI through the compile-time builtin registry.

Example:

```php
ob_start();
echo "hello";
ob_flush();
```

This source-level example uses a statically named PHP builtin, so codegen can route it directly through the PHP builtin ABI.

Expected shape:

```llvm
call i1 @echo_php_ob_start()
call void @echo_write(ptr @echo_str_0, i64 5)
call i1 @echo_php_ob_flush()
```

The expected IR shape shows the ABI split: `ob_*` calls use `echo_php_*`, while `echo` syntax stays on the core output ABI.

`echo` remains syntax, not a PHP function call, so it uses the core output ABI rather than an `echo_php_echo` builtin.

## Dynamic Function Calls

Variable function calls are runtime operations in PHP. They must not be rewritten to direct builtin calls just because a local variable currently holds a string literal.

Example:

```php
$fn = "ob_start";
$fn();
```

This source uses PHP's variable-function behavior, so it must remain a runtime dispatch even when the variable currently contains a builtin name.

Expected shape:

```llvm
call %EchoValue @echo_call_function(ptr @echo_str_0, i64 8)
```

The expected IR shape preserves PHP dynamism by sending the callable name to the runtime dispatcher rather than baking in a static builtin symbol.

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
- Runtime reflection registry: receives generated PHP builtin, Echo std, and userland function metadata at program startup through `echo_reflection_register_function(ptr, i64, ptr, i64, ptr, i64, i32)`. The final `i32` is the source kind, so PHP compatibility helpers such as `function_exists()` can filter to PHP globals while `std.reflect` can inspect every registered function.

The codegen registry is an ABI-routing table, not a compile-time proof that every possible call is safe. Compile-time safety checks belong in a later semantic resolver, not in ABI declaration code.

## Standard Library Boundary

`echo_std` is the Echo-facing standard library layer. It should expose APIs such as networking and HTTP to Echo programs while depending on lower-level runtime primitives where needed. PHP compatibility builtins remain in the `echo_php_*` ABI, and future optional modules should use the `echo_ext_*` ABI.

The first HTTP server should be written as an Echo program using `echo_std`, not as an `xo serve` command.

Ownership rules are documented in [Echo Standard Library](stdlib.md). In short: codegen depends on the small core runtime ABI, PHP-compatible functions use `echo_php_*`, Echo-native library APIs live in `echo_std`, optional modules use `echo_ext_*`, and runtime internals stay private.

Trusted stdlib Echo source may contain regular Echo functions/classes and may also declare intrinsic `fn` functions and methods. Regular std declarations compile through the normal Echo pipeline. Intrinsic declarations lower through a compiler-owned intrinsic binding registry to `echo_std_*` or core runtime ABI symbols. Public class methods use `pub fn` or `pub intrinsic fn`; unprefixed Echo class methods are private by default.

Trusted stdlib source declares modules with Echo module syntax such as
`module std.net`.
User/package code may not declare modules or PHP-compatible namespaces that
canonicalize to the reserved `std` root, including `module std.net`,
`namespace std\Net`, and `namespace Std\Net`.

Example:

```php
from std use net\TcpServer

let $server = TcpServer::listen("127.0.0.1:8080")
```

This is the user-facing stdlib call shape: code imports a trusted std type and calls a method without seeing Rust symbol names.

Expected intrinsic binding shape:

```text
std.net.TcpServer::listen(string): TcpServer
  -> echo_std_net_tcp_server_listen
```

The binding shape is compiler-owned metadata that connects the trusted declaration to a specific `echo_std_*` ABI symbol.

`echo_std_*` symbols are not looked up from arbitrary user source. User code cannot name Rust symbols, and non-stdlib files cannot declare `intrinsic` bindings.

## Single Language Mode

Echo compiles `.php`, `.echo`, and `.xo` files as one language. Valid PHP remains
valid, and Echo language features are available without opting into a separate
mode.

The CLI does not provide parser-mode switches. Commands compile the same
language regardless of extension:

```sh
xo run file.php
xo run file.echo
xo build file.xo -o /tmp/app
```

The ABI namespace still separates core runtime symbols, PHP builtins, stdlib
intrinsics, and future extensions. That separation is about symbol ownership,
not parser mode.

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

This declaration group is the current PHP-facing output-buffering ABI surface; adding an `ob_*` builtin should extend this layer, not core `echo_*`.

Current Echo stdlib PHP reflection intrinsics use unary `%EchoValue` calls:

```llvm
declare %EchoValue @echo_std_reflect_exists(%EchoValue)
declare %EchoValue @echo_std_reflect_params(%EchoValue)
declare %EchoValue @echo_std_reflect_return_type(%EchoValue)
declare %EchoValue @echo_std_reflect_type_of(%EchoValue)
```

These declarations are Echo stdlib intrinsics, so they use `echo_std_*` even though they expose reflection information about PHP builtins.

Current PHP-facing builtins use `%EchoValue` calls so PHP scalar coercion and
array behavior stay centralized in the runtime value layer:

```llvm
declare %EchoValue @echo_php_strlen(%EchoValue)
declare %EchoValue @echo_php_count(%EchoValue)
declare %EchoValue @echo_php_array_values(%EchoValue)
declare %EchoValue @echo_php_array_keys(%EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_array_fill(%EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_array_fill_keys(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_array_combine(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_array_pad(%EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_array_reverse(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_array_slice(%EchoValue, %EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_array_chunk(%EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_array_merge(%EchoValue)
declare %EchoValue @echo_php_array_replace(%EchoValue)
declare %EchoValue @echo_php_array_flip(%EchoValue)
declare %EchoValue @echo_php_array_count_values(%EchoValue)
declare %EchoValue @echo_php_array_key_exists(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_array_key_first(%EchoValue)
declare %EchoValue @echo_php_array_key_last(%EchoValue)
declare %EchoValue @echo_php_in_array(%EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_array_search(%EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_array_sum(%EchoValue)
declare %EchoValue @echo_php_array_product(%EchoValue)
declare %EchoValue @echo_php_function_exists(%EchoValue)
declare %EchoValue @echo_php_gettype(%EchoValue)
declare %EchoValue @echo_php_is_array(%EchoValue)
declare %EchoValue @echo_php_is_countable(%EchoValue)
declare %EchoValue @echo_php_is_iterable(%EchoValue)
declare %EchoValue @echo_php_is_numeric(%EchoValue)
declare %EchoValue @echo_php_is_null(%EchoValue)
declare %EchoValue @echo_php_is_bool(%EchoValue)
declare %EchoValue @echo_php_is_int(%EchoValue)
declare %EchoValue @echo_php_is_string(%EchoValue)
declare %EchoValue @echo_php_is_scalar(%EchoValue)
declare %EchoValue @echo_php_strval(%EchoValue)
declare %EchoValue @echo_php_boolval(%EchoValue)
declare %EchoValue @echo_php_intval(%EchoValue)
declare %EchoValue @echo_php_floatval(%EchoValue)
declare %EchoValue @echo_php_strtoupper(%EchoValue)
declare %EchoValue @echo_php_strtolower(%EchoValue)
declare %EchoValue @echo_php_ucwords(%EchoValue)
declare %EchoValue @echo_php_strrev(%EchoValue)
declare %EchoValue @echo_php_ucfirst(%EchoValue)
declare %EchoValue @echo_php_lcfirst(%EchoValue)
declare %EchoValue @echo_php_ord(%EchoValue)
declare %EchoValue @echo_php_str_rot13(%EchoValue)
declare %EchoValue @echo_php_soundex(%EchoValue)
declare %EchoValue @echo_php_wordwrap(%EchoValue, %EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_chr(%EchoValue)
declare %EchoValue @echo_php_crc32(%EchoValue)
declare %EchoValue @echo_php_bindec(%EchoValue)
declare %EchoValue @echo_php_hexdec(%EchoValue)
declare %EchoValue @echo_php_octdec(%EchoValue)
declare %EchoValue @echo_php_base_convert(%EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_deg2rad(%EchoValue)
declare %EchoValue @echo_php_rad2deg(%EchoValue)
declare %EchoValue @echo_php_sin(%EchoValue)
declare %EchoValue @echo_php_cos(%EchoValue)
declare %EchoValue @echo_php_tan(%EchoValue)
declare %EchoValue @echo_php_asin(%EchoValue)
declare %EchoValue @echo_php_acos(%EchoValue)
declare %EchoValue @echo_php_atan(%EchoValue)
declare %EchoValue @echo_php_atan2(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_intdiv(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_sinh(%EchoValue)
declare %EchoValue @echo_php_cosh(%EchoValue)
declare %EchoValue @echo_php_tanh(%EchoValue)
declare %EchoValue @echo_php_asinh(%EchoValue)
declare %EchoValue @echo_php_acosh(%EchoValue)
declare %EchoValue @echo_php_atanh(%EchoValue)
declare %EchoValue @echo_php_ceil(%EchoValue)
declare %EchoValue @echo_php_floor(%EchoValue)
declare %EchoValue @echo_php_round(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_number_format(%EchoValue, %EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_sqrt(%EchoValue)
declare %EchoValue @echo_php_exp(%EchoValue)
declare %EchoValue @echo_php_expm1(%EchoValue)
declare %EchoValue @echo_php_log(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_log10(%EchoValue)
declare %EchoValue @echo_php_log1p(%EchoValue)
declare %EchoValue @echo_php_pow(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_fdiv(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_fpow(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_hypot(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_pi()
declare %EchoValue @echo_php_fmod(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_bin2hex(%EchoValue)
declare %EchoValue @echo_php_md5(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_sha1(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_base64_encode(%EchoValue)
declare %EchoValue @echo_php_base64_decode(%EchoValue)
declare %EchoValue @echo_php_rawurlencode(%EchoValue)
declare %EchoValue @echo_php_rawurldecode(%EchoValue)
declare %EchoValue @echo_php_urlencode(%EchoValue)
declare %EchoValue @echo_php_urldecode(%EchoValue)
declare %EchoValue @echo_php_hex2bin(%EchoValue)
declare %EchoValue @echo_php_chdir(%EchoValue)
declare %EchoValue @echo_php_getcwd()
declare %EchoValue @echo_php_getenv(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_gethostname()
declare %EchoValue @echo_php_getmypid()
declare %EchoValue @echo_php_phpversion(%EchoValue)
declare %EchoValue @echo_php_php_sapi_name()
declare %EchoValue @echo_php_zend_version()
declare %EchoValue @echo_php_extension_loaded(%EchoValue)
declare %EchoValue @echo_php_get_loaded_extensions(%EchoValue)
declare %EchoValue @echo_php_get_extension_funcs(%EchoValue)
declare %EchoValue @echo_php_putenv(%EchoValue)
declare %EchoValue @echo_php_sys_get_temp_dir()
declare %EchoValue @echo_php_tempnam(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_uniqid(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_is_readable(%EchoValue)
declare %EchoValue @echo_php_is_writable(%EchoValue)
declare %EchoValue @echo_php_is_executable(%EchoValue)
declare %EchoValue @echo_php_filesize(%EchoValue)
declare %EchoValue @echo_php_fileatime(%EchoValue)
declare %EchoValue @echo_php_filectime(%EchoValue)
declare %EchoValue @echo_php_filemtime(%EchoValue)
declare %EchoValue @echo_php_fileinode(%EchoValue)
declare %EchoValue @echo_php_fileowner(%EchoValue)
declare %EchoValue @echo_php_filegroup(%EchoValue)
declare %EchoValue @echo_php_fileperms(%EchoValue)
declare %EchoValue @echo_php_filetype(%EchoValue)
declare %EchoValue @echo_php_file_get_contents(%EchoValue, %EchoValue, %EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_file_put_contents(%EchoValue, %EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_readfile(%EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_readlink(%EchoValue)
declare %EchoValue @echo_php_link(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_symlink(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_touch(%EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_copy(%EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_rename(%EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_unlink(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_mkdir(%EchoValue, %EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_rmdir(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_realpath(%EchoValue)
declare %EchoValue @echo_php_str_word_count(%EchoValue)
declare %EchoValue @echo_php_trim(%EchoValue)
declare %EchoValue @echo_php_ltrim(%EchoValue)
declare %EchoValue @echo_php_rtrim(%EchoValue)
declare %EchoValue @echo_php_addslashes(%EchoValue)
declare %EchoValue @echo_php_stripslashes(%EchoValue)
declare %EchoValue @echo_php_stripcslashes(%EchoValue)
declare %EchoValue @echo_php_quoted_printable_encode(%EchoValue)
declare %EchoValue @echo_php_quoted_printable_decode(%EchoValue)
declare %EchoValue @echo_php_htmlspecialchars(%EchoValue)
declare %EchoValue @echo_php_htmlspecialchars_decode(%EchoValue)
declare %EchoValue @echo_php_strip_tags(%EchoValue)
declare %EchoValue @echo_php_nl2br(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_quotemeta(%EchoValue)
declare %EchoValue @echo_php_str_contains(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_str_starts_with(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_str_ends_with(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_str_replace(%EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_str_ireplace(%EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_strtr(%EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_str_repeat(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_str_pad(%EchoValue, %EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_str_split(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_chunk_split(%EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_substr(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_strpos(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_stripos(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_strrpos(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_strripos(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_strstr(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_stristr(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_strrchr(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_strpbrk(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_strspn(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_strcspn(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_substr_count(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_substr_compare(%EchoValue, %EchoValue, %EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_strcmp(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_strcasecmp(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_strnatcmp(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_strnatcasecmp(%EchoValue, %EchoValue)
declare %EchoValue @echo_php_levenshtein(%EchoValue, %EchoValue, %EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_strncmp(%EchoValue, %EchoValue, %EchoValue)
declare %EchoValue @echo_php_strncasecmp(%EchoValue, %EchoValue, %EchoValue)
```

This declaration group documents the value-ABI pattern for PHP builtins: runtime coercion and PHP-compatible return values stay centralized behind `%EchoValue`.

Core output behavior remains under `echo_*`:

```llvm
declare void @echo_write(ptr, i64)
declare void @echo_write_value(%EchoValue)
declare void @echo_shutdown()
declare %EchoValue @echo_call_function(ptr, i64)
```

These symbols are core language/runtime ABI, so codegen may use them for syntax and dynamic dispatch without treating them as PHP builtins.
