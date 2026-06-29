# Callable Resolution Uses the Shared Symbol Model

Echo has distinct callable surfaces for PHP globals, Echo standard-library APIs, and user/package declarations, but successful resolution feeds one shared symbol model before HIR, MIR, LLVM lowering, and runtime execution. PHP globals such as `strlen()` and `count()` keep PHP source names and compatibility metadata; Echo std APIs live under the reserved `std` module root; user/package declarations resolve after module/namespace canonicalization.

The standard library is a real Echo module graph, not only a runtime ABI table. Std APIs may be regular Echo source compiled through the normal pipeline, or trusted `intrinsic` declarations that lower through compiler-approved `echo_std_*` or core runtime ABI symbols.

`echo_semantics` owns the eventual resolver facts that say which callable a source call denotes. `echo_codegen` may route already-resolved known operations to ABI calls, but it must not become the source of truth for PHP builtin lookup, std import meaning, userland function binding, or dynamic callable validity.

The trade-off is that the resolver must model several source surfaces and ABI lanes, but Echo avoids hiding language resolution in codegen registries. That keeps PHP compatibility, std modules, user packages, LSP navigation, reflection metadata, native builds, and LLVM JIT execution aligned.
