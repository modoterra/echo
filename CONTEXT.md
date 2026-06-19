# Echo Context

This is the global glossary and domain context for Echo.

## Project Shape

Echo is a Rust implementation of a PHP superset. Existing PHP programs should
remain valid while Echo adds stricter modes, modern runtime features, and new
language constructs.

The regular language pipeline is:

1. source mode
2. lexer/parser
3. AST
4. semantic and type analysis
5. IR/codegen or VM execution
6. runtime behavior

Behavior belongs in the earliest shared layer that owns it. User-facing tools
may expose or format behavior, but should not define language semantics locally.

## Module Ownership

`echo_ast` owns syntax tree shape.

Collection syntax has distinct meanings and must not be conflated:

- `[]` is a PHP-compatible array literal.
- `{}` is a strict Echo list literal.
- `{ field: value }` is a strict Echo structural object literal.
- `()` is reserved for tuples.
- Fixed-size arrays are distinct from dynamic arrays and lists.
PHP `$value[] = item` append syntax applies only to non-fixed arrays, not Echo
lists or fixed-size arrays.

`echo_parser` owns source parsing and source-mode validation.

`echo_semantics` owns semantic and type analysis: variable bindings, expression
facts, scope rules, symbol resolution, and diagnostics that require meaning
rather than syntax alone. It should serve file compilation, REPL introspection,
future VM execution, and future LSP features.

`echo_codegen` owns LLVM lowering.

`echo_runtime` owns executable value and runtime behavior.

`xo` owns CLI orchestration and presentation.

## REPL

`xo repl` is an interactive host for normal Echo programs. It owns prompt
handling, command history, session presentation, and display formatting.

The REPL does not own Echo language semantics. It must not implement its own
parser, evaluator, type environment, reflection lookup table, or value rules.
When REPL examples expose missing behavior, implement that behavior in the
shared language pipeline so the same source works from a file.

Long term, the REPL should execute through an open IR/VM session instead of
shelling out through `clang` for each input. That VM is an execution strategy for
the same IR/runtime semantics used by regular programs, not a separate
interpreter.

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
  through `xo ast`, `xo ir`, `xo run`, and `xo build`, unless the feature is
  explicitly documented as VM-only.
- REPL state should eventually be a live IR/VM session over the shared runtime
  model.
- Missing type, reflection, or value behavior should be added to shared
  semantic, IR, or runtime layers before the REPL displays it.
