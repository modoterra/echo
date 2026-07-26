# Echo benchmarks

User-facing benches live **co-located** in `std/**/*.echo` as `test.bench(...)`.
The harness is `xo test --bench` (see [`docs/testing.md`](../docs/testing.md)).

## Record a run

```bash
just std-bench
# → streams JSONL to .xo/bench/last.jsonl while cases finish
```

Or explicitly:

```bash
xo test --bench std --bench-out .xo/bench/last.jsonl
```

## Compare to a baseline

```bash
# once, after a “good” run on this machine / opt recipe:
just std-bench-save   # copies last.jsonl → baseline.jsonl

# later:
just std-bench-compare   # fails if any bench is >20% slower (ns/op)
```

Compare keys are `file::name` (e.g. `std/math.echo::abs_i`).

## Notes

- Default suite opt is **O0** AOT; keep the recipe fixed when comparing.
- `.xo/bench/` is local cache-style output (not package content).
- Absolute `ns/op` is not portable across machines; use **relative** deltas.
