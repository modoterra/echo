# LLVM IR Execution Boundary

Echo treats LLVM IR as the execution boundary for compiled programs. `xo build` produces native binaries from generated LLVM IR, `xo run` may execute the same IR through either a temporary native binary or LLVM JIT, and embedded execution should use LLVM JIT with registered `echo_runtime` symbols rather than a separate Echo VM.

Echo is not planning a custom bytecode VM or interpreter as a parallel execution engine. That keeps runtime semantics centralized in Rust-owned `echo_runtime` plus LLVM lowering, and avoids having PHP compatibility, Echo extensions, reflection, output buffering, and dynamic calls drift between two engines. The trade-off is accepting LLVM as a deep dependency for execution, including JIT embedding, instead of owning a smaller portable bytecode format.
