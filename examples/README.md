# `examples/`

Sample Echo programs that exercise the language and toolchain.

| Path | Role |
|------|------|
| [`misc/`](misc/) | **Tiny programs for `xo run`** (codegen v1 demos) |
| [`app/`](app/) | Kitchen-sink surface + multi-file HTTP demo (`xo check`) |
| [`algos/`](algos/) | Classic algorithms (mostly `xo check` until surface expands) |

```bash
cargo build -p xo

# runnable today
./target/debug/xo run examples/misc/hello.echo
./target/debug/xo run examples/misc/multi/main.echo
./target/debug/xo run examples/misc/sum_list.echo ; echo exit:$?

# check (full surface / multi-file)
./target/debug/xo check examples/app/surface.echo
./target/debug/xo check examples/algos/factorial.echo
```

Imports use **module scope**: `/ std/io` → `io.print`, not bare `print`.

See [`misc/README.md`](misc/README.md), [`app/README.md`](app/README.md), [`algos/README.md`](algos/README.md).
