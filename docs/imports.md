# Imports

## Decision

Echo keeps PHP imports and Echo-owned imports separate.

```php
use Psr\Log\LoggerInterface
use Symfony\Component\HttpFoundation\Request as SymfonyRequest

from std use net\TcpServer
from illuminate/http use Request
from "./routes.echo" use route
from "./config.json" use config satisfies Config
```

This example shows the import lanes side by side: PHP namespace imports stay PHP-compatible, package-style Echo imports bind module exports, `std` imports bind Echo-owned library modules, and file imports can validate external data at compile time.

Plain `use ...` remains PHP namespace import syntax. Composer/autoloaded PHP classes, interfaces, functions, and constants continue to use PHP-compatible resolution.

`from ... use ...` is Echo-owned import syntax. It is for standard library imports, vendor or package modules, local Echo modules, and file-backed data modules.

`from std use ...` is intended to be real import syntax, not documentation sugar. The resolver must bind it to compiler-known standard library modules supplied by `echo_std`.

## Import Sources

```text
use Foo\Bar
  PHP namespace import; compatible with Composer/autoloading.

from std use net\TcpServer
  Echo standard library import; resolves to std.net.TcpServer and does not reserve a PHP namespace.

from illuminate/http use Request
  Echo package import; resolves through Echo package/module rules, not PHP namespace rules.

from "./file.echo" use name
  Local Echo module import.

from "./config.json" use config
  File-backed data import. The file extension selects the loader.
```

This block is the resolver contract: the import prefix determines whether Echo follows PHP namespace rules, stdlib module rules, package/module rules, local Echo module loading, or data-loader behavior.

The file extension stays in the import source because it is part of loader selection.

Initial loader direction:

- `.echo` / `.xo`: parse Echo exports.
- `.json`: parse as readonly data.
- `.yaml` / `.yml`: parse as readonly data if YAML support is enabled.
- Unknown extension: diagnostic.

## Typed Data Imports

Use `satisfies` to validate imported data against a structural type without changing the imported name syntax.

```php
type Config = {
    database: {
        host: string
        port: int
    }
}

from "./config.json" use config satisfies Config
```

This example loads a real configuration file as data and validates it before the rest of the program can use `config`.

Meaning:

```text
Load ./config.json as readonly data.
Bind the imported value as `config`.
Validate that the loaded data satisfies `Config`.
Expose `config` with type `Config` after validation.
```

The expanded meaning separates loading, binding, validation, and typing so diagnostics can point at the import and at the offending data path.

If validation fails, compilation should fail with a diagnostic pointing at both the import and the offending data path where possible.

For example, if `database.port` is a string instead of an int, report that path explicitly.

## Standard Library Imports

Standard library imports use `from std use ...`.

```php
from std use net\TcpServer
from std use http\Response
```

This is the normal application import shape for Echo-native networking and HTTP APIs: import from `std`, then use local names in code.

This imports from Echo's standard library module graph:

```text
net\TcpServer  -> std.net.TcpServer
http\Response  -> std.http.Response
```

This mapping keeps user syntax compact while preserving the compiler's full stdlib module identity internally.

The imported local names are `TcpServer` and `Response` unless an alias is provided.

```php
from std use http\Response as HttpResponse
```

Aliases are for avoiding local naming conflicts while still resolving through the same stdlib module graph.

`from std use ...` must not consult PHP namespace resolution or Composer autoloading. It resolves only against the compiler-known stdlib surface. Other `from <package> use ...` sources, such as `from illuminate/http use ...`, can use Echo package resolution later without changing the import grammar.

The stdlib surface can be implemented by a mix of Echo source and trusted intrinsic declarations. See [Echo Standard Library](stdlib.md) for the interop model.

Stdlib source declares its module with the matching `namespace std ...` form:

```php
namespace std net

class TcpServer {
    pub intrinsic static fn listen(string $address): TcpServer
}
```

This declaration is trusted stdlib source: it names the Echo module and declares the public API that the compiler can bind to runtime intrinsics.

`namespace std net` means `std.net`. It is not the same as `namespace std\Net`.

- `namespace std net`: trusted Echo stdlib module declaration.
- `namespace std\Net`: ordinary PHP namespace declaration named `std\Net`.

Only trusted stdlib files may use `namespace std ...`. User files that need a PHP namespace named `std\Net` can still use normal PHP namespace syntax.

## Why `satisfies`

`satisfies` reads as a constraint on an imported value:

```php
from "./config.json" use config satisfies Config
```

The value being constrained is the imported data, so `satisfies` reads like an import-time validation clause rather than a variable declaration.

It avoids making the import look like a declaration with reordered type syntax, and it can later work for multiple import forms.

Potential future examples:

```php
from "./routes.yaml" use routes satisfies list<Route>
from "./openapi.json" use schema satisfies OpenApiSchema
```

These examples show the intended scale of the feature: routes, schemas, and other file-backed data can become typed inputs without hand-written loader code.

## Namespace Compatibility

Echo standard library imports do not live under a reserved PHP namespace such as `Echo\...`, `EchoStd\...`, or `Std\...`.

This avoids breaking valid PHP programs that already define or autoload those namespaces.

```php
use Echo\Net\TcpServer
```

This remains ordinary PHP-compatible source and should continue to work for projects that already own an `Echo\...` namespace.

The example above remains a PHP/userland namespace import, not an Echo standard library import.

Use this for the Echo standard library instead:

```php
from std use net\TcpServer
```

This spelling opts into Echo's stdlib resolver explicitly and avoids reserving PHP namespaces globally.

## Parser And Resolver Shape

AST should distinguish PHP imports from Echo-owned imports.

```rust
pub enum ImportSource {
    PhpNamespace,
    Std,
    File { path: String },
}

pub struct ImportStmt {
    pub source: ImportSource,
    pub items: Vec<ImportItem>,
}

pub struct ImportItem {
    pub name: QualifiedName,
    pub alias: Option<Identifier>,
    pub satisfies: Option<TypeExpr>,
}
```

This AST shape keeps PHP namespace imports, stdlib imports, file imports, aliases, and data-validation constraints distinguishable before semantic resolution.

Plain PHP-compatible `use Foo\Bar` can remain represented separately as a PHP namespace import if that keeps compatibility parsing simpler.

Resolver responsibilities:

- `use Foo\Bar`: bind through PHP namespace rules and allow Composer/autoloaded symbols.
- `from std use Foo\Bar`: bind through the stdlib module graph supplied by `echo_std`.
- `from vendor/package use Foo`: bind through Echo package/module resolution.
- `from "./file.echo" use name`: load the Echo module and bind exported names.
- `from "./data.json" use data satisfies Type`: load data, validate it against `Type`, and bind the typed value.

## Current Coverage

- `from std use ...` is parsed into a distinct Echo-owned import AST node.
- Codegen validates the first imported path segment against packaged `echo_std` modules such as `std.net`, `std.time`, and `std.http`.
- Unknown std module imports fail with a diagnostic such as `unknown std import \`potato\``.
- Module imports bind the local module name for static module-style intrinsic calls such as `net.listen(...)` and `http.responseText(...)`.
- Module aliases are supported for module-style intrinsic calls, for example `from std use net as socket` enables `socket.listen(...)`.
- Known std module calls without a matching import fail with a diagnostic such as `std module \`net\` must be imported before use`.
- Item-level stdlib resolution, local file imports, and typed data imports are still future resolver work.
