set shell := ["bash", "-cu"]

check:
    cargo check --workspace

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
