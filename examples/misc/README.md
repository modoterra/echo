# `examples/misc/`

Tiny programs that **`xo run`** today (LLVM AOT + clang + `libecho_runtime`).
Each prints something via `/ std/io` → `io.print`.

```bash
cargo build -p xo
./target/debug/xo run examples/misc/hello.echo
./target/debug/xo run --jit examples/misc/hello.echo   # same runtime, no clang
./target/debug/xo run examples/misc/sum_list.echo
```

| File | Prints | Exit |
|------|--------|------|
| [`hello.echo`](hello.echo) | `42` | 0 |
| [`process.echo`](process.echo) | argv0, env, spawn-fail | 0 |
| [`fs.echo`](fs.echo) | write/read file + dirs under `/tmp` | 0 |
| [`add.echo`](add.echo) | `42` | 0 |
| [`countdown.echo`](countdown.echo) | `1` … `5` | 0 |
| [`break_loop.echo`](break_loop.echo) | `0` `1` `2` then `3` | 0 |
| [`sum_list.echo`](sum_list.echo) | `10` `20` `12` then `42` | 0 |
| [`if_branch.echo`](if_branch.echo) | `10` | 0 |
| [`result_ok.echo`](result_ok.echo) | `7` | 0 |
| [`result_err.echo`](result_err.echo) | `99` | 0 |
| [`strings.echo`](strings.echo) | pure + rich + `\{` `\}` `\xHH` | `hello pure` / `hello` `rich` / `{x}` / `A` |
| [`multi/main.echo`](multi/main.echo) | multi-file `./lib` + std | `multi-file` / `42` / `42` |
| [`const_hash.echo`](const_hash.echo) | `#` const-eval | `42` / `const ok` |
| [`interp.echo`](interp.echo) | rich `{name}` + `==` | `n=7!` / `eq ok` |
| [`point.echo`](point.echo) | struct lit + field R/W | `3` `4` `13` / `{x: 13, y: 4}` |
| [`anon_struct.echo`](anon_struct.echo) | structural `{ k: v }` product | `1` `2` `10` / `0` `3` |
| [`floats.echo`](floats.echo) | f64 arith + `str.from_float` | `4.5` / `6` / `5` |
| [`counter.echo`](counter.echo) | methods + receiver `.` | `1` / `11` / `11` |
| [`at_method/main.echo`](at_method/main.echo) | multi-file `%` + `@` methods | `0` / `1` / `2` / `2` |
| [`match_lit.echo`](match_lit.echo) | ordinary `\|` literal match | `102` |
| [`nested_assign.echo`](nested_assign.echo) | `~ p.nested.y =` chain | `2` / `9` / `10` |
| [`list_assign.echo`](list_assign.echo) | `~ xs[i] =` list mutation | `1` / `9` / `2` / `11` |
| [`width_i32.echo`](width_i32.echo) | `<i32>` / `<i64>` width tags | `30` / `8` / `103` |
| [`width_f32.echo`](width_f32.echo) | `<f32>` / `<f64>` width tags | `3.75` / `1` / `20` |
| [`bytes.echo`](bytes.echo) | `b'…'` / `b"…"` + `str.from_bytes` | `raw` / `esc\t!` / `1` |
| [`bytes_get.echo`](bytes_get.echo) | `std/bytes` `len` / `get` | `3` / `65` / `66` / `out of bounds` |
| [`siphash.echo`](siphash.echo) | `hash.sip` SipHash-2-4 paper vectors | digest ints |
| [`core_surface.echo`](core_surface.echo) | integrated core smoke | `3` / `9` / `ok=core` / … / `raw` |
| [`duration.echo`](duration.echo) | `5s` / `10ms` + add + `str.from_duration` | `5s` / `10ms` / `5010ms` / `eq` |
| [`hex_bin.echo`](hex_bin.echo) | `0x` / `0b` integer lits | `255` / `10` / `18` |
| [`bitwise.echo`](bitwise.echo) | `& \| ^ << >> ~` | `8` / `14` / `6` / `16` / `2` / `-1` |
| [`widths.echo`](widths.echo) | `i*` / `ui*` / `byte` / cast | `255` / `5` / `768` / `3` |
| [`locator.echo`](locator.echo) | `p'…'` / `p"…"` + live `{name}` + `str.from_locator` | paths + `eq` |
| [`struct_defaults.echo`](struct_defaults.echo) | omit fields with shape defaults | `Ada` / `0` |
| [`eq_deep_id.echo`](eq_deep_id.echo) | deep `==` vs identity `===` | `1` / `0` / `1` / … |
| [`multi_bind.echo`](multi_bind.echo) | same-line `~ a = 1, b = 2` | `3` / `30` |
| [`else_if.echo`](else_if.echo) | `?` / `: cond` / `:` chain | `two` |
| [`return_self.echo`](return_self.echo) | `^ .` keeps struct type | `1` |
| [`method_chain.echo`](method_chain.echo) | `c.inc().value()` chains | `1` / `3` |
| [`nested_fn.echo`](nested_fn.echo) | nested closed fn values | `42` |
| [`match_multi.echo`](match_multi.echo) | multi-value match arms | `hit` |
| [`match_type.echo`](match_type.echo) | `% Type` match arms for named structs | `5` |
| [`union_return.echo`](union_return.echo) | fn returns circle\|rect; match refines | `7` |
| [`first_class_fn.echo`](first_class_fn.echo) | pass fn value + call through | `42` |
| [`return_fn.echo`](return_fn.echo) | return a function value | `42` |
| [`field_fn.echo`](field_fn.echo) | fn value on struct field | `42` |
| [`range.echo`](range.echo) | inclusive `lo..hi` for-in + match | `10` / `big` |

## Limits (codegen v1)

Supported roughly: plain/`result`/`option` i64 path, binds, if, loops, list lit +
for-in + index, named struct `%` + tagged lit + structural `{ k: v }` + field
get/set (incl. `~ a.b.c =`, `~ xs[i] =`), methods with receiver `.` / `~ .field`, pure `'…'` /
rich `"…"` (escapes + `{name}` interp), string `==` / `!=` (no `+` concat),
`/ std/io` → `io.print` (**strings only**; use `str.from_int` / `str.from_float`),
multi-file user packages, `#` const-eval, f64/`<f32>` floats, bytes lits
(`b'…'` / `b"…"`, print via `str.from_bytes`), duration lits (`5s`/`10ms`/… as
nanoseconds; print via `str.from_duration`), locator lits (`p'…'` / `p"…"` with live `{name}` interp,
print via `str.from_locator`). Top-level statements are
the program
(no entry keyword).

Not yet: HTTP/`examples/app` full kitchen-sink run, most
of `examples/algos/`.
