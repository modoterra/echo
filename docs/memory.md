# Memory reclamation

How Echo reclaims managed heap. This is the implementer map for the product law
in [ADR 0016](adr/0016-scope-owned-memory.md).

| | |
|--|--|
| **Status** | **Law locked**; slice 1 landed; **slice 2 planned** (see § Slice 2 plan) |
| **Owners** | Semantics (lifetime facts) · MIR (`inject_lifetime`) · `echo_runtime` (`scope_*`) · codegen (emit ops only) |
| **Related** | [ADR 0016](adr/0016-scope-owned-memory.md), [`semantics.md`](semantics.md) § Managed heap lifetime, [`mir.md`](mir.md) § Scope ownership ops, [`runtime-abi.md`](runtime-abi.md) § Memory reclamation |

## Garbage collection

**Echo does not use tracing garbage collection** as the user-facing or primary
reclamation model. There is no Boehm-style or JVM-style concurrent collector,
no mark-and-sweep heap walk, and no cycle detector as the product story.

Classic tracing GC and pure reference counting were considered and rejected for
the primary design (ADR 0016). The preference is **predictable reclamation tied
to language structure**: when control leaves a scope, still-owned managed values
are disposed on that edge—deterministically, not when a collector later runs.

| Approach | Role in Echo |
|----------|----------------|
| **Tracing GC** | **Out** as the product model |
| **Pure refcounting** | **Out** as the user-facing model |
| **Scope-owned dispose** | **In** — every managed allocation has an owning scope; leave-scope edges **release** |
| Runtime RC-like internals | Allowed only where the MIR contract requires shared ownership; never the story users reason about |

Do not treat “add Boehm/JVM GC” as the default plan without reversing ADR 0016.

## Product law

**Every managed allocation is assigned an owning lexical or dynamic scope.**

Semantic lifetime analysis lowers scope transitions into explicit MIR operations.
**Every control-flow edge leaving a scope deterministically disposes of values
still owned by that scope.**

| Term | Meaning |
|------|---------|
| **Owning scope** | Lexical block, function frame, or other locked dynamic scope that **owns** a managed allocation |
| **Promotion** | Ownership (or lifetime) moves outward (return, store into a longer-lived object, …) |
| **Demotion** | Ownership moves into a nested/shorter scope when analysis requires it |
| **Release** | Deterministic dispose of still-owned values on **every** CFG edge that leaves the scope |

### Intuition

```text
{                       ← enter scope S
  $ xs = [1, 2, 3]      ← register allocation → owned by S
  …
}                       ← exit S → release still-owned values (including xs if not promoted)
```

If a value must outlive `S` (return, store into outer struct/list, …), analysis
**promotes** ownership to an enclosing scope before exit. Unpromoted owners are
released when their scope ends—including on `return`, `break`, `continue`, and
error-shaped early exits once those edges are fully wired.

## Layer ownership

| Layer | Responsibility |
|-------|----------------|
| **Semantics** | Lifetime / ownership facts; which allocations bind to which scopes; diagnostics for illegal escapes |
| **MIR** | Explicit promote / demote / release ops on scope transitions; CFG edges must not drop ownership silently |
| **Runtime** | Implement release/promote/demote for heap kinds ([ADR 0004](adr/0004-rust-runtime-owns-executable-semantics.md)) |
| **Codegen** | Emit the MIR ops; **no inventing a free policy** |

Escape analysis for **scalar box elision** (MIR today) is **orthogonal**: that is
ABI packing (native vs boxed), not ownership for dispose.

Value-vs-ref pass rules ([`semantics.md`](semantics.md) § Value vs reference) are
also orthogonal: ref types may share storage, but **ownership for dispose** is
scope-based.

## Pipeline (slice 1)

```text
structured MIR
  → inject_lifetime   # ScopeEnter / Exit / Register / Promote / Disown
  → CFG → SSA → …
codegen
  → echo_runtime_scope_*   # registries + logical release; deferred physical free
```

### MIR ops

| Op | Role |
|----|------|
| `ScopeEnter { id }` | Push dynamic ownership frame |
| `ScopeExit { id }` | Pop frame; release every still-owned handle |
| `ScopeRegister { value }` | Record a fresh allocation as owned by the current frame |
| `ScopePromote { value, target }` | Move ownership to an open ancestor frame |
| `ScopeDisown { value }` | Drop ownership without free (e.g. return transfer) |
| `ScopeRelease { value }` | Logical release of one value |

`id` is a per-function compile-time id; the same id may re-enter across nested
calls (each push is a distinct dynamic frame). See [`mir.md`](mir.md).

### Runtime ABI

| Symbol | Role |
|--------|------|
| `echo_runtime_scope_enter` | Push ownership frame |
| `echo_runtime_scope_exit` | Pop frame; release still-owned values |
| `echo_runtime_scope_register` | Own handle in current frame |
| `echo_runtime_scope_promote` | Transfer ownership to target scope |
| `echo_runtime_scope_disown` | Drop ownership without free |
| `echo_runtime_scope_release` | Logical release one value |
| `echo_runtime_scope_enqueue_release` | Logical release + enqueue for deferred free |
| `echo_runtime_scope_drain_deferred` | Physical free of the deferred batch |

Full signatures: [`runtime-abi.md`](runtime-abi.md) § Memory reclamation.

## Status (honest)

| | |
|--|--|
| **Law** | Locked — scope-owned dispose is the product reclamation model |
| **Slice 1 (landed)** | Exact scope registries; MIR inject for function root + if/loop/for-in/match arms; register fresh allocs; promote on nested field/list/index/assign escape; exit on return/break/continue edges |
| **Physical free today** | **Deferred** (logical death first) until promotion analysis is precise enough for safe immediate reclaim |
| **Not yet** | Precise lifetime analysis end-to-end, demotion optimization, full early-exit coverage, immediate/batched physical destroy as the steady state |
| **Gap framing** | Incomplete reclaim is a **gap**, not a competing design. Do not document process-arena / process-exit free as intentional product behavior once the vertical is complete |

Target: full promote/demote/release with deterministic immediate or batched
physical destroy—still **not** tracing GC as the user model.

## Slice 2 plan (ready to implement)

**Goal:** make scope-owned dispose **precise enough** that logical release can
become **immediate or short-batch physical free** without use-after-free, while
keeping the product model (no tracing GC).

### Starting facts (slice 1)

| Layer | Today |
|-------|--------|
| **MIR** | `echo_mir::lifetime::inject_lifetime` — conservative promote-to-root on escape; no demote; no semantic facts |
| **Runtime** | Exact per-scope ownership sets; promote/disown/release; **deferred** physical free (`enqueue` + `drain_deferred`) |
| **Semantics** | Law documented; **no** lifetime `SemanticModel` facts yet |
| **e26** | `echo26/run/lifetime/*` — behavioral smoke (block local, promote assign, if exit, break cleanup), not reclaim proofs |

### Work packages (order)

| # | Package | Done when | Primary crates |
|---|---------|-----------|----------------|
| **2a** | **Precise promote targets** | Escape edges promote to the **true** owning ancestor (not always root); unit tests + e26 | `echo_mir` (lifetime), optional seed facts from semantics |
| **2b** | **Early-exit coverage audit** | Every leave-scope edge emits exits: return/break/continue/effect short-circuit/`!`/`^` paths; miss = MIR/crate test | `echo_mir`, codegen emit |
| **2c** | **Semantic ownership facts (v0)** | `SemanticModel` (or adjacent) records bind → owning scope for managed kinds; illegal escape diagnostics when ready | `echo_semantics`, `echo_pipeline` product |
| **2d** | **Demotion** | Nested scope can take ownership when analysis proves shorter life; MIR `ScopeDemote` (or reuse promote) + runtime | `echo_mir`, `echo_runtime` |
| **2e** | **Immediate / batched physical free** | After 2a–2c are sound: free on exit (or drain at safe points) instead of process-lived deferral; ABI version bump | `echo_runtime`, `echo_codegen_abi`, fingerprint |
| **2f** | **Proof belt** | Crate tests + e26 lifetime fixtures that would **fail** if early free broke aliases; examples unchanged unless demos need it | `echo_mir`, `echo_runtime`, `echo26/run/lifetime` |

### Implementation rules

1. **Still not tracing GC** — ADR 0016 stands.
2. **Conservative until proven** — prefer over-promote over early free; flip free policy only after promote targets are precise.
3. **Shared pipeline only** — no host-local free policy in `xo` / LSP.
4. **Three proofs** — crate tests + e26 + examples as applicable (`AGENTS.md`).
5. **Version bumps** — `RUNTIME_ABI_VERSION` (and docs) when free policy or ABI changes.

### Non-goals for slice 2

- Full industrial region/type system (SOTA G9 remainder)
- Cycle collection or concurrent collector
- User-visible "manual free" API
- Changing value-vs-ref pass rules

### Suggested first commit of the slice

**2a alone:** replace "promote managed escapes to root" with promote to the
**nearest scope that outlives the use** (function root when binding escapes the
function; parent block when binding is outer-block only). Keep deferred free.
Prove with MIR unit tests + extend `echo26/run/lifetime` if observable.

### Checklist before coding 2a

- [ ] Read `crates/echo_mir/src/lifetime.rs` + `crates/echo_runtime/src/scope.rs`
- [ ] Read e26 `run/lifetime/*` and existing MIR lifetime unit tests
- [ ] Sketch promote-target table for: local bind, field store, list push, return, task capture (if any)
- [ ] Land tests first (red) for one wrong promote-to-root case, then green

## Features that allocate

Anything that creates managed heap (lists, strings, structs, bytes, …) must
eventually thread ownership through promote/demote/release in the same vertical
style as other language features ([ADR 0009](adr/0009-full-vertical-slices.md)).
New heap kinds without scope registration are incomplete work.

## Where facts live

| Kind of fact | Location |
|--------------|----------|
| Sticky decision | [ADR 0016](adr/0016-scope-owned-memory.md) |
| Language-facing lifetime law | [`semantics.md`](semantics.md) § Managed heap lifetime |
| MIR ops and inject pass | [`mir.md`](mir.md) |
| Runtime symbols | [`runtime-abi.md`](runtime-abi.md) |
| Vocabulary | [`glossary.md`](glossary.md) — *scope-owned memory*, *promotion / demotion / release* |
| Design/impl checklist row | [`roadmap.md`](roadmap.md) § Memory reclamation |
| This map | `docs/memory.md` |
