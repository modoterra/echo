# HIR and MIR Are Distinct Lowering Layers

Echo keeps HIR and MIR as separate compiler layers. HIR is the first compiler-friendly representation after parsing: it is derived from AST plus shared `echo_semantics` facts and preserves enough source structure for diagnostics, tooling, and language-level reasoning.

MIR is the required backend-neutral executable lowering layer between HIR and LLVM IR. It may desugar source constructs and regularize control flow, calls, imports, functions, classes, and runtime operations for code generation, but it must not become a VM bytecode, interpreter format, or second semantic engine.

The trade-off is another intermediate representation to maintain, but the boundary keeps parsing, semantic analysis, executable lowering, and LLVM emission testable independently. This is the normal compiler-design split Echo needs for PHP compatibility, Echo extensions, LSP support, formatting, native builds, and LLVM JIT execution to share one language model.
