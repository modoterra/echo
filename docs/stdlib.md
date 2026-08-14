# Standard library and runtime package

| | |
|--|--|
| **Status** | **Locked** (resolution + `/ runtime` in std only + pipeline ownership) |
| **Owners** | See [pipeline ownership](#pipeline-ownership-locked) |
| **Related** | `docs/modules.md`, `docs/runtime-abi.md`, `docs/pipeline.md`, [`stdlib-go-gaps.md`](stdlib-go-gaps.md), ADR 0001, ADR 0004 |

## Policy (locked)

### Userland is ordinary Echo only

- No free builtins (`print` is **not** predefined).
- No `/ runtime` (rejected by resolver).
- No magic bare names and no private APIs.
- Users only see **`std`**:

```echo
/ std/io
/ std/str

io.print(str.from_int(42))
io.print('hello')
```

`io.print` prints **strings only**. Numbers and other values must be converted
explicitly (`str.from_int`, `str.from_float`, …).

Bare `print` is unbound unless the user writes `$ print = …` (ordinary local,
**not** the std/runtime implementation).

### `std` is a privileged package (resolver)

| Rule | Meaning |
|------|---------|
| Form | `/ std/…` |
| Root | (1) **Install prefix** co-located `std/` (`<prefix>/bin/xo` + `<prefix>/std`, via `scripts/install.sh`), (2) **workspace `std/`** for toolchain dev |
| Identity | Canonical path under that root |
| User `./std` | Ordinary relative package — **not** privileged |

### `/ runtime` — **std library files only**

Std authors bridge to the native runtime with a **normal import**, legal **only**
in sources under the privileged std root:

```echo
; std/io.echo
/ runtime

$ print = (value) {
    runtime.print(value)
}

\ print
```

```echo
; std/str.echo
/ runtime

$ from_int = (n) {
    ^ runtime.str_from_int(n)
}

$ from_float = (n) {
    ^ runtime.str_from_float(n)
}

$ from_bytes = (b) {
    ^ runtime.str_from_bytes(b)
}

$ from_duration = (d) {
    ^ runtime.str_from_duration(d)
}

$ from_locator = (loc) {
    ^ runtime.str_from_locator(loc)
}

$ from_debug = (v) {
    ^ runtime.str_from_debug(v)
}

\ from_int, from_float, from_bytes, from_duration, from_locator, from_debug
```

| Rule | Meaning |
|------|---------|
| Who may write `/ runtime` | Files under the **toolchain std root** only |
| Userland `/ runtime` | **Error** (`res-runtime-forbidden` or `res-import`) |
| Module name | Last segment → **`runtime`** (same as all imports) |
| Use | `runtime.export(…)` — ordinary module call surface |
| What `runtime` is | Toolchain **runtime-primitive package** (not user-authored `.echo` tree) |
| User `./runtime` | Ordinary relative module — **not** the privileged runtime package |

**No English keywords.** No `#bridge`. The only special case is **resolver
permission** + **codegen lowering** of calls whose definition is a `runtime`
export.

### What each audience sees

```text
Userland                std/io.echo                 runtime package
────────                ───────────                 ───────────────
/ std/io                / runtime                   (toolchain virtual
io.print(x)             $ print = (value) {           package of primitives)
                          runtime.print(value)
                        }
                        \ print
     │                        │                            │
     │  check / LSP /         │  check / LSP /             │  check / LSP:
     │  reflection            │  reflection                │  known exports
     │  (real Echo)           │  (real Echo)               │  + signatures
     │                        │                            │
     └────────────────────────┴── codegen ─────────────────┘
                    runtime.print → echo_runtime_*
```

### `std/test`

Suite helpers for `xo test` (see [`testing.md`](testing.md)). Exports `it`,
`bench`, `eq`, `ne`, `true`, `false`, `fail`. Bridges to `runtime.test_*`
(suite mode via env `XO_TEST`; benches need `XO_BENCH` / `xo test --bench`).

| Audience | Sees |
|----------|------|
| **User** | `io.print` (strings) + `str.from_*` (int/float/bytes/duration/locator) |
| **Std author** | Real `$ print` body calling `runtime.print` |
| **LSP / check / reflection** | Full Echo for both std and (when in graph) `runtime` exports |
| **Codegen** | `runtime.*` calls → `echo_runtime_*` symbols in `echo_runtime` |

### Surface vs implementation

| Concern | Source of truth | Lowers to native? |
|---------|-----------------|-------------------|
| `io.print` exists, signature, span | `std/io.echo` | No (tools) |
| Body of `print` | same file (includes `runtime.print`) | No (tools) |
| Goto-def `io.print` | → `std/io.echo` | — |
| Goto-def `runtime.print` (in std) | → runtime package export (metadata / table) | — |
| Call `runtime.print` in std | semantics: known primitive | **Yes → `echo_runtime_*`** |
| Call `runtime.print` in userland | cannot import `/ runtime` | — |
| Language lists / result packing | syntax | **Yes** (language runtime, not `std`) |

### Not the same as language runtime

List lits, for-in, result/option packing use `echo_runtime_*` from **syntax**,
not from `/ runtime`. That remains language-owned (see `docs/runtime-abi.md`).

---

## Pipeline ownership (locked)

Every layer must implement the same story. Hosts do **not** invent a parallel
std or free `print`.

| Stage | Crate | Responsibility |
|-------|--------|----------------|
| **Source identity** | `echo_source` | Paths; mark/query whether a file is under std root (or resolver provides it) |
| **Lex / parse / AST** | `echo_lexer`, `echo_parser`, `echo_ast` | `/ runtime` is ordinary import syntax; no special tokens |
| **Index** | `echo_index` | Facts for std modules and (when imported) runtime module exports |
| **Resolve** | `echo_resolver` | (1) `/ std/…` → toolchain std root; (2) `/ runtime` only if **importer file** is under that std root; (3) bind module name `runtime`; (4) closed graph includes runtime unit as a **synthetic/virtual module** with known exports; (5) fill `ModuleExport.return_ty` from `runtime_ret_kind` or defining-file inference |
| **Semantics** | `echo_semantics` | Treat `runtime.export` as a normal module export when import is legal; arity/use checks; imported return kinds when known; import **params** stay `value`; **never** inject free `print` into user scopes |
| **HIR / MIR** | `echo_hir`, `echo_mir` | Preserve calls; tag or keep callee as runtime primitive for codegen |
| **ABI map** | `echo_codegen_abi` (+ `echo_std`) | `(runtime export name) → echo_runtime_*` string; single name authority |
| **Codegen** | `echo_codegen` | Emit `call echo_runtime_*` for those callees; never expose symbols to Echo source |
| **Native lib** | `echo_runtime` | Implement `echo_runtime_*`; rlib + staticlib; AOT + JIT (ADR 0004) |
| **Reflection** | `echo_reflection` | Metadata from **graph exports** (std + user + runtime package table), not from `nm` alone |
| **LSP** | `echo_lsp` | Same pipeline as check: hover/complete/goto on `io.print` and on `runtime.*` inside std; optional note “runtime primitive” is presentation-only |
| **CLI** | `xo` | Orchestrate; no host-local std or print |
| **e26 / fixtures** | `echo26` | Prefer `/ std/io` + `io.print` for run cases once wired; no bare builtin `print` |

### Resolver diagnostics (planned codes)

| Code | Meaning |
|------|---------|
| `res-runtime-forbidden` | `/ runtime` outside privileged std sources |
| `res-import` | Path not found (including missing std root) |
| `sem-module-export` | `runtime.foo` not an export of the runtime package |

### Invariants (do not break)

1. **One pipeline** (ADR 0001) — LSP and `xo check` agree on names and exports.  
2. **Userland cannot name the bridge** — no `/ runtime`, no free `print`.  
3. **Std bodies are real Echo** — tools always see `$ print` and its call.  
4. **AOT and JIT** share `echo_runtime` symbols (ADR 0004).  
5. **Name authority** for native symbols is `echo_codegen_abi`, matching `#[no_mangle]` in `echo_runtime`.  
6. **No dual public API** — users are not documented toward `runtime.*` or `echo_runtime_*`.

---

## Runtime vs std surface (locked)

Two layers — do **not** mix responsibilities:

| Layer | Form | Role |
|-------|------|------|
| **`/ runtime`** (std sources only) | Free functions only (`runtime.tcp_listen`, …) | Thin bridge to `echo_runtime_*`. **No** methods. **No** user-facing types. |
| **`/ std/…`** | Ordinary Echo: **`%` shapes + free helpers and/or methods** | Product API. Named structs carry methods when useful; free fns OK too. |

### Build order (locked method)

**Always build runtime primitives first, then construct std on top.**

```text
1. echo_runtime          echo_runtime_*  (C ABI, AOT staticlib + JIT map)
2. echo_codegen_abi      RT_* name constants
3. echo_std              RUNTIME_EXPORTS  (runtime.export → native)
4. echo_codegen          declare + arity + JIT map for that symbol
5. std/**.echo           / runtime + thin wrappers, policy, % shapes
6. proofs                crate tests + xo test std/… + docs bump
```

| Rule | Meaning |
|------|---------|
| **Need OS/heap/UTF-8/clock?** | New `echo_runtime_*` first — never invent that in Echo alone |
| **Need result policy / types / names?** | Std only (`! "out of bounds"`, `% conn`, export discipline) |
| **Pure Echo enough?** | No new runtime (e.g. `list.is_empty` = `len == 0`, `set` over `hash_table`) |
| **Language syntax** | Separate: list lits, for-in, `==` may emit `echo_runtime_*` **without** a `/ runtime` import — not std |
| **Userland** | Only `/ std/…` — never `/ runtime`, never `echo_runtime_*` |

### Where logic lives (locked)

Prefer the **highest** layer that is honest and maintainable:

| Layer | Prefer when | Do **not** |
|-------|-------------|------------|
| **Pure Echo** (`std/**` without `/ runtime`) | Policy, product types, composition, simple algorithms (map/set/queue, path string helpers, log levels, SipHash paper impl if already pure) | Invent OS/crypto/network/encoding parsers in Echo when they are hard and already solved |
| **Rust runtime + crates** | OS, sockets, clocks, JSON, HTTP parse, real crypto (SHA-2, CSPRNG), hex/base64, DNS | Hand-roll hex/base64/JSON/TLS/HTTP parsers in our Rust; **use a crate** |
| **Thin `std` bridge** | Result policy, named exports, `%` types around opaque handles | Re-export `runtime.*` to userland |

**Hard, solved problems stay crates** (or thin wrappers over crates): HTTP parse (`httparse`), JSON (`serde_json`), SHA-2 (`sha2`), CSPRNG (`getrandom`), hex/base64 (`hex`, `base64`), TLS (later, e.g. rustls). Small OS/libm shims (`std::fs`, `f64::sqrt`) are fine without extra crates.

**Implement as much as possible in Echo** when the language surface is enough: collections, pure helpers, cleartext HTTP client over TCP std, path string ops, leveled log. Only add `echo_runtime_*` when Echo cannot do it correctly (or would be a bad reimplementation of a solved problem).

Authority for names: `echo_codegen_abi` + `RUNTIME_EXPORTS` must match `#[no_mangle]` in `echo_runtime`.  
See also [`runtime-abi.md`](runtime-abi.md).


### String and bytes policy (v0 — locked)

| Kind | Rule |
|------|------|
| **`std/str`** | UTF-8 text. `len` / `get` / `slice` use **byte** indices (same as today). Growth ops (`split`, `join`, `trim`, `replace`, `to_lower` / `to_upper` for ASCII) document ASCII-first where casefold is incomplete. |
| **`std/bytes`** | Opaque byte sequences; no UTF-8 assumption. Encoding helpers live under `std/encoding/*` and return **bytes** or result. |
| **Decode to str** | `str.from_bytes` remains lossy/runtime-defined; prefer `encoding` validate when adding strict UTF-8. |

### Domain map (primitives → std)

| Domain | Runtime package (`runtime.*`) | Std product | Pure Echo on top |
|--------|------------------------------|-------------|------------------|
| **I/O print** | `print` | `std/io` | — |
| **String** | `str_from_*`, `str_len`, `str_cat`, `str_get`, `str_slice`, `str_contains`, `str_starts_with`, `str_ends_with` | `std/str` | `is_empty`; result policy on `get`/`slice` |
| **Bytes** | `bytes_len`, `bytes_get`, `bytes_slice`, `bytes_cat`, `bytes_from_i64`, `bytes_from_str` | `std/bytes` | `is_empty`; result policy on `get`/`slice` |
| **List** | `list_len`, `list_get` (+ language push/index) | `std/list` | `is_empty`, `contains` |
| **Time** | `now_ms`, `sleep_ms` | `std/time` | — |
| **Process** | `process_args`, `process_env_*`, `process_exit`, `process_run` | `std/process` | option `env`, result `run` |
| **Filesystem** | `fs_*` path/file/dir + open/read/write/seek/close | `std/fs` | `% meta`, `% file` streaming |
| **Reflect** | `reflect_kind`, `reflect_kind_name`, `reflect_key_bytes` | `std/reflect` | `is_*`, `KIND_*` |
| **Test** | `test_register`, `test_fail`, `test_finish` | `std/test` | `eq` / `true` / … |
| **Net TCP/UDP** | `tcp_*`, `udp_*` | `std/net/tcp`, `udp` | `% conn` / methods |
| **HTTP** | `http_parse_request`, `http_*_complete` | `std/net/http` + request/response/server | serve loop |
| **Crypto / collections** | hash/csprng natives as added | `std/crypto/*`, `std/collections/*` | SipHash, map/set/table, queue |
| **Math** | `math_*` f64 | `std/math` | min/max/abs pure ints |
| **Encoding** | — or thin | `std/encoding/hex`, `base64` | pure roundtrip |
| **Path** | `locator_class` + `fs_join` | `std/path` | class/is_abs/is_uri; other helpers; locators |
| **JSON** | `json_parse`, `json_stringify` | `std/json` | product types only |
| **Random** | `random_*` | `std/random` | **non-crypto** unless `crypto/random` |
| **Log** | — | `std/log` | pure over `io` |
| **OS** | `os_*` | `std/os` | pid/cwd/hostname/platform |
| **DNS** | `dns_lookup` | `std/net/dns` | list of addr strings |
| **HTTP client** | thin over tcp | `std/net/http` client helpers | cleartext v0 |

**Language-owned** (not `/ runtime` package, still `echo_runtime_*` from syntax): list
lits, for-in, struct field ops, deep `==`, scope ownership, tasks, string builders.

### Checklist for a new std feature

1. **Classify** — needs native? or pure Echo?
2. If native: implement + unit-test in `echo_runtime`; add `RT_*` + `RUNTIME_EXPORTS` + codegen declare/arity/JIT.
3. Bump `RUNTIME_ABI_VERSION` / `STDLIB_VERSION` as appropriate.
4. Write `std/…` wrappers (policy, result shapes, exports); co-located `xo test`.
5. Update this inventory + [`runtime-abi.md`](runtime-abi.md) symbol table when durable.

**Net layout:** protocol **folders** (`/ std/net/tcp`, `/ std/net/udp`); role
files inside (`conn`, `listener`, `socket`). Import one path, get union exports.

**Net specifically** (aligns with [`semantics.md`](semantics.md) § Value vs reference):

1. **Runtime** stays free functions forever (`runtime.tcp_listen`, `tcp_accept`, …).
   Returns are **opaque handles** or small **anon struct products** — not Echo
   socket types.
2. **Std** exposes **named structs only**: `% listener`, `% conn`, … with a
   **`handle` field** holding those opaque bits. Methods/free helpers call
   runtime with `.handle` / `c.handle`.
3. **Passing a socket** = passing a **struct by reference** (`% conn` / `% listener`).
   There is no userland `Socket` / bare tcp type. Aliases share one object
   (one close closes for all).
4. **Reify at the boundary** — never return raw handles to app code:

```echo
; runtime.tcp_accept → anon product { conn, remote }
$ a = runtime.tcp_accept(.handle)
; a is a temporary anon struct ref — reify to named % conn:
^ conn {
    remote: a.remote,
    handle: a.conn,
    open: |
}
```

5. Users never call `runtime.tcp_*` — only `std/net/*`.

```echo
% conn {
    $ handle
    $ read = (limit) {
        ^ runtime.tcp_read(.handle, limit)
    }
}
$ c = tcp.connect(addr)       ; RefValue::Struct (% conn)
$ f = (peer) { … }            ; peer: same — struct ref
f(c)                          ; copy ref → share connection
```

Method type after monomorphic named return / method-return-other-struct is **done**.  
**Free-fn parameters** that receive a monomorphic named-struct argument at the
call site are typed for method resolve (MIR call-site flow). Free shims
(`tcp.read(c, n)`) remain optional convenience, not required for typing.

## Layout (`std/` tree)

```text
std/
  io.echo              ; may / runtime ; export print, log, …
  str.echo / bytes.echo / list.echo
  time.echo            ; now_ms, sleep_ms, mono_ms
  process.echo         ; args / env / exit / run / run_capture
  fs.echo              ; paths, whole-file, copy/rename, % meta, streaming % file, temp/symlink
  math.echo
  path.echo
  json.echo
  random.echo          ; non-crypto PRNG
  log.echo             ; leveled emit over io
  os.echo
  encoding/
    hex.echo
    base64.echo
  crypto/
    hash/sip.echo, sha256.echo
    random.echo        ; CSPRNG
  collections/
    hash_table.echo / map.echo / set.echo / queue.echo
  net/
    tcp/                 ; folder module `/ std/net/tcp`
      conn.echo          ; % conn (pass by ref)
      listener.echo      ; % listener (pass by ref)
      socket.echo        ; free listen/connect/accept/read/write/close
    udp/                 ; folder module `/ std/net/udp`
      socket_type.echo   ; % socket (pass by ref)
      socket.echo        ; free bind/send_to/recv_from/close (reify)
    request.echo
    response.echo
    server.echo
    http.echo            ; parse / serve; Content-Length body complete
    dns.echo             ; lookup
    http_client.echo     ; cleartext get (ADR 0017)
```

No `*_ops` in std for now. Multi-file `@` remains legal language (demo:
`examples/app/`).

## Style

- **struct names** lowercase `snake_case`
- Members `$` / `~` / `#` (data or function values)
- Free functions `$ name = (args) { … }` and/or methods on `%` types
- Module-scoped imports only (`docs/modules.md`)

## Export discipline (locked)

**`\ name` is the only public/private control for std (and user modules).**
Importers may use **only** exported free names and **exported** `%` / type
shapes. Everything else in a file is **file-private** — including helpers,
constants, and co-located `test.it` cases.

| Kind | Public? | Rule |
|------|---------|------|
| Free `$ name` | Only if listed in `\ …` | Factories, pure free ops, suite entrypoints |
| `% type` | Only if the type name is listed in `\ …` | Product ADTs; **methods travel with the type** (no separate `\ put`) |
| `# CONST` | Only if listed (rare) | Prefer methods / free fns over raw constants in public API |
| Nested helpers | Never | Keep unexported; same-file use only |
| Implementation types (`entry`, internal state) | Never | Do not `\ entry` just because map needs buckets |
| Co-located suite | Never | `test.it` is not an export |

### How to write `\ `

1. **Start empty** — then add only names an app or another std module must call.
2. **One line of intent** in the file header: what importers are allowed to use.
3. **Prefer methods on an exported type** over free `module.op(value, …)` for
   instance APIs (`m.put` not `map.put(m, …)`).
4. **Prefer free factories** on the module (`map.make`, `map.from_indexed`).
5. **Do not export** internal structs, bucket constants, or serve-loop plumbing
   unless they are a deliberate product API.
6. **Cross-std imports obey the same line** — `map` may only use what
   `hash_table` exports; it cannot reach `empty_buckets` without an export.

### Target public surfaces (inventory)

Exports must match the `\ ` line in each module. Status of expansive rows is in
[Expansive roadmap status](#expansive-roadmap-status-tier-ad) (Partial ≠ Done).

| Module | Export | Intentionally private |
|--------|--------|------------------------|
| `std/io` | `print`, `log`, `eprint` | — |
| `std/str` | `from_int`, `from_float`, `from_bytes`, `from_duration`, `from_locator`, `from_debug`, `len`, `is_empty`, `cat`, `contains`, `starts_with`, `ends_with`, `get`, `slice`, `trim`, `to_lower`, `to_upper`, `split`, `replace`, `join`, `repeat`, `parse_int`, `parse_float` | suite; byte indices; ASCII-first case; parse result-shaped |
| `std/list` | `len`, `is_empty`, `get`, `contains`, `sum_ints`, `sort_ints` | suite; HOF map/filter/fold **not** exported yet |
| `std/bytes` | `len`, `is_empty`, `get`, `slice`, `cat`, `from_int`, `from_str` | suite |
| `std/reflect` | `kind`, `kind_name`, `key_bytes`, `is_*`, `KIND_*` | suite (not tools `echo_reflection`) |
| `std/time` | `now_ms`, `sleep_ms`, `mono_ms`, `format`, `parse` | suite; chrono strftime-like patterns |
| `std/process` | `args`, `env`, `env_set`, `env_unset`, `exit`, `run`, `run_capture`, `run_cwd`, `spawn_pipes`, `pipe_write`, `pipe_read`, `pipe_close`, `wait` | suite; option `env`; pipes product `{ child, stdin, stdout, stderr }` |
| `std/fs` | `exists`, `is_file`, `is_dir`, `join`, `read`, `write`, `remove`, `copy`, `rename`, `create_dir`, `create_dir_all`, `read_dir`, `remove_dir`, `metadata`, `open`, `create`, `append`, `meta`, `file`, `temp_dir`, `create_temp`, `symlink`, `chmod` | suite; path string or locator; whole-file **bytes**; streaming methods on `% file` |
| `std/bufio` | `lines`, `read_lines` | suite; split on `\n` |
| `std/test` | `it`, `eq`, `ne`, `true`, `false`, `fail` | — |
| `std/crypto/hash` | `sip`, `sha256`, `sha512` (folder) | sip internals; paper keys |
| `std/crypto/hmac` | `sha256` | suite; HMAC-SHA256 |
| `std/crypto/aes_gcm` | `encrypt`, `decrypt` | suite; AES-256-GCM; key 32B nonce 12B |
| `std/crypto/random` | `fill`, `u64` | suite; **CSPRNG** (not `std/random`) |
| `std/compress/gzip` | `compress`, `decompress` | suite |
| `std/compress/zip` | `pack`, `unpack_first` | suite; single-entry thin |
| `std/encoding/csv` | `parse_line`, `parse`, `format_line` | suite; thin split (no rich quotes) |
| `std/collections/hash_table` | `hash_table`, `make` | `entry`, `empty_buckets`, SipHash constants; field `capacity`; keys via `reflect.key_bytes` |
| `std/collections/map` | `map`, `make`, `from_indexed` | suite; keys/values/entries/`to_list` (entries) snapshots |
| `std/collections/set` | `set`, `make`, `from_list` | suite; `values`/`to_list` members (no `keys`) |
| `std/collections/queue` | `queue`, `make` (methods `len`, `is_empty`, `push`, `pop`) | suite |
| `std/net/tcp` | `conn`, `listener`, free `listen`/`connect`/… | — |
| `std/net/udp` | `socket`, free bind/send/recv/close | — |
| `std/net/http` (+ request/response/server) | types; `serve`, parse/format, response helpers, `dispatch`, `handle_connection` | `status_reason` |
| `std/net/dns` | `lookup` | suite |
| `std/net/http_client` | `get`, `request`, `get_tls`, `request_tls` | suite; cleartext + TLS via runtime.tls_*; empty ca_pem = platform roots |
| `std/net/url` | `parse`, `format` | suite; http/https; runtime product fields |
| `std/net/unix` | `listener`, `conn`, `listen`, `accept`, `connect`, `read`, `write`, `close` | suite; Unix domain; handle-0 like TCP |
| `std/net/tls` | `listener`, `conn`, `listen`, `accept`, `connect`, `read`, `write`, `close`, `close_listener`, `load_pem` | handle-0 failure like TCP; rustls; empty ca_pem = platform roots |
| `std/cli` | `parse`, `has`, `get`, `positionals` | pure Echo; flat token encoding; not full getopt |
| `std/math` | `abs_i`, `min`, `max`, `sqrt`, `sin`, `cos`, `tan`, `floor`, `ceil`, `abs_f`, `pow` | suite |
| `std/encoding/hex` | `encode`, `decode` | suite |
| `std/encoding/base64` | `encode`, `decode` | suite |
| `std/path` | `join`, `class`, `is_abs`, `is_uri`, `file_name`, `parent`, `extension`, `clean`, `rel`, `walk` | suite; class 0/1/2 on string or locator; walk is shallow |
| `std/json` | `parse`, `stringify` | suite; product types only |
| `std/random` | `seed`, `u64`, `float` | suite; **not** CSPRNG — use `std/crypto/random` |
| `std/log` | `emit`, `debug`, `info`, `warn`, `error`, `kv`, `info_kv` | suite; caller passes `min_level` (no global `set_level`) |
| `std/os` | `pid`, `cwd`, `chdir`, `hostname`, `platform` | suite |

When you add a std helper, **default is private**. Export is an explicit product
decision, recorded on the `\ ` line and in this inventory when durable.

## Implementation status

| Piece | Status |
|-------|--------|
| Design / docs | **Locked** (this file) |
| Privileged std root in resolver | Entry walk + cwd + `$XO_INSTALL_ROOT` + parent of `bin/xo` when `std/` is co-installed |
| `/ runtime` gate + virtual package | **Done** (`res-runtime-forbidden`; `<echo:runtime>`) |
| `runtime.*` → `echo_runtime_*` in codegen | **Done** (`runtime.print` → `echo_runtime_print_i64`) |
| Runtime free-only (no methods on runtime) | **Locked** |
| Std named wrappers for net | **Done** — `% conn` / `% listener` methods; factories return named types |
| `runtime.http_parse_request` | **Done** — **httparse**; method/path/body + headers product |
| `runtime.tcp_*` / `runtime.udp_*` | **Done** — real OS sockets in `echo_runtime` net |
| `std/net/tcp/` | **Done** — folder: `conn` + `listener` shapes + `socket` free surface |
| `std/net/udp/` | **Done** — folder: `% socket` + free reify surface (struct by ref) |
| `std/net/http` serve / handle_connection | **Done** — accept loop + `+ handle_connection`; **Content-Length body** via `http_request_complete` |
| `std/str` | **Done** — conversions + text ops + byte `get`/`slice` |
| `std/bytes` | **Done** — `get`/`slice`/`cat`/`from_int`/`from_str` |
| `std/list` | **Done** — `len` / `is_empty` / result `get` / `contains` |
| `std/time` | **Done** — `now_ms` / `sleep_ms` via `runtime.now_ms` / `sleep_ms` |
| `std/reflect` | **Done** — runtime kind API; checker params often `value` ([`semantics.md`](semantics.md)) |
| `std/crypto/hash` | **Done** — folder module; `sip(k0, k1, msg)` SipHash-2-4 → `ui64` |
| `std/collections/hash_table` | **Done** — `hash_key` → `reflect.key_bytes`; CRUD + grow + snapshots; mixed keys |
| `std/collections/map` | **Done** — CRUD + snapshots + factories; mixed keys |
| `std/collections/set` | **Done** — add/remove/has/len + `values` snapshot + factories; mixed members |
| Std export discipline | **Locked** — this file § Export discipline; inventory table |
| Method type after free-fn return (named struct) | **Done** — lit + call-chain + monomorphic `returns_structs` |
| Method type after union return | **Done** — refine via `%` match |
| Method on raw runtime product/handle | **Out** — wrap in `%` in std instead |
| Bare userland `print` | **Not** intrinsic (unbound / unknown fn) |
| `std/io.echo` body using `/ runtime` | **Done** |


## Expansive roadmap status (Tier A–D)

**Status legend:** **Partial** = sources + runtime/ABI largely present, but three
proofs (crate · e26 · examples) and/or www/inventory not closed. **Done** =
vertical complete under AGENTS. Do not mark Done while path/fs string lifecycle
or suite failures remain.

| Module | Status | Notes |
|--------|--------|-------|
| `std/math` | **Done** (thin) | e26 `001_abs`; `xo test`; example `math.echo`; www stub |
| `std/encoding/hex` | **Done** (thin) | e26 hex; `xo test` |
| `std/encoding/base64` | **Done** (thin) | e26 `002_base64`; `xo test` |
| `std/str` growth | **Done** (thin) | trim/split/replace/case; suite green; ASCII-first |
| `std/bytes` growth | Thin core | encoding under `std/encoding/*` |
| `std/path` | **Done** (thin) | class/is_uri/is_abs on string or locator; e26 `locator/002_class` + suite |
| `std/json` | **Done** (thin) | e26 parse/stringify; product types |
| `std/random` | **Done** (thin) | seeded e26; **not** CSPRNG |
| `std/log` | **Done** (thin) | e26 level filter; caller-supplied min level |
| `std/os` | **Done** (thin) | e26 `001_pid`; suite |
| `std/process` capture + pipes | **Done** (thin) | `run_capture`/`run_cwd` + `spawn_pipes`; e26 `004_pipes_cat`; suite |
| `std/fs` polish | **Done** (thin) | temp/symlink; suite green after lifecycle fix |
| `std/time` growth | **Done** (thin) | `mono_ms` + `format`/`parse` (chrono); e26 day format |
| `std/net/dns` | **Done** (thin) | e26 localhost |
| `std/net/http_client` | **Done** (thin) | `get`/`request` + TLS helpers; e26 localhost `.run`; suite refuse-port |
| `std/net/url` | **Done** (thin) | e26 parse; runtime product |
| `std/net/unix` | **Done** (thin) | Unix domain; e26 `001_loopback`; crate + suite |
| `std/net/tls` | **Done** (thin) | rustls; platform roots when ca_pem empty; e26 loopback `.run` |
| `std/cli` | **Done** (thin) | pure parse; e26 fixed argv; not GNU getopt |
| `std/crypto` sha256/sha512 + HMAC + AES-GCM + CSPRNG | **Done** (thin) | e26 sha256/hmac; suites |
| `std/bufio` | **Done** (thin) | `lines`; e26 |
| `std/encoding/csv` | **Done** (thin) | e26 split |
| `std/compress/gzip` + `zip` | **Done** (thin) | e26 gzip; zip suite |
| `std/path` clean/rel/walk | **Done** (thin) | e26 clean; walk shallow |
| `std/fs` chmod | **Done** (thin) | Unix mode |
| `std/process` run_cwd | **Done** (thin) | capture + cwd |
| `std/collections/queue` + list helpers | **Done** (thin) | queue methods; `sum_ints`/`sort_ints`; no HOF map/filter/fold |
| `std/regex` / ICU / recursive Walk | **deferred** | regex needs ADR if public |

**Lifecycle note (2026-07):** call arguments must not be `ScopePromote`d as
nested into call results (`nested_owned_names_in_expr`). That bug freed path
strings after `fs.create_dir_all` / other result-returning callees.

### New-module checklist (copy per vertical)

```text
[ ] Classify: pure Echo vs echo_runtime_* (prefer pure Echo; hard solved → crate)
[ ] If Rust: implement via a crate (or std), not a hand-rolled codec/crypto/parser
[ ] Runtime + unit tests (if native)
[ ] RT_* + RUNTIME_EXPORTS + codegen arity/JIT
[ ] Bump RUNTIME_ABI_VERSION / STDLIB_VERSION when surface changes
[ ] std/**.echo + \ exports + co-located xo test
[ ] echo26/run/<domain>/NNN_*.echo + .run (+ reject if must-fail)
[ ] examples/misc when user-runnable
[ ] This inventory + expansive status row
[ ] docs/runtime-abi.md if durable native
[ ] www /docs/std/… page or section
[ ] scripts/gate echo26 (or filter) green
[ ] No userland / runtime; no std/task
```
