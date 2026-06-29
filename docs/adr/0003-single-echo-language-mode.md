# Single Echo Language Mode

Echo compiles `.php`, `.echo`, and `.xo` files as the same language. Valid PHP remains valid, Echo extensions are available everywhere, and file extension does not enable or disable parser rules, semantic validation, dynamic dispatch, keyed arrays, PHP references, or other source behavior.

Extensions still matter for ecosystem conventions such as package discovery, filenames, editor associations, and whether a file is expected to run under stock PHP, but they are not compiler modes. A `.php` file that uses Echo-only syntax is no longer stock-PHP source; that is a source compatibility choice, not a parser mode.

This replaces the earlier strict/unsafe split because mode-specific validation made behavior harder to predict and encouraged tools such as the CLI and LSP to carry parallel language policy. Future stricter semantics should be explicit source declarations rather than extension-driven behavior.
