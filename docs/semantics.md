# Echo Semantics

`echo_semantics` is the shared compiler layer between parsed AST and execution
backends. It owns language facts that require meaning rather than syntax alone:
variable bindings, expression types, scopes, symbol resolution, and semantic
diagnostics.

The layer is intentionally reusable:

```text
echo_parser -> echo_ast -> echo_semantics -> echo_codegen
                                      |
                                      +-> xo repl metadata
                                      +-> future LLVM JIT execution
                                      +-> future echo_lsp
```

`xo repl` must not maintain a private type environment. It may keep an open
program session as source/AST state, but display metadata should come from
`echo_semantics` over that accumulated program. File compilation uses the same
analysis before LLVM lowering, so REPL behavior and file behavior stay aligned.

Current scope:

- infer basic literal and collection expression types;
- track variable types through `let`, assignment, and reference assignment;
- report undefined variable references before codegen;
- expose expression facts by source span for presentation tools.

Future scope:

- source-mode-aware strict typing;
- import and standard-library resolution;
- richer object, array, function, and callable types;
- expression identifiers that are stable across repeated REPL snippets;
- LSP-friendly symbol tables and reference indexes.
