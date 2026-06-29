# Echo Context

This is the global glossary and domain context for Echo.

## Project Shape

Echo is a Rust implementation of a PHP superset. Existing PHP programs should
remain valid while Echo adds modern runtime features and new language
constructs.

The compatibility promise means valid PHP remains valid Echo. It does not mean
every Echo-accepted `.php` file remains runnable by stock PHP after using
Echo-only syntax.

The regular language pipeline is:

1. lexer/parser
2. AST
3. semantic and type analysis
4. HIR and MIR lowering
5. LLVM IR/codegen
6. native binary or LLVM JIT execution
7. runtime behavior

Behavior belongs in the earliest shared layer that owns it. User-facing tools
may expose or format behavior, but should not define language semantics locally.

## Module Ownership

`echo_ast` owns syntax tree shape. AST nodes represent parsed source syntax and
source-level structure, not lowered semantic, runtime, or backend meaning.

Collection syntax has distinct meanings and must not be conflated:

- `[]` is a PHP-compatible array literal.
- `{}` is a strict Echo list literal.
- `{ field: value }` is a strict Echo structural object literal.
- `()` is reserved for tuples.
- Fixed-size arrays are distinct from dynamic arrays and lists.
PHP `$value[] = item` append syntax applies only to non-fixed arrays, not Echo
lists or fixed-size arrays.

`echo_parser` owns source parsing. Echo has one language mode for `.php`,
`.echo`, and `.xo` files; extensions do not enable or disable syntax or
semantic validity.

Opt-in modernization policies are source-level semantic declarations, not file
modes. A future declaration such as `semantics { strict }` should be represented
in the AST and enforced by `echo_semantics`.

Echo `module acme.http` declarations and PHP-compatible `namespace Acme\Http`
declarations are source syntaxes that can denote the same internal package or
module identity. The compiler may preserve source spelling for diagnostics,
formatting, package conventions, and PHP compatibility, but corresponding module
and namespace declarations should resolve to one symbol model and lower to the
same HIR, MIR, LLVM IR, and runtime behavior.

The canonical `std` root is reserved for Echo's compiler-owned standard
library. User and package code must not declare modules or namespaces that
canonicalize to `std`, including `module std.net` and `namespace Std\Net`.

Callable resolution has three source surfaces that converge after resolution:
PHP globals, Echo std modules, and user/package declarations. PHP globals keep
PHP names and compatibility metadata; std APIs live under the reserved `std`
root and may be regular Echo source or trusted intrinsics; user/package
declarations resolve through the shared module/namespace symbol model.

`echo_semantics` owns semantic and type analysis: variable bindings, expression
facts, scope rules, symbol resolution, and diagnostics that require meaning
rather than syntax alone. It should serve file compilation, REPL introspection,
LLVM JIT execution, and future LSP features.

`echo_resolver` is the planned crate for project-wide name and import resolution
policy. It should consume `echo_index` facts plus package roots, Composer
autoload metadata, Echo package metadata, std metadata, and source roots, then
produce resolved imports, resolved symbols, canonical names, ambiguity
diagnostics, and dependency edges for semantics, LSP, `xo`, and codegen.

`echo_diagnostics` owns the shared diagnostic contract. Diagnostics should grow
from the current message/span bootstrap shape into stable codes, severity,
primary and related spans, and owning-layer metadata that CLI, LSP, tests, and
docs can all consume.

`echo_hir` owns the first compiler-friendly representation after parsing. HIR
is derived from AST plus `echo_semantics` facts and preserves enough source
structure for diagnostics, tooling, and language-level reasoning.

`echo_mir` owns backend-neutral executable lowering between HIR and LLVM IR. MIR
may desugar source constructs and regularize control flow, calls, imports,
functions, classes, and runtime operations for code generation, but it must not
become VM bytecode, an interpreter format, or a second semantic engine.

`echo_codegen` owns MIR to LLVM IR lowering and ABI routing. It may choose
stable runtime symbols for known operations, but it does not own PHP built-in,
stdlib intrinsic, dynamic-call, collection, reflection, or value semantics.

`echo_runtime` owns executable value and runtime behavior in Rust: PHP/Echo
values, collections, output buffering, reflection dispatch, dynamic calls,
built-ins, standard-library intrinsics, process/task behavior, and other
observable runtime operations.

`xo` owns CLI orchestration and presentation.

## REPL

`xo repl` is an interactive host for normal Echo programs. It owns prompt
handling, command history, session presentation, and display formatting.

The REPL does not own Echo language semantics. It must not implement its own
parser, evaluator, type environment, reflection lookup table, or value rules.
When REPL examples expose missing behavior, implement that behavior in the
shared language pipeline so the same source works from a file.

Long term, the REPL should execute through an open LLVM JIT session instead of
shelling out through `clang` for each input. That JIT session is an execution
strategy for the same LLVM IR and runtime semantics used by regular programs,
not a separate interpreter.

## Open Program Session

An open program session is the REPL's accumulated program state: previous
imports, declarations, definitions, assignments, and runtime values that future
inputs can see.

This state should be represented in terms of the shared compiler and runtime
model. It should not become a REPL-local semantic side table that regular Echo
programs cannot use.

## Invariants

- REPL examples are language-development inputs. Do not solve them with
  REPL-only hacks.
- If a REPL input should be valid Echo, the equivalent source file should work
  through `xo ast`, `xo ir`, `xo run --jit`, `xo run`, and `xo build`.
- REPL state should eventually be a live LLVM JIT session over the shared
  runtime model.
- Missing type, reflection, or value behavior should be added to shared
  semantic, IR, or runtime layers before the REPL displays it.
