# Module and Namespace Syntax Converge Before Lowering

Echo treats `module acme.http` and PHP-compatible `namespace Acme\Http` as source syntaxes that can denote the same internal package/module identity. The compiler may preserve which spelling appeared for diagnostics, formatting, package conventions, and PHP compatibility, but neither spelling is the privileged runtime or lowering model.

When an Echo module path and a PHP namespace correspond under package naming rules, they should resolve to the same symbols and lower to the same HIR, MIR, LLVM IR, and runtime behavior. Imports follow the same principle: Echo `use`/`from` forms and PHP `use` forms may have different source grammar and ecosystem lookup rules, but successful resolution should feed one shared symbol model instead of an Echo-only path and a PHP-only path.

The trade-off is that name resolution must carry both canonical identity and source spelling. That complexity is worth it because Echo remains compatible with existing PHP namespaces while giving Echo code lower-dot module syntax without creating two languages or two code-generation paths.
