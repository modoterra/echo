# `std` Is a Reserved Standard Library Root

Echo reserves the canonical `std` root for the compiler-owned standard library. Packaged standard-library source declares modules under that root with Echo module syntax such as `module std.net`; user and package code must not declare modules or namespaces that canonicalize to that root, including `module std.net` and PHP-compatible namespace declarations such as `namespace Std\Net`.

This follows from Echo's module/namespace convergence rule: if `module std.net` and `namespace Std\Net` can lower to the same internal identity, then both spellings must be protected from user declarations. User code imports the standard library through Echo std import syntax such as `use std.net` or `from std use net`; it does not create declarations under `std`.

The trade-off is that Echo reserves a small namespace that stock PHP would otherwise allow. That is acceptable because `std` is a core Echo language facility, and allowing user declarations under canonical `std` would make resolution, LSP navigation, package loading, and code generation ambiguous.
