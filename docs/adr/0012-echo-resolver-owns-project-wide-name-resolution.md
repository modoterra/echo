# `echo_resolver` Owns Project-Wide Name Resolution

Echo will use an `echo_resolver` crate for project-wide name and import resolution policy. `echo_index` stores extracted facts; `echo_resolver` consumes those facts plus package roots, Composer autoload metadata, Echo package metadata, std metadata, and source roots to produce resolved imports, resolved symbols, canonical names, ambiguity diagnostics, and dependency edges.

The concrete resolver artifact should be explicit enough for compiler and tooling consumers:

```rust
pub struct ResolverInput {
    pub index: echo_index::EchoIndex,
    pub package_roots: Vec<PackageRoot>,
    pub std_modules: Vec<StdModule>,
}

pub struct ResolvedSymbol {
    pub canonical_name: CanonicalName,
    pub file_id: echo_index::FileId,
    pub declaration_id: echo_index::SymbolId,
    pub surface: SourceSurface,
}

pub struct ResolvedImport {
    pub local_name: String,
    pub canonical_name: CanonicalName,
    pub target: ResolvedSymbol,
}
```

`echo_resolver` owns module/namespace canonicalization, reserved `std` enforcement, `from std use ...` binding, Echo package module lookup, Composer classmap/PSR-4 lookup, ambiguity handling, and resolver diagnostics. It must not parse source text, own AST shape, perform semantic type checking, lower code, execute Composer, or decide runtime behavior.

Existing ad hoc resolution in `xo`, `echo_lsp`, and codegen is transitional. New resolution policy should land in `echo_resolver`, and local lookup code should move there when touched so LSP navigation, semantic analysis, CLI build/run, reflection metadata, native builds, and LLVM JIT execution consume the same resolution facts.
