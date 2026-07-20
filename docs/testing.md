# Testing (`xo test` + `std/test`)

| | |
|--|--|
| **Status** | **v0** (Model A registration) |
| **Related** | `docs/stdlib.md`, `docs/runtime-abi.md`, `docs/pipeline.md`, `docs/fixtures.md` |

## Three layers

| Layer | Role |
|-------|------|
| Crate tests | Rust unit tests for compiler/runtime |
| **e26 / echo26** | Toolchain conformance (Echo 2026 contract) |
| **`xo test`** | **User / std suites** written in Echo |

They do not replace each other.

## Model A (locked for v0)

1. Suite files call **`std/test`** helpers at top level (e.g. `test.it`).
2. Those helpers register cases via **`runtime.test_register`** (std only).
3. Registration is active only when the host sets **`XO_TEST`** (done by `xo test`).
4. After the entry top-level body, **`runtime.test_finish`** runs registered cases
   and becomes the process exit status (failure count). Under `xo run`, finish
   returns “suite off” so normal programs are unchanged.

## `std/test`

```echo
/ std/test

test.it("name", () {
    test.eq(1, 1)
    test.true(|)
})
```

Bodies may also be named binds (`$ case = () { … }; test.it("n", case)`).
| Export | Role |
|--------|------|
| `it(name, body)` | Register a zero-arg function value as a case |
| `eq` / `ne` | Assert deep equality |
| `true` / `false` | Assert bool |
| `fail(msg)` | Mark current case failed |

Custom libraries may wrap these; only privileged std talks to `/ runtime`.

## `xo test` CLI

```bash
xo test                          # ./ *_test.echo and tests/**
xo test path/to/file.echo
xo test path/to/dir
xo test '**/*_test.echo'
xo test a_test.echo tests/
```

- Each matched **file** is compiled and executed as a suite entry with `XO_TEST=1`.
- Paths: files, directories (walk), or simple globs (`*`, `**`, `?`).
- Directory convention: `*_test.echo` anywhere, and all `.echo` under `tests/`.

## Co-located vs separate files

Either is allowed:

- **Separate** `map_test.echo` next to production (recommended for libs)
- **Same file** with top-level `test.it(...)` — cases run only under `xo test`,
  not under `xo run` (registration no-op without `XO_TEST`)

Production APIs still use normal `\ ` exports for importers.

## Not for

- Replacing **e26** goldens
- Magic test keywords
- Userland `/ runtime` test hooks
