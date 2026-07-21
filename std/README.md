# `std/` — design sources

Public standard library for Echo. **Authority:** [`docs/stdlib.md`](../docs/stdlib.md).

## Rules (locked)

| Rule | |
|------|--|
| Userland | `/ std/…` only — e.g. `io.print`. No free `print`. No `/ runtime`. |
| These files | May `/ runtime` and call `runtime.export(…)` to reach the native runtime. |
| Tools | LSP / check / reflection see **real Echo** (including bodies that call `runtime.*`). |
| Codegen | `runtime.*` → `echo_runtime_*` (only for the privileged runtime package). |

## Intended `io` shape

```echo
/ runtime

$ print = (value) {
    runtime.print(value)
}

$ log = (value) {
    runtime.print(value)
}

$ eprint = (value) {
    runtime.print(value)
}

\ print, log, eprint
```

(Current tree may still be stubs until resolver + codegen implement `/ runtime`.)

## Layout

| Path | Role |
|------|------|
| `io.echo` / `time.echo` / `test.echo` / `bytes.echo` | free function values (`test` for `xo test`; `bytes` for `b'…'` len/get) |
| `collections/map/` | folder module: `% map`, `from_indexed`, … |
| `net/tcp/` | folder module: `conn`, `listener`, free `socket` |
| `net/udp/` | folder module: `% socket` + free reify `socket` |
| `net/request.echo` | `% request` |
| `net/response.echo` | `% response` |
| `net/server.echo` | `% server` |
| `net/http.echo` | `parse_request` + helpers |

No `*_ops` split. Multi-file `@` is still legal language (see `examples/app/`).
