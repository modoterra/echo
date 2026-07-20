# Standard library and runtime package

| | |
|--|--|
| **Status** | **Locked** (resolution + `/ runtime` in std only + pipeline ownership) |
| **Owners** | See [pipeline ownership](#pipeline-ownership-locked) |
| **Related** | `docs/modules.md`, `docs/runtime-abi.md`, `docs/pipeline.md`, ADR 0001, ADR 0004 |

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
| Root | (1) **System/install** Echo std, (2) **workspace `std/`** for toolchain dev |
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

Suite helpers for `xo test` (see [`testing.md`](testing.md)). Exports `it`, `eq`,
`ne`, `true`, `false`, `fail`. Bridges to `runtime.test_*` (suite mode via env
`XO_TEST`).

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
| **Resolve** | `echo_resolver` | (1) `/ std/…` → toolchain std root; (2) `/ runtime` only if **importer file** is under that std root; (3) bind module name `runtime`; (4) closed graph includes runtime unit as a **synthetic/virtual module** with known exports |
| **Semantics** | `echo_semantics` | Treat `runtime.export` as a normal module export when import is legal; arity/use checks; **never** inject free `print` into user scopes |
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
  time.echo
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
```

No `*_ops` in std for now. Multi-file `@` remains legal language (demo:
`examples/app/`).

## Style

- **struct names** lowercase `snake_case`
- Members `$` / `~` / `#` (data or function values)
- Free functions `$ name = (args) { … }` and/or methods on `%` types
- Module-scoped imports only (`docs/modules.md`)

## Implementation status

| Piece | Status |
|-------|--------|
| Design / docs | **Locked** (this file) |
| Privileged std root in resolver | Workspace package roots (install root later) |
| `/ runtime` gate + virtual package | **Done** (`res-runtime-forbidden`; `<echo:runtime>`) |
| `runtime.*` → `echo_runtime_*` in codegen | **Done** (`runtime.print` → `echo_runtime_print_i64`) |
| Runtime free-only (no methods on runtime) | **Locked** |
| Std named wrappers for net | **Done** — `% conn` / `% listener` methods; factories return named types |
| `runtime.http_parse_request` | **Done** — **httparse**; method/path/body + headers product |
| `runtime.tcp_*` / `runtime.udp_*` | **Done** — real OS sockets in `echo_runtime` net |
| `std/net/tcp/` | **Done** — folder: `conn` + `listener` shapes + `socket` free surface |
| `std/net/udp/` | **Done** — folder: `% socket` + free reify surface (struct by ref) |
| `std/net/http` serve / handle_connection | **Done** — accept loop + `+ handle_connection`; **Content-Length body** via `http_request_complete` |
| `std/str.len` | **Done** — `runtime.str_len` |
| `std/str.cat` | **Done** — `runtime.str_cat` |
| Method type after free-fn return (named struct) | **Done** — lit + call-chain + monomorphic `returns_structs` |
| Method type after union return | **Done** — refine via `%` match |
| Method on raw runtime product/handle | **Out** — wrap in `%` in std instead |
| Bare userland `print` | **Not** intrinsic (unbound / unknown fn) |
| `std/io.echo` body using `/ runtime` | **Done** |
