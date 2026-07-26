# Testing (`xo test` + `std/test`)

| | |
|--|--|
| **Status** | **v0** (Model A registration + benches) |
| **Related** | `docs/stdlib.md`, `docs/runtime-abi.md`, `docs/pipeline.md`, `docs/fixtures.md` |

## Three layers

| Layer | Role |
|-------|------|
| Crate tests | Rust unit tests for compiler/runtime |
| **e26 / echo26** | Toolchain conformance (Echo 2026 contract) |
| **`xo test`** | **User / std suites** written in Echo |

They do not replace each other.

## Model A (locked for v0)

1. Suite files call **`std/test`** helpers at top level (e.g. `test.it`, `test.bench`).
2. Those helpers register cases via **`runtime.test_register`** /
   **`runtime.test_bench_register`** (std only).
3. Registration is active only when the host sets **`XO_TEST`** (done by `xo test`).
4. After the entry top-level body, **`runtime.test_finish`** runs the selected
   kind of cases and becomes the process exit status (failure count). Under
   `xo run`, finish returns “suite off” so normal programs are unchanged.
5. **`xo test`** runs only `test.it` cases. **`xo test --bench`** sets
   **`XO_BENCH`** and runs only `test.bench` cases (harness-looped auto-N).

## `std/test`

```echo
/ std/test

test.it("name", () {
    test.eq(1, 1)
    test.true(|)
})

test.bench("hot_path", () {
    $ n = abs_i(-3)
    test.eq(n, 3)
})
```

Bodies may also be named binds (`$ case = () { … }; test.it("n", case)`).
| Export | Role |
|--------|------|
| `it(name, body)` | Register a zero-arg function value as a test case |
| `bench(name, body)` | Register a zero-arg body; harness calls it N times |
| `eq` / `ne` | Assert deep equality |
| `true` / `false` | Assert bool |
| `fail(msg)` | Mark current case failed |

Custom libraries may wrap these; only privileged std talks to `/ runtime`.

### Benchmarks

- Body is **zero-arg**; the runtime scales **N** until one measured run lasts
  about **1s** (capped), then reports `N` and **ns/op**.
- Assertions (`eq` / `fail`, …) still work inside a bench body; a failed assert
  fails the bench.
- Co-located next to production or tests (same file as `test.it` is fine).
- Registration is a no-op without `XO_TEST`; benches run only with `XO_BENCH`.
- Prefer **no asserts in the hot body** so `ns/op` measures the work, not `test.eq`.

### Std + algorithm benches

Benches call **real** functions (same bodies as demos / std), with args built
in the bench body. No separate “canary wrapper” module.

| Location | Examples |
|----------|----------|
| `std/list` | `sum_ints_1k`, `sort_ints_1k` |
| `std/bytes` | `checksum_1k`, `checksum_64k` |
| `std/crypto/hash/sip` | `sip_empty` … `sip_64k` |
| `std/crypto/hash/sha256` | `sha256_empty`, `sha256_1k` |
| `std/collections/map` | `seed_1k_get` |
| `examples/algos/*` | `fib`, `gcd`, `sum_to`, sorts, primes, call overhead |

```bash
just bench-host
just std-bench          # all std co-located benches
just algo-bench         # algorithms + selected std hot paths
xo test --bench -O2 std --bench-out .xo/bench/last.jsonl
xo test --bench -O2 std --bench-out .xo/bench/last.jsonl \
  --bench-baseline .xo/bench/baseline.jsonl --bench-threshold 20
```

Example output:

```text
bench abs_i  N=25000000  40ns/op  (1000000000ns)
xo test --bench: 1 passed, 0 failed, 1 total (…)
```

### Recording and comparing results

While benches run, each finished case can be **appended as JSONL** (one object
per line) when `--bench-out PATH` is set. The file is truncated at the start of
the run, then grown as cases complete (safe to `tail -f`).

| Flag | Role |
|------|------|
| `-O` / `--opt-level LEVEL` | LLVM opt for suite compile: `0`/`1`/`2`/`3`/`z` (same as `xo run`). Default `0`. Prefer **`-O2`** for benches. |
| `--bench-out PATH` | Stream JSONL results (requires `--bench`) |
| `--bench-baseline PATH` | Prior JSONL to compare after the run (needs `--bench-out`) |
| `--bench-threshold PCT` | Exit 1 if any case is **worse** than baseline by more than `PCT`% `ns/op` |

JSONL fields: `v`, `file`, `name`, `opt` (`O0`…`Oz`), `status` (`ok`/`fail`),
and on success `n`, `ns_per_op`, `total_ns`.

Compare keys are `file::name@opt` so O0 and O2 baselines do not mix.

Compare lines look like:

```text
  REG  std/math.echo::abs_i@O2  1000 → 1300 ns/op  (+30.0%)
  IMP  std/str.echo::cat@O2     2000 → 1800 ns/op  (-10.0%)
  NEW  std/list.echo::sum@O2    500ns/op
summary: 1 regression(s), 1 improvement(s), 1 new, 0 gone
```

Save a local baseline after a good run:

```bash
xo test --bench -O2 std --bench-out .xo/bench/last.jsonl
cp .xo/bench/last.jsonl .xo/bench/baseline.jsonl
# later:
xo test --bench -O2 std --bench-out .xo/bench/last.jsonl \
  --bench-baseline .xo/bench/baseline.jsonl --bench-threshold 20
```

Keep the **opt level fixed** when comparing. Absolute `ns/op` is not portable
across machines; use relative deltas on the same recipe.

### Prebuilt host + warm suite cache

- **Host:** run a prebuilt `xo` (`target/debug/xo` or release). Do not fold
  `cargo build -p xo` into every bench iteration — that measures the Rust
  toolchain, not Echo.
- **Suite files:** each `.echo` is still compiled to an AOT child, but **IR and
  AOT artifact caches** (under `.xo/cache/`) make the second run mostly load +
  exec. Use `--cache-status` to confirm `hit`; use `--no-cache` only for cold
  pipeline experiments.
- **`just std-bench`** uses a prebuilt `XO` binary and passes `--cache-status`.
## `xo test` CLI

```bash
xo test                          # ./ *_test.echo and tests/** (+ std/ co-located suites)
xo test path/to/file.echo
xo test path/to/dir
xo test '**/*_test.echo'
xo test a_test.echo tests/
xo test std                      # co-located std suites (`test.it`)
xo test --bench                  # only files with test.bench (default discovery)
xo test --bench std              # all co-located std benchmarks
xo test --bench std/math.echo
```

- Each matched **file** is compiled and executed as a suite entry with `XO_TEST=1`.
- With **`--bench`**, also `XO_BENCH=1` — only benchmarks run; `test.it` is skipped.
  Discovery skips files that do not contain `test.bench(`.
- Without **`--bench`**, only `test.it` runs; registered benches are ignored.
- Paths: files, directories (walk), or simple globs (`*`, `**`, `?`).
- Directory convention:
  - `*_test.echo` anywhere
  - all `.echo` under `tests/`
  - under a `std/` path: co-located suites that call `test.it(` / `test.bench(`

## Co-located vs separate files

Either is allowed:

- **Separate** `map_test.echo` next to production (recommended for libs)
- **Same file** with top-level `test.it(...)` / `test.bench(...)` — cases run
  only under `xo test` / `xo test --bench`, not under `xo run` (registration
  no-op without `XO_TEST`)

Production APIs still use normal `\ ` exports for importers.

## Not for

- Replacing **e26** goldens
- Magic test keywords
- Userland `/ runtime` test hooks
- Fuzzing, allocation counters, or statistical (Criterion-style) benches (later)
