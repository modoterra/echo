# Echo benchmarks

User-facing benches live **co-located** in `std/**/*.echo` as `test.bench(...)`.
The harness is `xo test --bench` (see [`docs/testing.md`](../docs/testing.md)).

## Prebuilt `xo` + warm cache (important)

**ns/op only times the bench body** inside the AOT child. You still want the
**host** and **suite artifacts** warm so wall-clock is not dominated by
rebuilding the compiler or re-running the full pipeline:

| Piece | What to do |
|-------|------------|
| Host `xo` | Build once: `just xo-rebuild` or `cargo build -p xo` / release. Recipes use that binary; they do **not** `cargo build` every time. |
| Suite IR + AOT | First `just std-bench` may **miss** caches; later runs should show **hit** (see `--cache-status`). |
| Cold pipeline | `just std-bench-cold` (`--no-cache`) — for compile-cost experiments, not microbench deltas. |

```bash
cargo build -p xo                 # once (or just xo-rebuild)
just std-bench                    # may compile suite files on first run
just std-bench                    # warm: IR/AOT hit → mostly exec + measure
just std-bench XO=target/release/xo
```

With `--cache-status` you should see per file something like:

```text
codegen cache: hit
aot cache: hit
```

If you always see `miss`, the cache is not sticky (wrong project root, ABI
version bump, or `--no-cache`).

## Record a run

```bash
just std-bench              # -O2; streams JSONL → .xo/bench/last.jsonl
just std-bench O=0          # unoptimized
```

Or explicitly:

```bash
./target/debug/xo test --bench -O2 std \
  --bench-out .xo/bench/last.jsonl --cache-status
```

LLVM levels match `xo run`: `0` / `1` / `2` / `3` / `z` (`Oz`).

## Compare to a baseline

```bash
just std-bench-save      # last.jsonl → baseline.jsonl
just std-bench-compare   # same -O2; fail if >20% slower
```

Compare keys: `file::name@opt` (e.g. `std/math.echo::abs_i@O2`).

## Notes

- `just std-bench` uses **O2** and a **prebuilt** `xo` (env `XO`, default `target/debug/xo`).
- Keep opt level fixed when comparing; it is stored in each JSONL row.
- Absolute `ns/op` is not portable; use **relative** deltas on the same recipe.
- `.xo/bench/` and `.xo/cache/` are local; not package content.
