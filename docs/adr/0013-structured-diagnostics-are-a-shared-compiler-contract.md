# Structured Diagnostics Are a Shared Compiler Contract

Echo diagnostics are a shared compiler contract owned by `echo_diagnostics`, not local presentation structs in `xo`, LSP, parser, resolver, semantics, HIR, MIR, codegen, or runtime-contract checks. The current `Diagnostic { message, span }` shape is a minimal bootstrap form; it should grow into a structured diagnostic model with stable code, severity, primary span, optional related spans, and owning layer.

The target shape should be explicit enough for CLI output, LSP publishing, fixture assertions, quick fixes, and documentation:

```rust
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
    pub primary_span: Span,
    pub related: Vec<RelatedDiagnostic>,
    pub owner: DiagnosticOwner,
}

pub enum DiagnosticOwner {
    Lexer,
    Parser,
    Resolver,
    Semantics,
    Hir,
    Mir,
    Codegen,
    RuntimeContract,
}
```

Each layer owns the diagnostics for the facts it owns: lexer diagnostics for tokenization, parser diagnostics for syntax, resolver diagnostics for imports and ambiguity, semantic diagnostics for meaning and types, lowering diagnostics for unsupported IR transitions, and runtime-contract diagnostics for compile-time/runtime ABI violations. Presentation layers may format or translate diagnostics, but must not invent independent diagnostic categories or codes.

The trade-off is maintaining stable diagnostic metadata as the compiler grows. That cost is worth it because LSP filtering, quick fixes, test assertions, semantic profiles, resolver ambiguity errors, and CLI reports need the same diagnostic vocabulary instead of fragile message-string matching.
