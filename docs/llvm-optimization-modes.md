# LLVM Optimization Modes

## Goal

Echo should emit correct, simple, readable LLVM IR first, then optionally run LLVM's standard optimization pipelines for release-oriented builds.

Optimization should be a compiler mode, not ad-hoc codegen rewrites. Debug/default builds should preserve readable IR and straightforward diagnostics. Optimized builds should verify generated IR, run an LLVM pipeline, and compile or emit the optimized result.

## Build Modes

Echo should support these optimization levels:

- `O0`: no optimization; current/default behavior.
- `O1`: light LLVM optimization.
- `O2`: normal release LLVM optimization.
- `O3`: aggressive LLVM optimization.
- `Oz`: size-focused LLVM optimization.

Expected CLI shape:

```bash
xo build file.echo
xo build -O0 file.echo
xo build -O1 file.echo
xo build -O2 file.echo
xo build -O3 file.echo
xo build -Oz file.echo
```

IR inspection should also support optimized output:

```bash
xo build file.echo --emit-ir
xo build file.echo --emit-ir -O2
```

The exact flags can follow `xo`'s CLI conventions, but `O0` should remain the default until there is an explicit release mode.

## Pipeline

The target compiler pipeline is:

```text
parse PHP/Echo source
lower AST to straightforward LLVM IR
verify LLVM module
if optimization level != O0, run LLVM default optimization pipeline
emit IR or link native binary from the selected module
```

Preferred LLVM pipeline mapping:

```text
O1 -> default<O1>
O2 -> default<O2>
O3 -> default<O3>
Oz -> default<Oz>
```

The long-term implementation should run passes in-process through LLVM/Inkwell. Shelling out to `opt` is acceptable only as a bootstrap if in-process support blocks progress.

## Why LLVM First

Echo's current IR is intentionally direct and runtime-call-heavy. That is good for PHP compatibility work because each slice stays readable and testable.

LLVM should handle the first optimization layer:

- constant folding
- dead code elimination
- instruction combining
- simplify control flow
- mem2reg / stack-to-register promotion
- common subexpression elimination
- global value numbering
- loop invariant code motion
- scalar replacement of aggregates
- loop unrolling
- tail-call optimization

The most important early pass is `mem2reg`. Echo can initially lower locals with simple `alloca` / `store` / `load` patterns and let LLVM promote eligible values into SSA/registers.

## Frontend Optimizations

Do not block LLVM optimization work on Echo-specific frontend optimizations.

Later, Echo can add conservative frontend optimizations such as:

- literal constant folding
- string literal concatenation
- dead branch elimination for literal conditions
- simple echo coalescing
- type-specialized lowering
- unreachable-statement removal after `return`

Because Echo is PHP-compatible, frontend optimizations must be conservative. PHP-like behavior can expose side effects through warnings, conversions, output buffering callbacks, dynamic calls, autoloading, magic methods, and observable output order.

For example, coalescing adjacent echoes is only safe when buffering callbacks and intervening side effects cannot observe the boundary.

## Runtime Calls

Runtime calls such as `echo_write` are side-effecting and must not be removed or reordered incorrectly.

Start with conservative runtime declarations. Add LLVM attributes only when they are exactly true. Incorrect attributes can cause miscompiles.

Potential future attributes include:

- `nounwind`
- `readonly`
- `readnone`
- `noalias`
- `nocapture`
- `nonnull`
- `willreturn`

Examples that may eventually be safe after careful review:

```llvm
declare i64 @echo_string_len(ptr) readonly nounwind
declare noalias ptr @echo_alloc(i64)
```

Do not apply these attributes broadly to output or buffering functions.

## Implementation Plan

1. Add an optimization level enum shared by CLI/build code:

```rust
pub enum OptimizationLevel {
    O0,
    O1,
    O2,
    O3,
    Oz,
}
```

2. Parse optimization flags on `xo build`, defaulting to `O0`.

3. Verify LLVM modules after IR generation and before optimization/linking.

4. Run the selected LLVM default pipeline when level is not `O0`.

5. Let `--emit-ir` emit raw IR for `O0` and optimized IR for optimized modes.

6. Link native binaries from optimized IR/modules when optimization is enabled.

7. Add tests for broad optimization properties rather than exact LLVM output.

## Acceptance Criteria

- `xo build --emit-ir file.echo` emits readable unoptimized IR.
- `xo build --emit-ir -O0 file.echo` emits unoptimized IR.
- `xo build --emit-ir -O2 file.echo` emits optimized IR.
- `xo build -O2 file.echo` produces a working native binary.
- Existing PHP fixtures produce the same stdout under optimized and unoptimized builds.
- Runtime calls such as `echo_write` remain observable and are not optimized away.

Good early tests:

- A constant-foldable program such as `echo 2 + 3;` still outputs `5`, with optimized IR showing folding when practical.
- A simple variable program such as `$x = 1; echo $x;` still outputs `1`, with optimized IR avoiding obvious stack/load/store overhead if that lowering exists.

Do not assert exact full optimized IR. LLVM output changes across versions and pipeline details.
