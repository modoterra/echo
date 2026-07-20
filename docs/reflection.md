# Reflection

Callable and reflection metadata for tools (LSP, runtime inspect, docs).

| | |
|--|--|
| **Status** | **Stub** (`echo_reflection` crate name only) |
| **Owners** | `echo_reflection` |
| **Related** | [`stdlib.md`](stdlib.md), [`pipeline.md`](pipeline.md), [`lsp.md`](lsp.md) |

## Scope

Stable descriptions of exports, signatures, and shapes that **mirror** what
check/resolve already know — never a second type system.

## Facts

- Crate exists for workspace linkage; no public API yet beyond `crate_name()`.
- When implemented: feed from index + resolver + semantics (same pipeline).

## Non-goals

- PHP-style reflection APIs
- Parallel “native-only” std tables invisible to check

## Open questions

- What is exposed to userland vs tools-only
