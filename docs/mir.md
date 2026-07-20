# MIR

Mid-level, backend-neutral executable intermediate representation.

| | |
|--|--|
| **Status** | **Active** (codegen consumer; expands with execute verticals) |
| **Owners** | `echo_mir` |
| **Related** | `docs/hir.md`, `docs/llvm.md`, `docs/runtime-abi.md`, ADR 0002, `docs/incremental.md`, `docs/semantics.md` § Value vs reference |

## Value class (language pass semantics)

Distinct from storage [`MirRepr`](../crates/echo_mir/src/repr.rs). API:
`echo_mir::ValueClass` (`value_class.rs`).

| Class | Kinds | Pass |
|-------|--------|------|
| **RefValue** | struct, list | copy reference (share) |
| **StaticValue** | int/float/bool/string/bytes/… | copy value |

`ValueClass::from_value_kind` / `from_mir_repr` / `from_hir_expr` classify for
tests and future lower seeds. ABI handle packing for strings is still
StaticValue in the language model.

## Ironclad goal

**MIR exposes language semantics and native representations so codegen can emit
hyper-optimizable LLVM IR.** Residual **generic** machine-independent
optimization is **LLVM’s** job (`default<On>`).

| Echo MIR owns | LLVM owns |
|---------------|-----------|
| CFG + SSA form | Sparse constant propagation |
| Representation analysis (native vs boxed) | GVN / CSE |
| Explicit box/unbox boundaries | LICM |
| Local simplify (redundant box/unbox, trivial copies) | Induction / loop opts |
| **Escape analysis** + **`NoEscape` scalar box elision** (runtime ABI) | General DCE and mid-end |
| Language-shaped lowering (list get → runtime, …) | `default<O1>`…`default<Oz>` |

**Do not** re-add MIR passes that only reimplement LLVM (constprop, GVN, LICM,
IV, BCE, …) unless IR/benchmarks prove an Echo/ABI barrier LLVM cannot see.
Escape / box elision is intentional MIR work: it needs Echo runtime
non-retaining call knowledge LLVM does not have.

## Pipeline (authority)

```text
structured MIR
  → CFG
  → SSA
  → analyze_reprs
  → simplify_local
  → analyze_escapes (NoEscape box elision)
  → simplify_local
  → LLVM (verify ± default<On>)
```

### Intentional residual MIR passes

| Pass | Why it stays in MIR |
|------|---------------------|
| `analyze_reprs` | Echo native vs boxed form for hyper-optimizable LLVM handoff |
| `simplify_local` | Cancel redundant box/unbox and trivial copies at representation boundaries |
| `analyze_escapes` | Classify escape with Echo ABI (e.g. non-retaining `runtime.print`); elide `NoEscape` scalar boxes |

## Scope

Control flow, values, calls, and other executable structure prior to LLVM.
Targets LLVM only (ADR 0002) but does not embed LLVM types in the IR design.

## Facts (current)

- Entry: `lower_program(entry_path, modules) -> LoweredProgram`.
- Code objects: closed bodies from HIR `bodies` (symbol = linkage id) + method
  symbols + optional exported value getters (`__val_*`) + entry `__toplevel`.
- Function values: `FnValue` (code pointer); `CallTarget::Indirect` for call
  through a value. Direct `CallTarget::Function` when the bind resolves to a
  known body symbol.
- Each `MirFn` carries SSA `cfg`, `reprs`, and `escapes` (local escape classes).
- **Return shapes:** plain / result / option from syntax (`^` / `!`), not user
  type names (`MirRetShape`).
- **Calls:** free, module export, `runtime.*` → `CallTarget::Runtime`, methods
  via type env (struct lit → local type → method table).
- **Heap values (handles as i64):** list lit/index, string lit/interp, struct
  lit / field get / field set.
- **Control:** if, while/infinite/for-in, break/continue, tagged match
  (result/option), **value match** (if/`==`/`||` chain for multi-expr arms;
  bool arms `|` / `_` at arm start).
- Const `#` folding into `const_env` for getters and foldable exprs (lower-time,
  not a mid-end pass).
- **List index:** CFG emits `ListGetChecked` → `echo_runtime_list_get` (soft OOB
  in runtime). No MIR bounds-check elimination (LLVM / runtime own that class).
- **Width tags:** `MirRepr::Int32` / `Float32` (plus default `Int64` / `Float64`)
  for `<i32>` / `<f32>` lits and same-width native arith; box edges widen to the
  heap ABI (`i64` / heap float).
- **Bytes lits:** `MirExpr::BytesLit` → `MirRepr::BytesRef` (heap handle, parallel
  to `StringRef` but a distinct runtime kind).
- **Duration lits:** `MirExpr::ConstDuration(nanos)` → `MirRepr::Duration` (i64
  nanoseconds; `+`/`-` with other durations).
- **Locator lits:** `MirExpr::LocatorLit` → `MirRepr::LocatorRef` (heap handle,
  path/URI text; parallel to `BytesRef` / `StringRef`).

## Schema / cache

Bump `MIR_SCHEMA_VERSION` / `MIR_LOWERER_VERSION` in `echo_fingerprint` when MIR
shape or lowering meaning changes ([`incremental.md`](incremental.md)).

## Multi-file methods (`%` / `@`)

- Each module’s HIR records methods defined in **that file**.
- `lower_program` unions them into a graph-wide table:
  `struct → method → (defining_module, mangled_name)`.
- `MethodCall` uses the **defining** module path in `CallTarget::Function`
  (not the call-site module). Duplicate members are rejected earlier by resolve
  merge (`res-struct-dup-member`).

## Structural products (`{}`)

Anonymous `{ k: v, … }` lowers as HIR/MIR struct lit with empty type name
(`echo_runtime_struct_new`). Named tagged lits use `echo_runtime_struct_new_named`
so the heap carries a type tag for `|` `% Type` arms. Field get/set use string
field keys. Not a map; no methods table.

## Match

- **Value / range / `% type`:** lower to if/else (`==` / range membership /
  `MirExpr::StructTypeIs` → `echo_runtime_struct_type_is`).
- **Option/Result:** `MatchTagged` terminator (tag unpack + bind payload).

## Named-struct unions

- HIR `returns_structs: Vec` — empty / one / many from valued `^` named lits.
- MIR seeds monomorphic `__fnret_*` only when `len == 1`.
- `% Type` match arms **refine** a name scrutinee in `type_env` for the arm body.
