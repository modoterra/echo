# Linux And Macos Are The Platform Baseline

Echo targets Linux and macOS as its supported host platforms. Windows-native behavior is out of scope for the current compatibility baseline, including Windows path syntax, drive roots, ACLs, shell quoting, text-mode file translation, and Windows-specific filesystem or process behavior. Running Echo under WSL is acceptable because it presents a Linux host model to the compiler and runtime.

This keeps PHP compatibility work focused on the Unix/POSIX-like behavior Echo already uses for local files, permissions, symlinks, process environment, and native execution. Cross-platform Rust APIs remain useful when they preserve Linux/macOS behavior, but new compatibility slices should not spend implementation or test budget on Windows-specific semantics unless Echo later adopts a separate platform policy.
