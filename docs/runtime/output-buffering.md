# Output Buffering Runtime Spec

Sources: PHP manual Output Control pages, especially:

- https://www.php.net/manual/en/function.echo.php
- https://www.php.net/manual/en/language.operators.string.php
- https://www.php.net/manual/en/outcontrol.user-level-output-buffers.php
- https://www.php.net/manual/en/outcontrol.nesting-output-buffers.php
- https://www.php.net/manual/en/outcontrol.operations-on-buffers.php
- Individual `ob_*` function pages such as `function.ob-flush.php`, `function.ob-end-flush.php`, `function.ob-clean.php`, and `function.ob-end-clean.php`.
- https://www.php.net/manual/en/function.ob-get-level.php
- https://www.php.net/manual/en/function.ob-get-contents.php

## Scope

Echo implements PHP user-level output buffers as a runtime-managed stack. Generated LLVM must route output through `echo_runtime` functions, not directly to `printf`, `puts`, or raw stdout writes.

This spec covers the no-handler subset first. Output handler callbacks, compression handlers, URL rewriting, HTTP/server buffer integration, configurable buffer flags, and chunk-size auto-flush are deferred.

## Runtime Model

- The runtime owns a stack of output buffers.
- In PHP CLI, `output_buffering` is always off by default, so a script starts with no user-level output buffer unless code calls `ob_start()`.
- In non-CLI PHP, php.ini can start an initial output buffer via `output_buffering` or `output_handler`; Echo does not model that yet.
- `echo_write(bytes)` appends to the active top buffer when one exists.
- If no output buffer is active, `echo_write(bytes)` writes to stdout.
- `ob_start()` pushes a new empty buffer onto the stack.
- Most `ob_*` operations affect only the active buffer: the last buffer started.
- Nested buffers isolate output: output written into an inner buffer is not visible to its parent until the inner buffer is flushed or ended with flush.
- Flushing a nested buffer sends its bytes to the parent buffer, not directly to stdout.
- Flushing the outermost buffer sends its bytes to stdout.
- At PHP script shutdown, unclosed buffers are flushed and turned off in reverse start order.

PHP manual source notes:

- User-level buffers can be started, manipulated, and terminated from PHP code.
- Flushing sends and discards the contents of the active buffer.
- Every output buffer not closed by script end or `exit()` is flushed and turned off by PHP shutdown in reverse start order.

## User Buffers vs System Buffers

PHP has two different buffering layers that must not be confused:

- User-level output buffers are controlled by most `ob_*` functions, such as `ob_start()`, `ob_flush()`, `ob_end_flush()`, `ob_clean()`, and `ob_end_clean()`.
- System/SAPI buffers are controlled by `flush()` and `ob_implicit_flush()`/`implicit_flush`.

`flush()` only asks PHP and the backend/SAPI to flush system write buffers. It does not flush user-level output buffers. If a user buffer is active, `flush()` alone cannot make buffered `echo` output visible. The PHP pattern for forcing buffered output outward is `ob_flush(); flush();`: first move bytes out of the active user buffer, then ask lower layers to flush.

In CLI, system flushing is output-only. In web SAPIs, flushing may send headers first and then output. Echo currently targets CLI-style binaries and has no HTTP/header layer.

## Implicit Flush

- `implicit_flush` defaults to `false` in php.ini, but defaults to `true` under PHP CLI SAPI.
- `ob_implicit_flush(true)` turns on implicit system flushing; `ob_implicit_flush(false)` turns it off.
- Implicit flushing is equivalent to calling `flush()` after every non-empty output block.
- It does not call `ob_flush()` and does not flush user-level output buffers.
- Empty strings and headers are not considered output and do not trigger implicit flushing.
- Control characters such as `"\n"`, `"\r"`, and `"\0"` still count as output and can trigger implicit flushing.
- Echo currently flushes Rust stdout immediately when bytes reach the system-output layer. That is closer to CLI implicit flushing being on, but Echo does not yet expose `flush()` or `ob_implicit_flush()` as PHP functions.

## Function Semantics

### `ob_start()`

- Pushes an empty output buffer.
- `ob_start(null)` explicitly starts a buffer without an output callback.
- Returns `true` in PHP on success.
- Echo supports statement-form `ob_start();` and `ob_start(null);`; generated code may call either the legacy no-argument runtime entry or the value-based runtime entry with `EchoValue::Null`.
- Deferred: non-null callbacks, `chunk_size`, flags, failure modes.

### `flush()`

- Flushes PHP/system/SAPI output buffers only.
- Does not affect active user-level output buffers.
- Returns no value.
- Echo has not implemented this as a PHP-callable function yet. Current runtime writes to stdout and flushes Rust stdout immediately once bytes reach the system-output layer.

### `ob_implicit_flush()`

- Enables or disables implicit system flush after each non-empty output block.
- Does not affect user-level output buffers and does not implicitly call `ob_flush()`.
- Returns no value.
- Echo has not implemented this yet.

### `ob_flush()`

- Requires an active flushable output buffer.
- Flushes/sends the active buffer contents and discards those contents.
- Does not turn off the active buffer.
- For nested buffers, flushed bytes go to the parent buffer.
- For the outermost buffer, flushed bytes go to stdout.
- Returns `true` on success, `false` on failure and PHP emits `E_NOTICE` on failure.
- Echo currently returns a runtime bool but generated code ignores it; diagnostics/notices are deferred.

### `ob_end_flush()`

- Requires an active removable output buffer.
- Flushes/sends the active buffer contents.
- Discards the active buffer contents.
- Turns off/removes the active buffer.
- For nested buffers, flushed bytes go to the parent buffer.
- For the outermost buffer, flushed bytes go to stdout.
- Returns `true` on success, `false` on failure and PHP emits `E_NOTICE` on failure.
- Echo currently returns a runtime bool but generated code ignores it; diagnostics/notices are deferred.

### `ob_clean()`

- Requires an active cleanable output buffer.
- Discards active buffer contents.
- Does not turn off the active buffer.
- Returns `true` on success, `false` on failure and PHP emits `E_NOTICE` on failure.
- Echo currently returns a runtime bool but generated code ignores it; diagnostics/notices are deferred.

### `ob_end_clean()`

- Requires an active removable output buffer.
- Discards active buffer contents.
- Turns off/removes the active buffer.
- Does not flush contents to parent/stdout.
- Returns `true` on success, `false` on failure and PHP emits `E_NOTICE` on failure.
- Echo currently returns a runtime bool but generated code ignores it; diagnostics/notices are deferred.

### `ob_get_contents()`

- Returns a copy of the active buffer contents without clearing or removing it.
- Returns `false` if no output buffer is active.
- Copying can increase memory usage because PHP returns a new string.
- Echo supports string returns as opaque runtime string handles; echoing the no-active-buffer `false` value emits an empty string like PHP.

### `ob_get_clean()`

- Returns the active buffer contents.
- Discards active buffer contents.
- Turns off/removes the active buffer.
- Returns `false` if no output buffer is active. PHP notes no `E_NOTICE` for the no-active-buffer case.
- Echo supports string returns as opaque runtime string handles; echoing the no-active-buffer `false` value emits an empty string like PHP.

### `ob_get_flush()`

- Returns the active buffer contents.
- Flushes/sends the output handler result.
- Turns off/removes the active buffer.
- Returns `false` if no output buffer is active and PHP emits `E_NOTICE` on failure.
- Echo supports string returns and flushing; echoing the no-active-buffer `false` value emits an empty string like PHP, while the `E_NOTICE` diagnostic is deferred.

### `ob_get_length()`

- Returns the active buffer content length in bytes.
- Returns `false` if no output buffer is active.
- Echo represents this as an `int|false` runtime value for supported codegen paths; echoing the no-active-buffer `false` value emits an empty string like PHP.

### `ob_get_level()`

- Returns the nesting level of active output buffering.
- The first active buffer level is `1`.
- Returns `0` when output buffering is not active.
- Echo supports statement-form `echo ob_get_level();` and does not expose the return value to variables yet.

## Deferred PHP Features

- Output handlers/callbacks and handler return values.
- Handler phase flags and status flags.
- `ob_start()` flags controlling cleanable/flushable/removable operations.
- `chunk_size` auto-flush.
- `ob_get_status()` and `ob_list_handlers()`.
- `ob_implicit_flush()` PHP function and configurable system flush mode.
- `flush()` PHP function.
- `output_add_rewrite_var()` and `output_reset_rewrite_vars()`.
- zlib output compression and other extension-provided output buffers.
- Shutdown/error/exception handler interactions.

## Current Echo Coverage

- Direct echo output without buffers.
- Single buffer `ob_start()` + `ob_end_flush()`.
- Explicit no-callback `ob_start(null)` + `ob_end_flush()`.
- Single buffer `ob_flush()` keeps buffer active and clears flushed contents.
- Single buffer `ob_end_clean()` discards buffered contents and removes buffer.
- Nested `ob_end_flush()` flushes to parent buffer.
- Nested `ob_end_clean()` discards only active inner buffer.
- Nested `ob_flush()` flushes active inner buffer to parent and keeps inner buffer active.
- `ob_flush()` without a later `ob_end_flush()` writes outermost buffer contents to stdout.
- `ob_flush()`, `ob_end_flush()`, `ob_end_clean()`, and `ob_clean()` with no active buffer preserve level `0`; PHP also emits `E_NOTICE` and returns `false`, but Echo diagnostics and observable bool returns are deferred.
- `ob_clean()` clears the active buffer without removing it.
- Shutdown auto-flushes unclosed buffers in reverse nesting order.
- `ob_get_level()` reports zero, one, and nested active buffer levels.
- `ob_get_contents()` returns an owned copy of the active buffer contents without clearing it.
- `ob_get_contents()`, `ob_get_clean()`, and `ob_get_flush()` without an active buffer return PHP `false`; in echo context this emits an empty string.
- `ob_get_length()` reports active buffer length in bytes without clearing or flushing it.
- `ob_get_length()` without an active buffer returns PHP `false`; in echo context this emits an empty string.
- `ob_get_clean()` returns active buffer contents as an owned string and removes the active buffer without flushing it.
- `ob_get_flush()` returns active buffer contents as an owned string, flushes those contents, and removes the active buffer.
- Nested `ob_get_clean()` returns and removes only the active inner buffer without writing its bytes to the parent; nested `ob_get_flush()` returns the same bytes while also writing them to the parent.

## Next Thin Slices

- Output handler callback support for `ob_start()`.
- Failure return values for `ob_flush()`, `ob_end_flush()`, `ob_clean()`, and `ob_end_clean()` with no active buffer after bool return values are observable.
