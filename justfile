set shell := ["bash", "-cu"]

check:
    cargo check --workspace

# Workspace check with warnings as errors (matches pre-commit).
check-deny:
    #!/usr/bin/env bash
    set -euo pipefail
    extra=(-Dwarnings)
    if [[ "$(uname -s)" == "Linux" ]] && command -v mold >/dev/null 2>&1; then
      extra+=(-C "link-arg=-fuse-ld=mold")
    fi
    export RUSTFLAGS="${RUSTFLAGS:+${RUSTFLAGS} }${extra[*]}"
    cargo check --workspace --locked

# Point this clone at versioned .githooks/ (pre-commit rustc clean).
hooks:
    scripts/install-hooks.sh

test CRATE:
    cargo test -p {{CRATE}}
test-fast:
    scripts/gate changed

test-full:
    scripts/gate workspace

# echo26 suite against the workspace xo binary
e26:
    cargo build -q -p xo -p e26
    cargo run -q -p e26 -- --binary target/debug/xo

# Co-located std unit suites
std-test:
    cargo build -q -p xo
    ./target/debug/xo test std

# Co-located std benchmarks (auto-N / ns/op; ~1s each)
std-bench:
    cargo build -q -p xo
    mkdir -p .xo/bench
    ./target/debug/xo test --bench std --bench-out .xo/bench/last.jsonl

# Re-run std benches and compare to .xo/bench/baseline.jsonl (20% worse → fail)
std-bench-compare:
    cargo build -q -p xo
    mkdir -p .xo/bench
    ./target/debug/xo test --bench std \
      --bench-out .xo/bench/last.jsonl \
      --bench-baseline .xo/bench/baseline.jsonl \
      --bench-threshold 20

# Promote last run to local baseline
std-bench-save:
    mkdir -p .xo/bench
    cp -f .xo/bench/last.jsonl .xo/bench/baseline.jsonl
    @echo "saved .xo/bench/baseline.jsonl"

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt-check

profile:
    time cargo check --workspace
    time cargo test --workspace --no-run
    time cargo nextest run --workspace

sccache:
    sccache --show-stats

tools:
    scripts/gate tools

# User XDG install (release xo + std → ~/.local/bin/xo)
install:
    scripts/install.sh install

upgrade:
    scripts/install.sh upgrade

uninstall *ARGS:
    scripts/install.sh uninstall {{ARGS}}

install-doctor:
    scripts/install.sh doctor

gate *ARGS:
    scripts/gate {{ARGS}}

web-install:
    npm --prefix www install

web-dev:
    npm --prefix www run dev

web-lint:
    npm --prefix www run lint

web-format:
    npm --prefix www run format

web-build:
    npm --prefix www run build
