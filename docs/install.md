# Installing Echo (`xo`)

User-facing install layout for the **xo** toolchain. Aligns with
[ADR 0014](adr/0014-modules-packages-paths.md): tool-owned trees use **`.xo`**
(or the `xo` XDG application name), not a parallel `echo/…` brand under XDG.

| Script | Role |
|--------|------|
| [`scripts/install.sh`](../scripts/install.sh) | install / upgrade / uninstall / doctor |
| [`scripts/uninstall.sh`](../scripts/uninstall.sh) | thin wrapper → `install.sh uninstall` |

## Quick start (prebuilt — recommended)

Install the **latest GitHub release** for your platform (no Rust toolchain required
for the install itself; `clang` is still needed to **run/build** Echo programs):

```bash
curl -fsSL https://raw.githubusercontent.com/modoterra/echo/main/scripts/install.sh \
  | bash -s -- from-release

# Pin a tag
curl -fsSL https://raw.githubusercontent.com/modoterra/echo/main/scripts/install.sh \
  | bash -s -- from-release v0.0.1-alpha.1

# From a checkout
./scripts/install.sh from-release
./scripts/install.sh doctor
```

Prebuilt platforms (CI release assets `xo-<artifact>.tar.gz`):

| Artifact | Host |
|----------|------|
| `linux-x86_64` | Linux x86_64 |
| `macos-arm64` | Apple Silicon |
| `windows-x86_64` | Windows x86_64 (tarball; use WSL or unpack manually if preferred) |

Each archive contains `bin/xo`, `bin/libecho_runtime.a` (when produced), and
`std/`. Assets are attached when a GitHub release is published (see `docs/ci.md`).

Ensure `~/.local/bin` (or `$XO_BIN_DIR`) is on your `PATH`.

```bash
xo --help
xo home          # package cache root ($XO_HOME)
```

## Quick start (from a checkout)

```bash
# Release build + install under XDG (default when Cargo.toml is present)
./scripts/install.sh

# Or explicitly
./scripts/install.sh install

# Show where things went
./scripts/install.sh doctor
```

Debug build (faster, for toolchain hacking):

```bash
CARGO_PROFILE=debug ./scripts/install.sh install
```

## Layout

### Toolchain (durable data)

| Path | Role |
|------|------|
| `$XDG_DATA_HOME/xo/toolchains/<version>/` | One installed release (`bin/xo`, `std/`, …) |
| `$XDG_DATA_HOME/xo/current` | Symlink → active toolchain |
| `$XO_BIN_DIR/xo` (default `~/.local/bin/xo`) | PATH entry → `current/bin/xo` |

Default `$XDG_DATA_HOME` is `~/.local/share`.

Each toolchain directory contains:

```text
toolchains/<version>/
  bin/xo
  std/…          # co-installed privileged std sources
  version
  installed_at
```

`xo` discovers `std/` via the install prefix: when the binary is
`<prefix>/bin/xo`, the resolver also treats `<prefix>` as a package root if
`<prefix>/std` exists. Optional override: **`$XO_INSTALL_ROOT`**.

### Package cache (user `.xo` — ADR 0014)

| Path | Role |
|------|------|
| **`$XO_HOME`** | User `.xo` root |
| `$XO_HOME/packages/<id>/<version>/` | Fetched packages (`xo get`) |

`$XO_HOME` resolution (same as `xo home`):

1. `$XO_HOME` if set  
2. else `$XDG_CACHE_HOME/.xo`  
3. else `~/.cache/.xo`

**Project** `.xo/cache/` is IR/AOT only — never package downloads.

### State and config

| Path | Role |
|------|------|
| `$XDG_STATE_HOME/xo/` | Durable state (e.g. REPL history) |
| `$XDG_CONFIG_HOME/xo/` | Config (reserved; empty by default) |

Defaults: `~/.local/state/xo`, `~/.config/xo`.

## Upgrade

```bash
# From checkout: rebuild + install new version dir
./scripts/install.sh upgrade

# Prebuilt: re-fetch latest (or tag) release
./scripts/install.sh from-release
# ECHO_RELEASE=v0.0.2 ./scripts/install.sh from-release
```

Upgrade path:

1. Obtain a new `xo` (build from the checkout **or** download a release tarball).
2. Install into a **new** version directory under `toolchains/<version>/`
   (staging dir, then rename).
3. Atomically repoint `current` → the new version.
4. Refresh the PATH symlink.
5. **Keep** previous version directories (rollback: repoint `current` manually).

Force a version directory name:

```bash
ECHO_VERSION=v0.0.2 ./scripts/install.sh upgrade
```

## Uninstall

```bash
# Toolchain + PATH link only; keep package cache and state
./scripts/uninstall.sh
# or
./scripts/install.sh uninstall

# Also remove $XO_HOME (packages), state, and config
./scripts/uninstall.sh --purge
```

Uninstall removes:

- `$XO_BIN_DIR/xo` only if it is a symlink into the data install root  
- `$XDG_DATA_HOME/xo/` (all toolchains + `current` + manifest)

`--purge` also removes `$XO_HOME`, `$XDG_STATE_HOME/xo`, and `$XDG_CONFIG_HOME/xo`.

## Environment summary

| Variable | Meaning |
|----------|---------|
| `XO_HOME` | User package/cache `.xo` root |
| `XO_BIN_DIR` | Directory for the `xo` PATH link |
| `XO_INSTALL_ROOT` | Extra std package root (optional) |
| `ECHO_REPO` | GitHub `owner/name` for prebuilts (default `modoterra/echo`) |
| `ECHO_RELEASE` | Release tag or `latest` for `from-release` |
| `ECHO_VERSION` / `XO_VERSION` | Toolchain version directory name |
| `GITHUB_TOKEN` / `GH_TOKEN` | Optional auth for GitHub API / downloads |
| `CARGO_PROFILE` | `release` (default) or `debug` (checkout builds) |
| `XDG_DATA_HOME` / `XDG_CACHE_HOME` / `XDG_STATE_HOME` / `XDG_CONFIG_HOME` | Standard XDG bases |

## Doctor

```bash
./scripts/install.sh doctor
./scripts/install.sh paths
```

## Relation to development

Day-to-day language work still uses the git workspace:

```bash
cargo build -p xo
./target/debug/xo run examples/misc/hello.echo
```

The install script is for a **user-facing** `xo` on `PATH` with co-located
`std/`, separate from per-project `.xo/cache/`.
