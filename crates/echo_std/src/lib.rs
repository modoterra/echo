//! Echo standard library boundary.
//!
//! `echo_std` is the home for Echo-facing standard library APIs. It should be
//! built on top of `echo_runtime` primitives and kept separate from PHP builtin
//! compatibility exports (`echo_php_*`) and future extension exports
//! (`echo_ext_*`).
//!
//! The first HTTP server should be expressed through this standard library, not
//! as an `xo serve` command.

pub fn library_name() -> &'static str {
    "echo_std"
}
