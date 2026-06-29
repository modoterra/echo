# Semantic Facts Belong in `echo_semantics`

Echo treats semantic analysis as the authoritative compiler layer for language meaning. Facts such as variable bindings, expression types, scopes, receiver validity, symbol resolution, and diagnostics that require meaning rather than syntax alone belong in `echo_semantics` and are consumed by HIR, MIR, codegen, REPL presentation, LSP, indexing, and future LLVM JIT execution.

This follows the normal compiler split between parsing, semantic analysis, lowering, and execution. The trade-off is that editor and CLI features must wait for shared semantic APIs instead of adding local shortcuts, but Echo avoids duplicate type checkers, private REPL environments, LSP-only resolvers, and backend-specific semantic rules.
