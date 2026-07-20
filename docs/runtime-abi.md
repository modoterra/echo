# Runtime ABI

Contracts between codegen and the Rust runtime for AOT and JIT.

| | |
|--|--|
| **Status** | Active (minimal vertical) |
| **Owners** | `echo_runtime`, `echo_codegen_abi`, `echo_codegen` |
| **Related** | `docs/adr/0004-rust-runtime-owns-executable-semantics.md`, `docs/llvm.md` |

## Scope

Runtime symbols, value representation at the ABI boundary, calling conventions
for builtins, and guarantees that AOT and JIT share one contract.

## Facts

- Runtime crate types: `rlib` and **`staticlib`** (`libecho_runtime.a`).
- Language semantics stay in Rust; host linkers are build plumbing only.
- Symbol names live in `echo_codegen_abi`.
- **No user type names** at the ABI: int/result/option are *shapes* from surface
  syntax (`^` / `!`, lits, …). The only explicit kind surface remains width tags
  like `<i32>` on numeric lits (see `docs/semantics.md`).

### Std → runtime bridge (locked)

- Userland never names `echo_runtime_*` and never imports `/ runtime`.
- **Std sources only** may `/ runtime` and call `runtime.export(…)`.
- Codegen lowers those calls via
  `(runtime export name) → echo_runtime_*` (`echo_codegen_abi`).
- User-facing API remains `std` (`io.print`, …). Tools see real Echo in
  `std/**/*.echo` including the `runtime.*` call in the body.
- Full policy: [`stdlib.md`](stdlib.md), [`modules.md`](modules.md).

### Language runtime (not the `/ runtime` package)

List lits, for-in, index, result/option packing emit `echo_runtime_*` from
**syntax**, not from a `runtime` import.
## Symbols (v1)

| Symbol | Signature (C) | Role |
|--------|---------------|------|
| `echo_entry` | `i64 echo_entry(void)` | Generated program body |
| `main` | `i32 main(void)` | Process entry → truncates `echo_entry` |
| `echo_runtime_abort` | `void echo_runtime_abort(const uint8_t *ptr, size_t len)` | Hard abort (rare; not `!`) |
| `echo_runtime_print_i64` | `void echo_runtime_print_i64(int64_t v)` | Print **string** handle only + newline (non-strings ignored) |
| `echo_runtime_str_from_int` | `int64_t (int64_t)` | Format signed int → string handle |
| `echo_runtime_str_from_float` | `int64_t (int64_t)` | Format float handle/bits → string handle |
| `echo_runtime_str_from_bytes` | `int64_t (int64_t)` | Bytes handle → string (UTF-8 lossy) |
| `echo_runtime_str_from_duration` | `int64_t (int64_t)` | Duration nanos → string (`5s`, `10ms`, …) |
| `echo_runtime_locator_from_utf8` | `int64_t (const uint8_t *p, size_t n)` | Locator handle from path/URI text |
| `echo_runtime_str_from_locator` | `int64_t (int64_t)` | Locator → string (path/URI text) |
| `echo_runtime_bytes_from_ptr` | `int64_t (const uint8_t *p, size_t n)` | Bytes handle from payload copy |
| `echo_runtime_float_from_f64` | `int64_t (double)` | Box f64 as heap float handle |
| `echo_runtime_float_to_f64` | `double (int64_t)` | Unbox heap float (or bitcast fallback) |
| `echo_runtime_string_from_utf8` | `int64_t echo_runtime_string_from_utf8(const uint8_t *p, size_t n)` | String handle |
| `echo_runtime_eq` / `_ne` | `int64_t (int64_t, int64_t)` | **Deep** eq (lists/structs recursive; string/bytes content; ints) |
| `echo_runtime_eq_id` / `_ne_id` | `int64_t (int64_t, int64_t)` | **Identity** eq (`===` / `!==` — handle/bit pattern) |
| `echo_runtime_string_builder_*` | new / push_str / push_value / finish | Rich `{name}` interpolation |
| `echo_runtime_list_new` | `int64_t echo_runtime_list_new(void)` | Empty list handle (pointer bits) |
| `echo_runtime_list_push` | `void echo_runtime_list_push(int64_t list, int64_t v)` | Append element |
| `echo_runtime_list_len` | `int64_t echo_runtime_list_len(int64_t list)` | Length |
| `echo_runtime_list_get` | `int64_t echo_runtime_list_get(int64_t list, int64_t i)` | Element or 0 |
| `echo_runtime_list_set` | `void echo_runtime_list_set(int64_t list, int64_t i, int64_t v)` | Store or soft no-op OOB |
| `echo_runtime_struct_new` | `int64_t echo_runtime_struct_new(void)` | Empty **anonymous** struct handle (no type tag) |
| `echo_runtime_struct_new_named` | `int64_t (const uint8_t *name, size_t)` | Empty struct with `% Shape` type tag |
| `echo_runtime_struct_type_is` | `int64_t (int64_t, const uint8_t *name, size_t)` | 1 if handle’s type tag equals `name` (for `|` `% Type` arms) |
| `echo_runtime_struct_set` | `void (int64_t, const uint8_t *name, size_t, int64_t)` | Insert/replace field by name |
| `echo_runtime_struct_get` | `int64_t (int64_t, const uint8_t *name, size_t)` | Field by name, or 0 |

## Value wire (v1)

| Shape (from syntax) | LLVM return | Notes |
|---------------------|-------------|--------|
| plain | `i64` | bool as 0/1; untagged ints default i64 |
| result (`!` path in fn) | `i128` | high 64 = tag (`0` ok, `1` err); low 64 = payload |
| option (bare `^` + valued `^`) | `i128` | high 64 = tag (`0` some, `1` none); low 64 = payload |
| list (`[…]`) | `i64` handle | runtime heap list (magic header + elems) |
| range (`lo..hi`) | `i64` handle | inclusive i64 range; `list_len`/`list_get` iterate |
| function value | `i64` handle | `fn_new(code, shape)`; shape 0 plain / 1 result / 2 option |
| HTTP request parse | `i64` struct handle | `http_parse_request(raw)` via **httparse**; `method`/`path`/`body`; `headers` product (names lowercased, `-`→`_`) |
| HTTP headers complete | `i64` 0/1 | `http_headers_complete(raw)` — `\r\n\r\n` present (serve read accumulate) |
| TCP listener / stream | `i64` handle | kinds 10/11; **runtime-only** opaque ids — language model: field of `% listener` / `% conn` ([`semantics.md`](semantics.md) § Value vs reference); not a userland `RefValue` leaf |
| UDP socket | `i64` handle | kind 12; same — wrap in a std struct for userland |
| Task handle | `i64` handle | kind 13; `task_spawn_entry(code,shape)` / `task_join` / `task_join_wide` / `task_block[_wide]` via **mio** poller + worker pool |
| TCP/UDP | nonblocking | WouldBlock → `park_fd` on mio registry; multi-worker so other tasks run |
| string (`'…'` / `"…"`) | `i64` handle | runtime heap UTF-8 (magic header); rich expands escapes at lower |
| bytes (`b'…'` / `b"…"`) | `i64` handle | runtime heap byte blob (magic header); distinct from string |
| named / anon struct (`name { k: v }` / `{ k: v }`) | `i64` handle | runtime heap struct (magic header + optional type tag + name→value fields) |
| float (default `f64`) | native `double` / `f32` in SSA (language: **value**, like int) | may box at universal `i64` slots via `float_from_f64` / `float_to_f64` — **ABI packing only**, not a user ref type |
| duration (`5s`, `10ms`, …) | native `i64` nanoseconds | format with `str_from_duration` |
| locator (`p'…'` / `p"…"`) | `i64` handle | heap path/URI text; not a string |

Tagged packing is **internal**. It is not a user struct and not a keyword. Users
produce/consume result/option only via `^` / `!` and `|` match arms. Lists come
from list **literals** and are held as runtime handles. Named structs come from
`%` shapes + tagged lits; fields are read/written by name at the runtime.

Width tags (`<i32>`, …) affect literal storage; they are not general ascriptions.

## Intrinsics / primitives

| Surface | Lowering |
|---------|----------|
| `runtime.print` (std only, via `/ runtime`) | `echo_runtime_print_i64` — **strings only** |
| `runtime.str_from_int` / `str_from_float` / `str_from_bytes` / `str_from_duration` / `str_from_locator` | explicit convert → string for print |
| `runtime.http_headers_complete` | 1 if `\r\n\r\n` present |
| `runtime.http_request_complete` | 1 if headers + body for `Content-Length` (if any) ready |
| `runtime.http_parse_request` | method/path/headers/body product |
| locator lit | `echo_runtime_locator_from_utf8` |
| bytes lit | `echo_runtime_bytes_from_ptr` |
| Bare userland `print` | **Not** an intrinsic (unbound unless user-defined) |
| List lit / for / index | `echo_runtime_list_*` (language) |
| Struct lit / field get/set | `echo_runtime_struct_*` (language) |

## Open questions

- Strings / heap payloads on err side
- Multi-file / exported shapes in the wire
- JIT registration of the same symbols
