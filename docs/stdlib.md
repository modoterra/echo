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

Pure value APIs should be written in Echo source. Echo stdlib declarations use
`fn` for functions. Class members are private by default, so public methods must
be explicitly prefixed with `pub fn` or `pub intrinsic fn`.

```echo
module std.http

type Response = {
    status: int
    headers: Headers
    body: bytes
}

fn text(body: string): Response {
    return Response {
        status: 200
        headers: Headers { contentType: "text/plain" }
        body: bytes($body)
    }
}
```

This example is the preferred shape for pure stdlib helpers: public Echo types and functions express behavior without needing a runtime intrinsic.

Resource and syscall APIs should be declared in trusted stdlib Echo source as intrinsics and implemented by Rust runtime primitives:

```echo
module std.net

class TcpServer {
    pub intrinsic static fn listen(address: string): TcpServer
    pub intrinsic fn accept(): TcpConnection
}

class TcpConnection {
    pub intrinsic fn read(maxBytes: int): bytes
    pub intrinsic fn write(data: bytes|string): int
    pub intrinsic fn close(): void
}
```

This example keeps the user-facing socket API in Echo source while routing the actual I/O work through trusted Rust runtime primitives.

Lowercase stdlib modules may also expose module-style intrinsic functions for value-like APIs:

```echo
module std.net

intrinsic fn listen(address: string): TcpServer
intrinsic fn connect(address: string): TcpConnection
intrinsic fn accept(server: TcpServer): TcpConnection
intrinsic fn read(connection: TcpConnection, maxBytes: int): bytes
intrinsic fn write(connection: TcpConnection, data: bytes|string): int
intrinsic fn close(connection: TcpConnection): void
```

This module-style surface supports imported calls such as `net.listen(...)` when a class-oriented API would add friction.

## Time Foundation

The Echo-native `time` module is planned around typed, opaque values rather
than PHP's mutable `DateTime` model. The design target is documented in
[Echo Standard Library Time Foundation](time-foundation.md).

```echo
from std use time

time.sleep(500ms)

let $timer = time.timer()
render()

if ($timer.elapsed() > 16ms) {
    echo "slow frame"
}
```

This example shows the intended split: module functions such as `time.sleep()`
and `time.timer()` create values or interact with runtime scheduling, while
receiver methods such as `$timer.elapsed()` operate on an existing opaque time
value.

Tests can use the tiny assertion stdlib module:

```echo
module std.assert

intrinsic fn ok(condition: bool): bool
intrinsic fn equals(actual: mixed, expected: mixed): bool
```

These assertions let Echo tests run through ordinary compiled programs while still reporting failures through the runtime.

Assertion failures are reported on stderr and make the process exit nonzero at
shutdown, so `xo test` can run Echo tests through the normal compiler/runtime
path.

Function reflection is exposed as an Echo strict-mode stdlib module so Echo
programs can inspect available functions without calling PHP reflection APIs
directly:

```echo
module std.reflect

intrinsic fn exists(name: string): bool
intrinsic fn params(name: string): string
intrinsic fn returnType(name: string): string
intrinsic fn typeOf(value: mixed): string
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

Standard-library source declares the compiler's internal stdlib module identity
with Echo module syntax. User code imports it with `from std use ...`. The
canonical `std` root is reserved after module/namespace canonicalization, so
user/package code may not declare `module std...`, `namespace std\...`, or
`namespace Std\...`.

This distinction is intentional:

```echo
module std.net
```

This declaration names trusted module identity for the compiler and is valid
only in packaged stdlib source, while:

```php
namespace std\Net
```

This declaration canonicalizes to the reserved `std` root and should be rejected
for user/package code.

Only trusted stdlib files may declare under `std`. Ordinary user files that
write `module std.net`, `namespace std\Net`, or `namespace Std\Net` should
receive a diagnostic.

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
- `echo_php_array_values(...)`, `echo_php_array_keys(...)`, `echo_php_array_fill(...)`, `echo_php_array_fill_keys(...)`, `echo_php_array_combine(...)`, `echo_php_array_pad(...)`, `echo_php_array_reverse(...)`, `echo_php_array_slice(...)`, `echo_php_array_chunk(...)`, `echo_php_array_merge(...)`, `echo_php_array_replace(...)`, `echo_php_array_flip(...)`, `echo_php_array_count_values(...)`, `echo_php_array_key_exists(...)`, `echo_php_array_key_first(...)`, `echo_php_array_key_last(...)`, `echo_php_in_array(...)`, `echo_php_array_search(...)`, `echo_php_array_sum(...)`, and `echo_php_array_product(...)` are PHP builtin ABI because PHP exposes helpers for reading array keys, constructing repeated arrays, combining parallel arrays, padding rows to a target width, changing array order, extracting windows, batching values, merging numeric rows, applying keyed replacements, building value-to-key lookups, counting repeated values, checking membership, finding the first matching key, reindexing values, and aggregating numeric array contents.
- `echo_php_function_exists(...)` is PHP builtin ABI because `function_exists()` is a PHP compatibility function.
- `echo_php_gettype(...)` is PHP builtin ABI because `gettype()` is a PHP compatibility function.
- `echo_php_is_array(...)` is PHP builtin ABI because `is_array()` is a PHP compatibility function.
- `echo_php_is_null(...)`, `echo_php_is_bool(...)`, `echo_php_is_int(...)`, `echo_php_is_string(...)`, `echo_php_is_countable(...)`, `echo_php_is_iterable(...)`, `echo_php_is_numeric(...)`, and `echo_php_is_scalar(...)` are PHP builtin ABI because the corresponding `is_*()` functions are PHP compatibility functions.
- `echo_php_strval(...)` is PHP builtin ABI because `strval()` is a PHP compatibility function.
- `echo_php_boolval(...)` is PHP builtin ABI because `boolval()` is a PHP compatibility function.
- `echo_php_intval(...)` is PHP builtin ABI because `intval()` is a PHP compatibility function.
- `echo_php_floatval(...)` is PHP builtin ABI because `floatval()` and its `doubleval()` alias are PHP compatibility functions.
- `echo_php_strtoupper(...)` and `echo_php_strtolower(...)` are PHP builtin ABI because `strtoupper()` and `strtolower()` are PHP compatibility functions.
- `echo_php_ucwords(...)` is PHP builtin ABI because `ucwords()` is a PHP compatibility function.
- `echo_php_strrev(...)`, `echo_php_ucfirst(...)`, and `echo_php_lcfirst(...)` are PHP builtin ABI because `strrev()`, `ucfirst()`, and `lcfirst()` are PHP compatibility functions.
- `echo_php_ord(...)`, `echo_php_str_rot13(...)`, and `echo_php_crc32(...)` are PHP builtin ABI because `ord()`, `str_rot13()`, and `crc32()` are PHP compatibility functions.
- `echo_php_chr(...)`, `echo_php_bin2hex(...)`, `echo_php_hex2bin(...)`, `echo_php_md5(...)`, and `echo_php_sha1(...)` are PHP builtin ABI because `chr()`, `bin2hex()`, `hex2bin()`, `md5()`, and `sha1()` are PHP compatibility functions.
- `echo_php_bindec(...)`, `echo_php_hexdec(...)`, `echo_php_octdec(...)`, and `echo_php_base_convert(...)` are PHP builtin ABI because PHP exposes explicit binary, hexadecimal, octal, and arbitrary-base conversion functions.
- `echo_php_base64_encode(...)` and `echo_php_base64_decode(...)` are PHP builtin ABI because `base64_encode()` and `base64_decode()` are PHP compatibility functions.
- `echo_php_implode(...)` is PHP builtin ABI because `implode()` and its `join()` alias are PHP compatibility functions for joining array values into a string.
- `echo_php_rawurlencode(...)`, `echo_php_rawurldecode(...)`, `echo_php_urlencode(...)`, and `echo_php_urldecode(...)` are PHP builtin ABI because PHP exposes separate raw URL and form/query URL encoding functions.
- `echo_php_deg2rad(...)` and `echo_php_rad2deg(...)` are PHP builtin ABI because `deg2rad()` and `rad2deg()` are PHP compatibility functions.
- `echo_php_sin(...)`, `echo_php_cos(...)`, `echo_php_tan(...)`, `echo_php_asin(...)`, `echo_php_acos(...)`, `echo_php_atan(...)`, and `echo_php_atan2(...)` are PHP builtin ABI because PHP exposes trigonometric helpers as compatibility functions.
- `echo_php_sinh(...)`, `echo_php_cosh(...)`, `echo_php_tanh(...)`, `echo_php_asinh(...)`, `echo_php_acosh(...)`, and `echo_php_atanh(...)` are PHP builtin ABI because PHP exposes hyperbolic math helpers as compatibility functions.
- `echo_php_ceil(...)`, `echo_php_floor(...)`, `echo_php_sqrt(...)`, and `echo_php_hypot(...)` are PHP builtin ABI because PHP exposes rounding and magnitude helpers as compatibility functions.
- `echo_php_exp(...)`, `echo_php_expm1(...)`, `echo_php_log(...)`, `echo_php_log10(...)`, `echo_php_log1p(...)`, `echo_php_pow(...)`, `echo_php_fdiv(...)`, and `echo_php_fpow(...)` are PHP builtin ABI because PHP exposes exponential, logarithmic, IEEE division, and IEEE power helpers as compatibility functions.
- `echo_php_pi(...)` and `echo_php_fmod(...)` are PHP builtin ABI because PHP exposes pi and floating-point remainder helpers as compatibility functions.
- `echo_php_trim(...)`, `echo_php_ltrim(...)`, and `echo_php_rtrim(...)` are PHP builtin ABI because `trim()`, `ltrim()`, and `rtrim()` are PHP compatibility functions.
- `echo_php_addslashes(...)`, `echo_php_stripslashes(...)`, and `echo_php_quotemeta(...)` are PHP builtin ABI because `addslashes()`, `stripslashes()`, and `quotemeta()` are PHP compatibility functions.
- `echo_php_str_contains(...)`, `echo_php_str_starts_with(...)`, and `echo_php_str_ends_with(...)` are PHP builtin ABI because `str_contains()`, `str_starts_with()`, and `str_ends_with()` are PHP compatibility functions.
- `echo_php_str_repeat(...)` and `echo_php_str_pad(...)` are PHP builtin ABI because `str_repeat()` and `str_pad()` are PHP compatibility functions for constructing strings with repeated bytes.
- `echo_php_str_split(...)` and `echo_php_chunk_split(...)` are PHP builtin ABI because `str_split()` and `chunk_split()` are PHP compatibility functions for fixed-width byte chunks.
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
- `echo_php_quoted_printable_encode(...)`, `echo_php_quoted_printable_decode(...)`, `echo_php_nl2br(...)`, `echo_php_str_replace(...)`, `echo_php_str_ireplace(...)`, and `echo_php_strtr(...)` are PHP builtin ABI because the corresponding string rewrite and transfer-encoding helpers are PHP compatibility functions.
- `echo_php_chdir(...)`, `echo_php_getcwd(...)`, `echo_php_getenv(...)`, `echo_php_gethostname(...)`, `echo_php_getmypid(...)`, and `echo_php_putenv(...)` are PHP builtin ABI because the corresponding working-directory, environment, hostname, and process-ID helpers are PHP compatibility functions for process-local state.
- `echo_php_sys_get_temp_dir(...)`, `echo_php_tempnam(...)`, `echo_php_is_readable(...)`, `echo_php_is_writable(...)`, `echo_php_is_executable(...)`, `echo_php_filesize(...)`, `echo_php_fileatime(...)`, `echo_php_filectime(...)`, `echo_php_filemtime(...)`, `echo_php_fileinode(...)`, `echo_php_fileowner(...)`, `echo_php_filegroup(...)`, `echo_php_fileperms(...)`, `echo_php_filetype(...)`, `echo_php_file_get_contents(...)`, `echo_php_file_put_contents(...)`, `echo_php_readfile(...)`, `echo_php_readlink(...)`, `echo_php_link(...)`, `echo_php_symlink(...)`, `echo_php_touch(...)`, `echo_php_copy(...)`, `echo_php_rename(...)`, `echo_php_unlink(...)`, `echo_php_mkdir(...)`, `echo_php_rmdir(...)`, and `echo_php_realpath(...)` are PHP builtin ABI because the corresponding temporary-file, filesystem metadata, local file content, link, and local filesystem mutation functions are PHP compatibility functions.
- `echo_php_uniqid(...)` is PHP builtin ABI because `uniqid()` is a PHP compatibility helper for time-based string identifiers.

Working-directory helpers let a script run a small relative-path task from a known directory and then restore the caller's location:

```php
let $start = getcwd()

chdir(__DIR__ . "/data")
echo "bytes:" . filesize("report.csv") . "\n"
echo "cwd:" . basename(getcwd()) . "\n"
chdir($start)
```

Use `chdir()` when a group of operations naturally belongs under one directory, such as reading several fixture files, importing generated reports, or matching a legacy PHP script that expects relative paths. Capture `getcwd()` first so the original directory can be restored after the localized work; that keeps later relative paths from accidentally resolving against the temporary directory.
`basename(getcwd())` turns the active directory path into a short name that is suitable for status output. In an importer, fixture runner, or deployment step, that lets the script report `data` or `reports` as the current workspace while keeping machine-specific prefixes such as `/tmp/builds/project/...` out of logs and user-facing errors.

Array key and lookup helpers are useful when a keyed row needs to be validated, normalized for display, or reduced to totals:

```php
<?php
let $row = ["sku" => "A-42", "status" => "active", "price" => 12, "quantity" => 3];
let $required = ["sku", "status", "quantity"];
let $allowedStatuses = ["active", "paused"];
let $statusCounts = array_fill_keys($allowedStatuses, 0);
let $emptyImportRow = array_fill(0, count($required), "");
let $partialImportRow = ["A-42", "active"];
let $normalizedImportRow = array_pad($partialImportRow, count($required), "");
let $importFields = array_combine($required, $normalizedImportRow);
let $defaults = array_fill_keys($required, "");
let $completeImportFields = array_replace($defaults, $importFields);
let $columns = array_keys($row);
let $values = array_values($row);
let $displayColumns = array_reverse($columns);
let $columnOffsets = array_flip($columns);
let $statusColumn = array_search("status", $columns, true);
let $statusSummary = array_count_values([$row["status"], "paused", $row["status"]]);
let $visibleColumns = array_slice($columns, 0, 3);
let $reviewBatches = array_chunk(array_values($importFields), 2);
let $exportRow = array_merge([$row["sku"]], array_values($completeImportFields));
let $lineTotal = array_product([$row["price"], $row["quantity"]]);

echo "first:" . array_key_first($row) . "\n";
echo "last:" . array_key_last($row) . "\n";
echo "has-sku:" . array_key_exists("sku", $row) . "\n";
echo "has-quantity:" . in_array("quantity", $required, true) . "\n";
echo "status-ok:" . in_array($row["status"], $allowedStatuses, true) . "\n";
echo "known-statuses:" . implode(",", array_keys($statusCounts)) . "\n";
echo "blank-row-fields:" . count($emptyImportRow) . "\n";
echo "import-status:" . $importFields["status"] . "\n";
echo "import-quantity:" . $importFields["quantity"] . "\n";
echo "complete-quantity:" . $completeImportFields["quantity"] . "\n";
echo "columns:" . implode(",", $columns) . "\n";
echo "values:" . implode("|", $values) . "\n";
echo "display-columns:" . implode(",", $displayColumns) . "\n";
echo "status-column:" . $columnOffsets["status"] . "\n";
echo "status-search:" . $statusColumn . "\n";
echo "active-count:" . $statusSummary["active"] . "\n";
echo "visible-columns:" . implode(",", $visibleColumns) . "\n";
echo "first-review-batch:" . implode("|", $reviewBatches[0]) . "\n";
echo "export-row:" . implode("|", $exportRow) . "\n";
echo "total:" . $lineTotal . "\n";
```

Use `array_fill_keys()` to turn an allow-list into a keyed lookup or counter map, such as initializing every supported status to zero before counting imported rows. Use `array_fill()` when a fixed-width import or export row needs placeholder values without spelling out the same empty field repeatedly. `array_pad()` is useful when an imported row has fewer fields than the header and needs explicit empty trailing fields before validation. `array_combine()` can then turn the header list and normalized values into a keyed row, so downstream code reads `$importFields["quantity"]` instead of relying on a fragile numeric column offset. `array_replace()` applies imported values over a keyed default row while preserving the required field order, which is useful before validation or templated output. `array_merge()` is better for numeric export rows where later segments should be appended and reindexed, such as adding a derived leading column before the normalized row values. `array_key_exists()` is the right guard before reading required fields because it still succeeds when a present field intentionally contains `null`. `array_key_first()` and `array_key_last()` let a caller inspect the shape of an ordered row without allocating the full key list. Use `in_array(..., true)` for allow-lists such as statuses or required columns so strings like `"0"` are not treated as the same value as `0`. `array_search(..., true)` is useful when code needs the first position of a required header, while keeping the `false` miss case distinct from a real key such as `0`. `array_count_values()` turns a cleaned list of status strings into a frequency table for summaries or import validation. `array_slice()` is useful for taking a display window such as the first few visible columns without mutating the full header list. `array_chunk()` breaks a normalized row into review-sized batches, which fits paged validation screens, multi-column summaries, or rate-limited downstream writes. `array_keys()` is still useful when a caller needs every label, `array_values()` prepares keyed rows for numeric-index consumers, and `array_reverse()` can derive a display order such as showing the last column first without mutating the original row. `array_flip()` is useful when an ordered header list needs fast name-to-position lookup, such as finding the `status` column in imported CSV data; duplicate labels keep the latest original key, matching PHP. `array_sum()` or `array_product()` handle small numeric reductions such as totals, weights, or price times quantity.

Filesystem metadata helpers can be combined to validate a user-provided path before using it in a generated response:

```php
let $report = realpath(__DIR__ . "/../data/report.csv")

echo "download:" . basename($report) . "\n"
echo "readable:" . is_readable($report) . "\n"
echo "bytes:" . filesize($report) . "\n"
```

This example uses `realpath()` to collapse `..` segments before display or logging, then uses `basename()` to derive the public-facing file name from the validated path. That is the common job for `basename()`: keep server paths such as `/srv/app/data/report.csv` internal while deriving a download label, audit-log value, import summary entry, or `Content-Disposition` filename such as `report.csv`. `is_readable()` and `filesize()` provide the metadata a caller would normally check before linking or serving the file.

URL encoding helpers are split by the part of the URL being built:

```php
let $department = rawurlencode("sales and marketing/Miami")
let $query = "department=" . urlencode("sales and marketing/Miami")

echo "/teams/" . $department . "?" . $query . "\n"
```

Use `rawurlencode()` for path segments so spaces become `%20` and embedded slashes are protected as `%2F`. Use `urlencode()` for form-style query values where spaces are conventionally written as `+`; decoding mirrors that distinction with `rawurldecode()` preserving literal plus signs and `urldecode()` turning them back into spaces.

Checksum and digest helpers turn a byte string into compact identifiers for compatibility code:

```php
let $payload = "Echo\nPHP"
let $checksum = dechex(crc32($payload))
let $cacheKey = md5($payload)
let $legacyDigest = sha1($payload)

echo "crc32:" . $checksum . "\n"
echo "cache:" . $cacheKey . "\n"
echo "audit:" . substr($legacyDigest, 0, 12) . "\n"
```

Use `crc32()` for quick corruption checks or compact non-security fingerprints where existing PHP systems expect that checksum. `md5()` and `sha1()` are useful for legacy cache keys, manifest digests, or protocol interop that already names those algorithms; they should not be used for password storage or new security-sensitive decisions.

Array joining helpers turn normalized values into delimited text at an output boundary:

```php
let $columns = ["lastname", "email", "phone"]
let $record = ["Doe", "d@example.com", "555-0100"]

echo implode(",", $columns) . "\n"
echo join(",", $record) . "\n"
```

Use `implode()` when code has already validated or escaped each element and now needs a compact string format such as a CSV line, log label, cache key, or HTTP header value. `join()` is the same operation under PHP's alias name, so existing PHP code can keep whichever spelling it already uses; the separator controls the output format while array values keep their stored order.

Angle conversion helpers let code accept human-readable degree settings while passing radians to lower-level math or geometry code:

```php
let $heading = 90
let $radians = deg2rad($heading)

echo "heading:" . intval(rad2deg($radians)) . "\n"
```

Use `deg2rad()` at input boundaries when configuration, UI controls, or map headings are written in degrees but the next calculation expects radians. Use `rad2deg()` when reporting a computed angle back to people or storing it in degree-based settings; the conversion keeps those boundary choices explicit instead of mixing units in intermediate code.

Trigonometric helpers keep angle math in radians while allowing degree-facing inputs and outputs at the edges:

```php
let $heading = deg2rad(30)
let $east = intval(sin($heading) * 1000 + 0.5)
let $north = intval(cos($heading) * 1000 + 0.5)
let $bearing = intval(rad2deg(atan2($north, $east)) + 0.5)

echo "east:" . $east . "\n"
echo "north:" . $north . "\n"
echo "bearing:" . $bearing . "\n"
```

Use `sin()`, `cos()`, and `tan()` for forward calculations from an angle in radians, such as deriving vector components or slopes for movement, layout, or mapping code. Use `asin()`, `acos()`, `atan()`, and `atan2()` when measured ratios or coordinates need to become an angle again; `atan2()` is preferable for coordinates because it uses both signs to preserve the quadrant.

Hyperbolic helpers are useful when a formula describes smooth saturation or catenary-like curves instead of circular angles:

```php
let $input = floatval("0.75")
let $normalized = tanh($input)
let $restored = atanh($normalized)

echo "normalized:" . intval($normalized * 1000) . "\n"
echo "restored:" . intval($restored * 1000) . "\n"
```

Use `sinh()`, `cosh()`, and `tanh()` when modelling growth, easing, or curve formulas that use hyperbolic functions directly. Use `asinh()`, `acosh()`, and `atanh()` when a stored or measured hyperbolic value needs to be converted back to the original input scale; callers should still guard domains for `acosh()` and `atanh()` when values may come from untrusted input.

Rounding and magnitude helpers turn fractional measurements into the counts or distances an application actually needs:

```php
let $tile_size = 32
let $width = 257
let $height = 143
let $columns = ceil($width / $tile_size)
let $rows = ceil($height / $tile_size)
let $diagonal = intval(hypot($width, $height))

echo "tiles:" . $columns . "x" . $rows . "\n"
echo "diagonal:" . $diagonal . "\n"
```

Use `ceil()` when a partial unit still needs a whole allocation, such as enough tiles, pages, or batches to cover all input. Use `floor()` when extra fractional capacity should be ignored, and use `sqrt()` or `hypot()` for geometry, distance checks, and vector lengths without open-coding the square-root calculation.

Float conversion and remainder helpers make user-provided scalar settings usable in recurring numeric code:

```php
let $interval = floatval("2.5 seconds")
let $elapsed = 8.75
let $phase = fmod($elapsed, $interval)
let $circle = pi() * 2

echo "interval:" . $interval . "\n"
echo "phase-ms:" . intval($phase * 1000) . "\n"
echo "turn:" . intval($circle * 1000) . "\n"
```

Use `floatval()` when a configuration or request value may have units or labels after the numeric prefix but the calculation only needs the leading number. Use `fmod()` for wraparound calculations such as timers, animation phases, or ring-buffer positions where a fractional remainder should keep the sign and precision of the original value.

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

Padding helpers are useful when an external system expects fixed-width text identifiers:

```php
let $batch = "7"
let $sequence = "42"
let $label = "job-" . str_pad($batch, 3, "0", 0) . "-" . str_pad($sequence, 5, "0", 0)

echo $label . "\n"
```

Use `str_pad()` when a value needs a predictable display or protocol width, such as invoice numbers, log prefixes, batch labels, or aligned command output. Left-padding with zeroes keeps numeric-looking identifiers stable after PHP has parsed them as ordinary numbers; right and both-side padding are useful for table output or fixed-width file formats.

Exponential and logarithmic helpers are useful when code needs to apply a growth rate, recover the elapsed rate from a ratio, or keep precision around very small changes:

```php
let $principal = 1000
let $annualRate = 0.05
let $years = 2

let $continuous = $principal * exp($annualRate * $years)
let $doublingYears = log(2) / $annualRate
let $smallDelta = expm1(0.000001)

echo "balance:" . $continuous . "\n"
echo "doubling:" . $doublingYears . "\n"
echo "delta:" . $smallDelta . "\n"
```

Use `exp()` and `pow()` to project values forward, such as continuous or discrete growth. Use `log()`, `log10()`, and `log1p()` to recover rates, compare orders of magnitude, or handle ratios near one; `expm1()` and `log1p()` avoid precision loss when the input is close to zero.

Chunking helpers are useful when a protocol or display format limits how many bytes belong on one line:

```php
let $token = "abcdef123456"
let $pairs = str_split($token, 2)
let $wrapped = chunk_split($token, 4, "\n")

echo "pairs:" . implode("-", $pairs) . "\n"
echo $wrapped
```

Use `str_split()` when later code needs to inspect, join, or index each chunk, such as formatting a token as byte pairs. Use `chunk_split()` when the output is still one string but must be wrapped for transport, logs, or fixed-width text displays; the separator is appended after every chunk, so callers should choose a separator that is valid at the end of the formatted value.

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

let $users: list<User> = {}
let $server = net.listen("127.0.0.1:8080")

loop {
    let $conn = join run { return net.accept($server) }

    run {
        let $request = http.readRequest($conn)

        $users.push({
            id: count($users) + 1
            email: "visitor" . count($users) . "@echo.local"
        }: User)

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

The target Echo-native time surface is documented in [Echo Standard Library Time Foundation](time-foundation.md). That design makes `time.Duration`, `time.Instant`, `time.MonoInstant`, `time.Period`, and `time.Timer` opaque stdlib types; construction stays on dot-notation module functions such as `time.now()`, `time.timer()`, and `time.duration(...)`, while value behavior is defined through `extend` receiver methods such as `$timer.elapsed()` and `$duration.total_millis()`. Echo stdlib calls should use `time.sleep(...)`, not PHP namespace-call spelling such as `time\sleep(...)`.

The planned time API uses duration values rather than raw integers:

```echo
time.sleep(500ms)

let $timer = time.timer()
render()

if ($timer.elapsed() > 16ms) {
    echo "slow frame"
}
```

This keeps units explicit and separates exact elapsed `time.Duration` values from calendar-relative `time.Period` values such as `time.period(months: 1)`.

Numeric literals are not objects, so `5.seconds()` is not valid Echo. Use `5s`, `time.seconds(5)`, or `time.duration(seconds: 5)`.

## First Slices

- Establish the `echo_std` crate/package boundary.
- Add low-level runtime networking primitives.
- Add `echo_std` networking wrappers.
- Prove a one-request raw TCP program can compile and run.
- Build HTTP response formatting in `echo_std`.
- Add request parsing and named-handler invocation.

The first HTTP runtime slice formats text responses with the Rust `http` crate and writes those bytes over `std.net`. Request parsing is intentionally left as the next layer; do not couple Echo's public HTTP API to a Rust parser crate choice.

The concurrency runtime model remains: Mio wakes sockets, Echo wakes tasks.
