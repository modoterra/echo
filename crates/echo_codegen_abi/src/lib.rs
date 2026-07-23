//! Stable ABI symbols between codegen and `echo_runtime`.
//!
//! Both AOT (clang link) and JIT must call the same `echo_runtime_*` names.
//! See `docs/runtime-abi.md` and ADR 0004.

#![forbid(unsafe_code)]

/// Stable crate identity for workspace linkage checks.
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

/// Program entry emitted by codegen (returns process status as i64).
pub const ECHO_ENTRY: &str = "echo_entry";

/// C `main` wrapper that truncates `echo_entry` to i32.
pub const C_MAIN: &str = "main";

/// Abort the process with a UTF-8 message (`ptr`, `len`).
pub const RT_ABORT: &str = "echo_runtime_abort";

/// Print a heap value (string / list / struct) followed by newline.
/// Bare integers and floats are not printed — convert with `str_from_*` first.
pub const RT_PRINT_I64: &str = "echo_runtime_print_i64";

/// `echo_runtime_string_from_utf8(ptr, len) -> handle`
pub const RT_STRING_FROM_UTF8: &str = "echo_runtime_string_from_utf8";

/// `echo_runtime_str_from_int(i64) -> string handle`
pub const RT_STR_FROM_INT: &str = "echo_runtime_str_from_int";

/// `echo_runtime_str_from_float(f64) -> string handle`
pub const RT_STR_FROM_FLOAT: &str = "echo_runtime_str_from_float";

/// `echo_runtime_bytes_from_ptr(ptr, len) -> bytes handle`
pub const RT_BYTES_FROM_PTR: &str = "echo_runtime_bytes_from_ptr";

/// `echo_runtime_str_from_bytes(bytes_handle) -> string handle` (UTF-8 lossy)
pub const RT_STR_FROM_BYTES: &str = "echo_runtime_str_from_bytes";

/// `echo_runtime_str_from_duration(i64 nanos) -> string handle`
pub const RT_STR_FROM_DURATION: &str = "echo_runtime_str_from_duration";

/// `echo_runtime_locator_from_utf8(ptr, len) -> locator handle`
pub const RT_LOCATOR_FROM_UTF8: &str = "echo_runtime_locator_from_utf8";

/// `echo_runtime_str_from_locator(locator_handle) -> string handle`
pub const RT_STR_FROM_LOCATOR: &str = "echo_runtime_str_from_locator";

/// `echo_runtime_str_from_debug(any_i64) -> string handle` — shallow debug text
/// for REPL / diagnostics (structs, lists, strings, floats, bare ints).
pub const RT_STR_FROM_DEBUG: &str = "echo_runtime_str_from_debug";

/// `echo_runtime_str_len(string_or_bytes) -> i64` UTF-8/byte length.
pub const RT_STR_LEN: &str = "echo_runtime_str_len";

/// `echo_runtime_bytes_len(bytes) -> i64` — length of a bytes handle only.
pub const RT_BYTES_LEN: &str = "echo_runtime_bytes_len";

/// `echo_runtime_bytes_get(bytes, index) -> i64` — byte 0..255, or -1 if OOB/invalid.
pub const RT_BYTES_GET: &str = "echo_runtime_bytes_get";

/// `echo_runtime_bytes_from_i64(n) -> bytes` — 8 little-endian bytes of `n`.
pub const RT_BYTES_FROM_I64: &str = "echo_runtime_bytes_from_i64";

/// `echo_runtime_bytes_slice(b, start, end) -> bytes` — byte range `[start, end)`.
pub const RT_BYTES_SLICE: &str = "echo_runtime_bytes_slice";

/// `echo_runtime_bytes_cat(a, b) -> bytes` — concatenate two bytes values.
pub const RT_BYTES_CAT: &str = "echo_runtime_bytes_cat";

/// `echo_runtime_bytes_from_str(string) -> bytes` — UTF-8 payload copy.
pub const RT_BYTES_FROM_STR: &str = "echo_runtime_bytes_from_str";

/// `echo_runtime_str_get(s, index) -> i64` — UTF-8 byte 0..255, or -1 if OOB.
pub const RT_STR_GET: &str = "echo_runtime_str_get";

/// `echo_runtime_reflect_kind(v) -> i64` — runtime kind code (0=int, heap kinds match header).
pub const RT_REFLECT_KIND: &str = "echo_runtime_reflect_kind";

/// `echo_runtime_reflect_kind_name(v) -> string` — stable kind name (`"int"`, `"string"`, …).
pub const RT_REFLECT_KIND_NAME: &str = "echo_runtime_reflect_kind_name";

/// `echo_runtime_reflect_key_bytes(v) -> bytes` — kind-tagged key material for hashing.
pub const RT_REFLECT_KEY_BYTES: &str = "echo_runtime_reflect_key_bytes";

/// `echo_runtime_str_cat(a, b) -> string handle` — concatenate two strings (or string+bytes).
pub const RT_STR_CAT: &str = "echo_runtime_str_cat";

/// `echo_runtime_str_slice(s, start, end) -> string` — UTF-8 byte range `[start, end)`.
pub const RT_STR_SLICE: &str = "echo_runtime_str_slice";

/// `echo_runtime_str_contains(hay, needle) -> i64` — 1/0 substring match.
pub const RT_STR_CONTAINS: &str = "echo_runtime_str_contains";

/// `echo_runtime_str_starts_with(s, prefix) -> i64` — 1/0.
pub const RT_STR_STARTS_WITH: &str = "echo_runtime_str_starts_with";

/// `echo_runtime_str_ends_with(s, suffix) -> i64` — 1/0.
pub const RT_STR_ENDS_WITH: &str = "echo_runtime_str_ends_with";

/// Box an `f64` as a heap float handle (`i64` bits).
pub const RT_FLOAT_FROM_F64: &str = "echo_runtime_float_from_f64";

/// Unbox a heap float handle (or raw bitcast fallback) to `f64`.
pub const RT_FLOAT_TO_F64: &str = "echo_runtime_float_to_f64";

/// `echo_runtime_eq(a, b) -> i64` (1/0); **deep** content equality.
pub const RT_EQ: &str = "echo_runtime_eq";

/// `echo_runtime_ne(a, b) -> i64` (1/0); deep inequality.
pub const RT_NE: &str = "echo_runtime_ne";

/// `echo_runtime_eq_id(a, b) -> i64` (1/0); **identity** (handle/bit equality).
pub const RT_EQ_ID: &str = "echo_runtime_eq_id";

/// `echo_runtime_ne_id(a, b) -> i64` (1/0); identity inequality.
pub const RT_NE_ID: &str = "echo_runtime_ne_id";

/// Rich-string interpolation builder.
pub const RT_STR_BUILDER_NEW: &str = "echo_runtime_string_builder_new";
pub const RT_STR_BUILDER_PUSH_STR: &str = "echo_runtime_string_builder_push_str";
pub const RT_STR_BUILDER_PUSH_VALUE: &str = "echo_runtime_string_builder_push_value";
pub const RT_STR_BUILDER_FINISH: &str = "echo_runtime_string_builder_finish";

/// Allocate empty list; returns handle as i64 (pointer bits).
pub const RT_LIST_NEW: &str = "echo_runtime_list_new";

/// `echo_runtime_list_push(list, value)`
pub const RT_LIST_PUSH: &str = "echo_runtime_list_push";

/// `echo_runtime_list_len(list) -> i64`
pub const RT_LIST_LEN: &str = "echo_runtime_list_len";

/// `echo_runtime_list_get(list, index) -> i64` (checked soft OOB → 0)
pub const RT_LIST_GET: &str = "echo_runtime_list_get";

/// `echo_runtime_list_set(list, index, value)` (soft OOB → no-op)
pub const RT_LIST_SET: &str = "echo_runtime_list_set";
/// Inclusive range `lo..hi` handle.
pub const RT_RANGE_NEW: &str = "echo_runtime_range_new";
/// First-class function value: code pointer + ret shape.
pub const RT_FN_NEW: &str = "echo_runtime_fn_new";
pub const RT_FN_CODE: &str = "echo_runtime_fn_code";
pub const RT_FN_SHAPE: &str = "echo_runtime_fn_shape";

/// `echo_runtime_http_parse_request(raw_string_or_bytes) -> struct handle`
pub const RT_HTTP_PARSE_REQUEST: &str = "echo_runtime_http_parse_request";

/// `echo_runtime_http_headers_complete(raw) -> i64` — 1 if `\r\n\r\n` present.
pub const RT_HTTP_HEADERS_COMPLETE: &str = "echo_runtime_http_headers_complete";
/// `echo_runtime_http_request_complete(raw) -> i64` — 1 if headers + body (Content-Length) ready.
pub const RT_HTTP_REQUEST_COMPLETE: &str = "echo_runtime_http_request_complete";

/// TCP/UDP (`std/net`) — OS sockets via `echo_runtime` net module.
pub const RT_TCP_LISTEN: &str = "echo_runtime_tcp_listen";
pub const RT_TCP_ACCEPT: &str = "echo_runtime_tcp_accept";
pub const RT_TCP_CONNECT: &str = "echo_runtime_tcp_connect";
pub const RT_TCP_READ: &str = "echo_runtime_tcp_read";
pub const RT_TCP_WRITE: &str = "echo_runtime_tcp_write";
pub const RT_TCP_CLOSE: &str = "echo_runtime_tcp_close";
pub const RT_UDP_BIND: &str = "echo_runtime_udp_bind";
pub const RT_UDP_SEND_TO: &str = "echo_runtime_udp_send_to";
pub const RT_UDP_RECV_FROM: &str = "echo_runtime_udp_recv_from";
pub const RT_UDP_CLOSE: &str = "echo_runtime_udp_close";

/// Tasks / mio event loop (ADR 0013).
/// `task_spawn_entry(code_ptr, shape) -> handle` — shape 0 plain / 1 result / 2 option.
pub const RT_TASK_SPAWN_ENTRY: &str = "echo_runtime_task_spawn_entry";
/// `task_spawn_args(code, shape, argc, a0..a7) -> handle`
pub const RT_TASK_SPAWN_ARGS: &str = "echo_runtime_task_spawn_args";
/// Fail process end if unjoined tasks remain.
pub const RT_TASK_CHECK_JOINED: &str = "echo_runtime_task_check_joined";
/// `task_join(handle) -> i64` — low 64 bits of packed result.
pub const RT_TASK_JOIN: &str = "echo_runtime_task_join";
/// `task_join_wide(handle) -> i128` — full pack for result/option.
pub const RT_TASK_JOIN_WIDE: &str = "echo_runtime_task_join_wide";
/// `task_block(code_ptr, shape) -> i64` — spawn + join (plain / low bits).
pub const RT_TASK_BLOCK: &str = "echo_runtime_task_block";
/// `task_block_wide(code_ptr, shape) -> i128`.
pub const RT_TASK_BLOCK_WIDE: &str = "echo_runtime_task_block_wide";
/// `task_shape(handle) -> i64` shape code.
pub const RT_TASK_SHAPE: &str = "echo_runtime_task_shape";

/// Allocate empty anonymous struct; returns handle as i64 (pointer bits).
pub const RT_STRUCT_NEW: &str = "echo_runtime_struct_new";

/// `echo_runtime_struct_new_named(name_ptr, name_len) -> i64` — tagged `% Shape` lit.
pub const RT_STRUCT_NEW_NAMED: &str = "echo_runtime_struct_new_named";

/// `echo_runtime_struct_type_is(handle, name_ptr, name_len) -> i64` — 1 if type tag matches.
pub const RT_STRUCT_TYPE_IS: &str = "echo_runtime_struct_type_is";

/// `echo_runtime_struct_set(handle, name_ptr, name_len, value)`
pub const RT_STRUCT_SET: &str = "echo_runtime_struct_set";

/// `echo_runtime_struct_get(handle, name_ptr, name_len) -> i64`
pub const RT_STRUCT_GET: &str = "echo_runtime_struct_get";

// --- Scope-owned memory (ADR 0016) ---
/// `void echo_runtime_scope_enter(int64_t scope_id)`
pub const RT_SCOPE_ENTER: &str = "echo_runtime_scope_enter";
/// `void echo_runtime_scope_exit(int64_t scope_id)`
pub const RT_SCOPE_EXIT: &str = "echo_runtime_scope_exit";
/// `void echo_runtime_scope_register(int64_t handle)`
pub const RT_SCOPE_REGISTER: &str = "echo_runtime_scope_register";
/// `void echo_runtime_scope_promote(int64_t handle, int64_t target_scope_id)`
pub const RT_SCOPE_PROMOTE: &str = "echo_runtime_scope_promote";
/// `void echo_runtime_scope_disown(int64_t handle)`
pub const RT_SCOPE_DISOWN: &str = "echo_runtime_scope_disown";
/// `void echo_runtime_scope_release(int64_t handle)`
pub const RT_SCOPE_RELEASE: &str = "echo_runtime_scope_release";
/// `void echo_runtime_scope_enqueue_release(int64_t handle)`
pub const RT_SCOPE_ENQUEUE_RELEASE: &str = "echo_runtime_scope_enqueue_release";
/// `void echo_runtime_scope_drain_deferred(void)`
pub const RT_SCOPE_DRAIN_DEFERRED: &str = "echo_runtime_scope_drain_deferred";

/// `void echo_runtime_test_register(int64_t name_str, int64_t fn_value)`
pub const RT_TEST_REGISTER: &str = "echo_runtime_test_register";
/// `void echo_runtime_test_fail(int64_t msg_str)`
pub const RT_TEST_FAIL: &str = "echo_runtime_test_fail";
/// `int64_t echo_runtime_test_finish(void)` — fail count, or -1 if suite mode off
pub const RT_TEST_FINISH: &str = "echo_runtime_test_finish";

/// `echo_runtime_now_ms() -> i64` — wall clock ms since Unix epoch.
pub const RT_NOW_MS: &str = "echo_runtime_now_ms";
/// `echo_runtime_sleep_ms(i64)` — sleep at least `ms` milliseconds (void).
pub const RT_SLEEP_MS: &str = "echo_runtime_sleep_ms";

// --- Process / env / spawn (`std/process`) ---
/// `echo_runtime_process_args() -> list` — argv as list of strings.
pub const RT_PROCESS_ARGS: &str = "echo_runtime_process_args";
/// `echo_runtime_process_env_has(name_str) -> i64` — 1/0.
pub const RT_PROCESS_ENV_HAS: &str = "echo_runtime_process_env_has";
/// `echo_runtime_process_env_get(name_str) -> string` — empty if unset.
pub const RT_PROCESS_ENV_GET: &str = "echo_runtime_process_env_get";
/// `echo_runtime_process_env_set(name_str, value_str)` void.
pub const RT_PROCESS_ENV_SET: &str = "echo_runtime_process_env_set";
/// `echo_runtime_process_env_unset(name_str)` void.
pub const RT_PROCESS_ENV_UNSET: &str = "echo_runtime_process_env_unset";
/// `echo_runtime_process_exit(code)` void — terminates process.
pub const RT_PROCESS_EXIT: &str = "echo_runtime_process_exit";
/// `echo_runtime_process_run(program_str, args_list) -> i64` — exit code, or -1 spawn fail.
pub const RT_PROCESS_RUN: &str = "echo_runtime_process_run";
