# Source Identity Belongs in echo_source

Echo source identity belongs in `echo_source`. Parsers, diagnostics, LSP features, resolver facts, HIR, MIR, codegen, and CLI presentation should refer to source files through a shared source registry instead of each layer inventing its own path/text bookkeeping.

The target model has three distinct concepts:

```rust
pub struct SourceId(u32);

pub struct SourceMap {
    // Stable registry from source identity to SourceFile.
}

pub struct SourceSpan {
    pub source_id: SourceId,
    pub span: Span,
}
```

`Span` remains a byte-offset range within one source text. `SourceId` identifies which registered source owns that range. `SourceSpan` combines both facts for diagnostics, indexing, LSP, resolver output, and eventual AST/HIR/MIR source references.

The migration must be phased:

1. Add `SourceId` and `SourceMap` to `echo_source` while keeping `Span` and `SourceFile::new` compatible with current callers.
2. Add `SourceSpan` and use it first in diagnostics and LSP conversion paths, where cross-file identity matters most.
3. Migrate diagnostics from bare `Span` to `SourceSpan`, preserving CLI output and LSP ranges.
4. Migrate parser entrypoints to accept registered `SourceFile` values instead of anonymous source text, so parsed programs have a source identity before AST construction.
5. Decide whether AST nodes store `SourceSpan` directly or keep local `Span` plus a program-level `SourceId`. That decision should be based on memory profile, AST ergonomics, and LSP/index needs after diagnostics and parser entrypoints have moved.

This is separate from language mode. `.php`, `.echo`, and `.xo` files still enter the same language pipeline; source identity records where text came from, not which syntax or semantic mode should apply. REPL snippets, std modules, package files, and anonymous test sources can all receive stable source identities without adding source-specific language rules.

The trade-off is introducing a registry before every compiler layer consumes it. That staged cost is acceptable because it gives diagnostics, resolver work, LSP features, project indexing, and multi-file compilation a common identity model without forcing an immediate AST-wide span rewrite.
