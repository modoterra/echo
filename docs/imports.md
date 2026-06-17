# Imports

## Decision

Echo keeps PHP imports and Echo-owned imports separate.

```php
use Psr\Log\LoggerInterface
use Symfony\Component\HttpFoundation\Request as SymfonyRequest

from std use Net\TcpServer
from "./routes.echo" use route
from "./config.json" use config satisfies Config
```

Plain `use ...` remains PHP namespace import syntax. Composer/autoloaded PHP classes, interfaces, functions, and constants continue to use PHP-compatible resolution.

`from ... use ...` is Echo-owned import syntax. It is for standard library imports, local Echo modules, future Echo packages, and file-backed data modules.

`from std use ...` is intended to be real import syntax, not documentation sugar. The resolver must bind it to compiler-known standard library modules supplied by `echo_std`.

## Import Sources

```text
use Foo\Bar
  PHP namespace import; compatible with Composer/autoloading.

from std use Net\TcpServer
  Echo standard library import; resolves to std.Net.TcpServer and does not reserve a PHP namespace.

from "./file.echo" use name
  Local Echo module import.

from "./config.json" use config
  File-backed data import. The file extension selects the loader.
```

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

Meaning:

```text
Load ./config.json as readonly data.
Bind the imported value as `config`.
Validate that the loaded data satisfies `Config`.
Expose `config` with type `Config` after validation.
```

If validation fails, compilation should fail with a diagnostic pointing at both the import and the offending data path where possible.

For example, if `database.port` is a string instead of an int, report that path explicitly.

## Standard Library Imports

Standard library imports use `from std use ...`.

```php
from std use Net\TcpServer
from std use Http\Response
```

This imports from Echo's standard library module graph:

```text
Net\TcpServer  -> std.Net.TcpServer
Http\Response  -> std.Http.Response
```

The imported local names are `TcpServer` and `Response` unless an alias is provided.

```php
from std use Http\Response as HttpResponse
```

`from std use ...` must not consult PHP namespace resolution or Composer autoloading. It resolves only against the compiler-known stdlib surface.

The stdlib surface can be implemented by a mix of Echo source and trusted intrinsic declarations. See [Echo Standard Library](stdlib.md) for the interop model.

Stdlib source declares its module with the matching `namespace std ...` form:

```php
namespace std Net

class TcpServer {
    intrinsic static function listen(string $address): TcpServer
}
```

`namespace std Net` means `std.Net`. It is not the same as `namespace std\Net`.

- `namespace std Net`: trusted Echo stdlib module declaration.
- `namespace std\Net`: ordinary PHP namespace declaration named `std\Net`.

Only trusted stdlib files may use `namespace std ...`. User files that need a PHP namespace named `std\Net` can still use normal PHP namespace syntax.

## Why `satisfies`

`satisfies` reads as a constraint on an imported value:

```php
from "./config.json" use config satisfies Config
```

It avoids making the import look like a declaration with reordered type syntax, and it can later work for multiple import forms.

Potential future examples:

```php
from "./routes.yaml" use routes satisfies list<Route>
from "./openapi.json" use schema satisfies OpenApiSchema
```

## Namespace Compatibility

Echo standard library imports do not live under a reserved PHP namespace such as `Echo\...`, `EchoStd\...`, or `Std\...`.

This avoids breaking valid PHP programs that already define or autoload those namespaces.

```php
use Echo\Net\TcpServer
```

The example above remains a PHP/userland namespace import, not an Echo standard library import.

Use this for the Echo standard library instead:

```php
from std use Net\TcpServer
```

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

Plain PHP-compatible `use Foo\Bar` can remain represented separately as a PHP namespace import if that keeps compatibility parsing simpler.

Resolver responsibilities:

- `use Foo\Bar`: bind through PHP namespace rules and allow Composer/autoloaded symbols.
- `from std use Foo\Bar`: bind through the stdlib module graph supplied by `echo_std`.
- `from "./file.echo" use name`: load the Echo module and bind exported names.
- `from "./data.json" use data satisfies Type`: load data, validate it against `Type`, and bind the typed value.
