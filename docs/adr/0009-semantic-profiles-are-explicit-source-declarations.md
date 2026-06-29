# Semantic Profiles Are Explicit Source Declarations

Echo may support opt-in semantic modernization profiles, but they are explicit Echo language declarations rather than file-extension modes, CLI flags, or PHP `declare(...)` directives. A future source form such as `semantics { strict }` should be parsed into the AST and consumed by `echo_semantics`, while the base language remains the shared PHP-compatible Echo superset.

The compatibility promise is that valid PHP remains valid Echo. It is not a promise that every Echo-accepted `.php` file can still run on stock PHP after it uses Echo-only features such as `module`, `fn`, typed collections, concurrency syntax, or semantic profile declarations.

The trade-off is adding a new language declaration and policy surface, but the boundary is cleaner than reviving `SourceMode`, `PhpCompat`, `--strict`, `--unsafe`, or extension-driven validation. Modernized code can give the compiler stronger facts for diagnostics, LSP features, HIR/MIR lowering, and optimized LLVM/runtime paths without splitting the parser or creating a second execution model.
