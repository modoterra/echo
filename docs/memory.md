# Memory reclamation

How Echo reclaims managed heap. This is the implementer map for the product law
in [ADR 0016](adr/0016-scope-owned-memory.md).

| | |
|--|--|
| **Status** | **Law locked**; **slice 2 landed** (precise promote, demotion, ownership facts, immediate free) |
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

## Pipeline (slice 2)

```text
structured MIR
  → inject_lifetime   # precise promote/demote + leave-scope exits
  → CFG → SSA → …
codegen
  → echo_runtime_scope_*   # registries; immediate free on exit
```

### MIR ops

| Op | Role |
|----|------|
| `ScopeEnter { id }` | Push dynamic ownership frame |
| `ScopeExit { id }` | Pop frame; release every still-owned handle |
| `ScopeRegister { value }` | Record a fresh allocation as owned by the current frame |
| `ScopePromote { value, target }` | Move ownership to an open ancestor **or** nested (demotion) frame |
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
| **Slice 1** | Registries + MIR inject + leave-scope exits (foundation) |
| **Slice 2 (landed)** | Precise promote to bind owner; demotion into once-scopes / loop wraps; `SemanticModel` owning_scope facts; **immediate physical free** on scope exit; enqueue/drain remains for explicit batching |
| **Physical free today** | **Immediate** on `scope_exit` / `scope_release`; `enqueue_release` + `drain_deferred` for short-batch points |
| **Still open (post-slice-2)** | Richer semantic illegal-escape diagnostics; industrial region types (SOTA G9); more precise per-path demotion |
| **Gap framing** | Incomplete reclaim was a **gap**; slice 2 closes the process-lived deferral product gap for values whose ownership ended |

Product model remains **scope-owned dispose** — still **not** tracing GC.

## Slice 2 (landed) — what shipped

| Package | Implementation |
|---------|----------------|
| **2a Precise promote** | `inject_lifetime` tracks bind introduction scopes; reassign / field / list / index stores promote to the **destination bind's owner**, not always root |
| **2b Leave-scope exits** | return ok/err/none, break/continue, if/match/loop arm enter+exit; effect short-circuit via MatchTagged arms |
| **2c Ownership facts v0** | `BindFact.owning_scope` + `introduce_in_scope` / `owning_scope_of` / `is_managed_kind`; pipeline `collect_stmt_facts` walks nested scopes |
| **2d Demotion** | Inward `ScopePromote` only for **unique fresh** owners unused after (alias-safe); once-entered scopes + whole-loop wraps — never demote aliases (`~ b = a`) |
| **2e Immediate free** | `logical_release` physically frees unless `defer_heavy`; `RUNTIME_ABI_VERSION` bumped |
| **2f Proof** | MIR lifetime unit tests; runtime scope free/promote tests; e26 `run/lifetime/001`–`007`; pipeline ownership fact test |

### Pipeline (slice 2)

```text
structured MIR
  → inject_lifetime   # precise promote/demote + leave-scope exits
  → CFG → SSA → …
codegen
  → echo_runtime_scope_*   # registries; immediate free on exit
```

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
