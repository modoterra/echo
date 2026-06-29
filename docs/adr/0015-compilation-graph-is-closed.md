# Compilation Graph Is Closed

Echo compiles a closed `CompilationGraph`, not an isolated entry file plus arbitrary runtime filesystem discovery. Static `require` and `include` edges are graph edges, and source code may admit additional graph members with a `compile { ... }` declaration so dynamic `require` targets are known before execution.

The declaration has three admission forms:

```php
compile {
    "./routes/*.php"
    "/srv/app/shared/bootstrap.php"
    "modoterra/laravel-echo"
}
```

Relative paths such as `"./routes/*.php"` resolve as if rooted at the declaring file's `__DIR__`. Absolute paths such as `"/srv/app/shared/bootstrap.php"` are host filesystem paths. Package names such as `"modoterra/laravel-echo"` load that entire package into the graph through package metadata.

Dynamic `require` and `include` may choose among files already admitted to the graph. If execution resolves a dynamic include to a file that is not in the graph, Echo reports an error instead of searching the filesystem or falling back to Composer's generated runtime autoload file.

The trade-off is deliberate: Echo gives up PHP's fully dynamic file loading for compiled programs in exchange for a static whole-program boundary, faster LLVM JIT/native execution, stronger diagnostics, safer incremental compilation, and a path to replace Composer autoload at runtime while still allowing Composer or future `xo get` metadata to acquire packages.
