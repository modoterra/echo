# 0016. Scope-owned memory (deterministic disposal)

## Status

Accepted.

## Context

Heap objects today are process-lived handles (`Box::into_raw` with no reclaim).
Classic tracing GC and pure refcounting were considered. Echo prefers **predictable
reclamation tied to language structure**, not a concurrent collector or cycle
detector as the primary story.

## Decision

**Echo assigns every managed allocation to an owning lexical or dynamic scope.**
Semantic lifetime analysis lowers scope transitions into **explicit MIR**
operations for **promotion**, **demotion**, and **release**. **Every control-flow
edge leaving a scope deterministically disposes of the values still owned by that
scope.**

| Term | Meaning |
|------|---------|
| **Owning scope** | Lexical block / function frame / other locked dynamic scope that **owns** a managed allocation |
| **Promotion** | Ownership (or lifetime) moves outward (e.g. return, store into longer-lived object) |
| **Demotion** | Ownership moves into a nested/shorter scope when analysis requires it |
| **Release** | Deterministic dispose of still-owned values on **every** CFG edge that leaves the scope |

This is the product **memory-reclamation law**. It is **not** tracing GC and
**not** pure RC as the user-facing model—though the runtime may use RC-like
internals only where the MIR contract requires shared ownership.

Implementer map (status, pipeline, ABI): [`../memory.md`](../memory.md).

### Layer ownership

| Layer | Responsibility |
|-------|----------------|
| **Semantics** | Lifetime / ownership facts; which allocations bind to which scopes; diagnostics for illegal escapes |
| **MIR** | Explicit promote / demote / release ops on scope transitions; CFG edges must not drop ownership silently |
| **Runtime** | Implement release/promote/demote for heap kinds (ADR 0004) |
| **Codegen** | Emit the MIR ops; no inventing free policy |

### Out of scope for this ADR

- Exact MIR opcodes and ABI for promote/demote/release (land with the vertical).
- Whether shared structure fields use secondary RC, unique ownership only, or
  other runtime techniques—**as long as** the scope-exit dispose law holds.
- Current process-arena behavior remains until the vertical ships; incomplete
  reclaim is a **gap**, not a competing design.

## Consequences

- Long-lived servers do not rely on process exit to reclaim managed heap once
  this vertical is complete.
- Escape analysis for **scalar box elision** (MIR today) stays orthogonal: that
  is ABI packing, not ownership.
- Features that create heap (lists, strings, structs, …) must eventually thread
  ownership through promote/demote/release in the same change style as other
  vertical slices (ADR 0009).
- Design docs and chat must not treat “add Boehm/JVM GC” as the default plan
  without reversing this ADR.
