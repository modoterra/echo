# Echo benchmarks

User-facing benches live **co-located** in `std/**/*.echo` as `test.bench(...)`.
The harness is `xo test --bench` (see [`docs/testing.md`](../docs/testing.md)).

## Two optimization layers (do not confuse them)

```text
1. HOST (Rust)     just bench-host
                   cargo build --release -p xo
                   + stage libecho_runtime.a next to it
                   → optimized compiler + optimized native runtime (sha256, …)

2. ECHO program    xo test --bench -O2 …
   (LLVM)          optimizes sip.echo / str.echo / … through LLVM
                   links against that libecho_runtime.a
                   first run: compile+link (cache may miss)
                   later runs: IR/AOT hit → mostly exec

3. MEASURE         auto-N loop inside the AOT child only
                   ns/op does NOT include cargo or full pipeline
```

### Recommended flow (sip vs sha256, etc.)

```bash
just bench-host       # (1) once per host/runtime change
just std-bench        # (2)+(3) first pass may compile Echo
just std-bench        # warm: look for "aot cache: hit"
just std-bench-save   # optional local baseline
```

Or one shot: `just std-bench-full` (= bench-host then std-bench).

| Recipe | Role |
|--------|------|
| `just bench-host` | Release `target/release/xo` + `libecho_runtime.a` |
| `just std-bench` | Prebuilt host + Echo `-O2` + cache; **no** cargo rebuild |
| `just std-bench-cold` | `--no-cache` (compile-cost experiments) |
| `just std-bench-compare` | Same host + `-O2` vs `.xo/bench/baseline.jsonl` |

Default host path: `XO=target/release/xo`. Override: `just std-bench XO=target/debug/xo`.

With `--cache-status` (on in just recipes):

```text
codegen cache: hit
aot cache: hit
```

## Record / compare

```bash
just std-bench
just std-bench-save
just std-bench-compare
```

JSONL keys include opt: `std/crypto/hash/sip.echo::sip_empty@O2`.

## Notes

- Host release ≠ Echo `-O2`. You need **both** for fair “native vs pure Echo.”
- Absolute `ns/op` is not portable; compare deltas on the same host + same `-O`.
- `.xo/bench/` and `.xo/cache/` are local.
