# Echo benchmarks

User-facing benches live **co-located** in `std/**/*.echo` as `test.bench(...)`.
The harness is `xo test --bench` (see [`docs/testing.md`](../docs/testing.md)).

## Record a run

```bash
just std-bench              # -O2 by default; streams JSONL → .xo/bench/last.jsonl
just std-bench O=0          # unoptimized (debug path)
```

Or explicitly:

```bash
xo test --bench -O2 std --bench-out .xo/bench/last.jsonl
xo test --bench -O3 std/math.echo
xo test --bench --opt-level 2 std   # long form
```

LLVM levels match `xo run`: `0` / `1` / `2` / `3` / `z` (`Oz`).

## Compare to a baseline

```bash
# once, after a “good” run on this machine / opt recipe:
just std-bench-save   # copies last.jsonl → baseline.jsonl

# later (same -O):
just std-bench-compare   # fails if any bench is >20% slower (ns/op)
```

Compare keys are `file::name@opt` (e.g. `std/math.echo::abs_i@O2`).

## Notes

- `just std-bench` uses **O2**; plain `xo test --bench` defaults to **O0**.
- Keep opt level fixed when comparing; it is stored in each JSONL row.
- `.xo/bench/` is local cache-style output (not package content).
- Absolute `ns/op` is not portable across machines; use **relative** deltas.
