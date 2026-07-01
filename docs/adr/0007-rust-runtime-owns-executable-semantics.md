# Rust Runtime Owns Executable Semantics

Echo keeps executable language semantics in Rust-owned runtime crates. `echo_runtime` owns PHP/Echo value behavior, collections, output buffering, reflection dispatch, dynamic calls, the Echo PHP Surface, standard-library intrinsics, process/task behavior, and other runtime operations; generated LLVM IR calls stable runtime ABI symbols such as `echo_*`, `echo_php_*`, `echo_std_*`, and future `echo_ext_*`.

`echo_codegen` owns MIR-to-LLVM lowering and ABI routing, not the semantic implementation of Echo PHP Surface functions or Echo runtime behavior. Codegen may select a direct runtime symbol for a statically known function or intrinsic, but dynamic calls, reflection, value coercions, collection operations, and observable runtime behavior must remain centralized in `echo_runtime`.

The trade-off is a larger Rust runtime ABI surface, but Echo avoids implementing language behavior in LLVM snippets, C/C++ helper libraries, CLI code, or test harnesses. Native builds and LLVM JIT execution therefore share the same runtime semantics.
