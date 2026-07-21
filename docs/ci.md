# Continuous integration

| | |
|--|--|
| **Status** | Active |
| **Workflow** | [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) |
| **Related** | [`development-speed.md`](development-speed.md), [`llvm.md`](llvm.md), [`fixtures.md`](fixtures.md) |

## Triggers

Multi-platform **build** CI runs **only** when a GitHub **Release is published**
(`release: types: [published]`). It does not run on push, PR, bare tags, or
manual dispatch.

Local day-to-day gate remains `scripts/gate echo26` / `cargo test` on the developer machine.

## Jobs

### `build` (matrix)

Release-builds `xo` (always includes LSP + REPL — no Cargo features) and uploads:

| Artifact name | Runner | Notes |
|---------------|--------|-------|
| `xo-linux-x86_64` | `ubuntu-24.04` | binary + `xo-linux-x86_64.tar.gz` |
| `xo-windows-x86_64` | `windows-2022` | `xo.exe` + tarball when produced |
| `xo-macos-arm64` | `macos-14` | Apple Silicon only (no Intel mac) |

Each matrix job also **attaches** `xo-<artifact>.tar.gz` to the published
GitHub release (`gh release upload`). Archive layout:

```text
bin/xo                  # or xo.exe on Windows
bin/libecho_runtime.a   # when the staticlib is produced
std/…                   # co-located std sources
version                 # release tag
```

Users install with `scripts/install.sh from-release` (see
[`docs/install.md`](install.md)).

Each job installs **LLVM 22** from **official**
[`llvm/llvm-project` release tarballs](https://github.com/llvm/llvm-project/releases)
via in-repo [`scripts/ci/llvm.sh`](../scripts/ci/llvm.sh)
(SHA256-pinned). Sets `LLVM_SYS_221_PREFIX`. Clears the repo `sccache` rustc
wrapper so runners without sccache still compile.

**Host-specific LLVM env**

| OS | What CI sets | What CI does **not** set |
|----|----------------|---------------------------|
| Linux | `LIBRARY_PATH`, `LD_LIBRARY_PATH` → LLVM `lib/` | — |
| macOS | `LIBRARY_PATH` → LLVM `lib/` (link-time only) | **`DYLD_LIBRARY_PATH`** — LLVM’s `libc++` must not shadow the system one or `rustc` aborts |
| Windows | prefix path via `cygpath` for Git Bash `tar` | raw `D:\…` extract dirs (break `tar`) |

After `cargo build`, [`scripts/ci/stage-runtime-lib.sh`](../scripts/ci/stage-runtime-lib.sh)
copies the newest `libecho_runtime*.a` into `target/<profile>/libecho_runtime.a`
so `xo run` AOT link always finds it.

No third-party `setup-llvm` action — keeps the CI supply chain to GitHub-hosted
actions (`checkout`, `upload-artifact`), `dtolnay/rust-toolchain`, and LLVM
upstream only.

### `test-linux` (hard gate)

On Ubuntu only:

1. Debug build of `xo` + `e26`
2. Smoke: `cargo test -p xo`
3. Smoke: `xo run --no-cache examples/misc/hello.echo` (needs clang + runtime)
4. **`scripts/gate echo26`** — workflow fails if red

Language correctness is enforced here; Windows/macOS jobs only prove the binary
links for that host.

## Host / known gaps

- Windows **AOT** (`xo run` via clang + pthread/dl flags) is not yet first-class;
  shipping `xo.exe` for check/fmt/lsp/repl is still the goal of the Windows build.
- Dynamic LLVM (`prefer-dynamic`) may need `PATH` / DLL layout on Windows; CI
  puts the LLVM bin dir on `PATH`.
