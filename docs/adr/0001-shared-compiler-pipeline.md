# Shared Compiler Pipeline

Echo uses one shared compiler/runtime pipeline for language meaning: source files are parsed into AST, analyzed by `echo_semantics`, lowered through HIR and MIR, emitted as LLVM IR by `echo_codegen`, and executed either as a native binary or through LLVM JIT. User-facing tools such as `xo`, the REPL, LSP, indexing, syntax metadata, fixtures, and future formatting or highlighting surfaces may adapt and present facts from that pipeline, but they must not define separate syntax, semantic, type, or runtime behavior.

This keeps PHP compatibility and Echo extensions from fragmenting across tools. The trade-off is more explicit crate boundaries and more shared APIs up front, but the project avoids parallel mini-implementations of the language in the CLI, REPL, editor server, or test harnesses.
