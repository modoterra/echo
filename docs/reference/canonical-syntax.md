# Canonical Echo Syntax

Echo accepts PHP-compatible source forms, but documentation, examples, generated code, and formatter output should prefer the canonical Echo forms below.

## File And Module Names

Echo source files use `snake_case.echo` file names, and directories use lowercase names with underscores when needed. Echo-native package files declare module names as lowercase `snake_case` segments separated by dots.

```echo
module modoterra.laravel_echo.console

let $command_name = "echo:start"
```

This keeps Echo package structure readable without carrying PHP class-file naming conventions into Echo source.

Module names must stay lowercase and snake_case.

```echo
module app.http.router
```

These spellings are invalid Echo module declarations:

```echo
module App.Http.Router
module app.http-router
module app.2http.router
module app..router
```

This gives Echo package modules one canonical spelling instead of inheriting PHP namespace casing rules.

The parser may still parse a non-canonical module declaration so diagnostics can point at the exact segment. The resolver or semantic layer should reject it with a module-name diagnostic rather than treating it as a distinct module identity.

Every Echo file that is meant to be imported by module identity should declare a `module`. Entry scripts and anonymous one-off scripts may omit the declaration; they do not need a placeholder such as `module main`.

```echo
use app.http.router

echo router.route($request)
```

This keeps importable package files explicit while preserving lightweight script files for entrypoints.

## Types And Declarations

Classes, traits, interfaces, enums, and type names use `PascalCase`. Functions, variables, modules, and file/module identifiers use `snake_case`. Echo-native exported declarations use `pub`.

```echo
module app.console

class StartServerCommand {
    pub fn handle($server_name) {
        echo $server_name
    }
}

fn normalize_path($input) {
    return $input
}
```

This style separates type-like names from value-like names and keeps Echo examples consistent across packages.

Module exports are explicit. Use Echo declaration forms such as `pub fn` and `pub type`; non-`pub` declarations are module-private.

```echo
module app.support.request_id

pub fn request_id() {
    return "req_123"
}

fn normalize_header($value) {
    return $value
}
```

Other modules may import `request_id`, but not `normalize_header`.

Use `fn` for Echo-native functions and methods. The `function` keyword remains accepted for PHP-compatible source; it declares the same kind of callable as `fn`, but the spelling is semantic metadata that tools and compatibility policy can observe.

```echo
pub fn handle($request: Request): Response {
    return response($request)
}

class StartServerCommand {
    pub fn handle() {
        return 0
    }
}
```

This lets tools distinguish canonical Echo declarations from PHP-compatible declarations without changing callable runtime behavior.

The same rule applies to methods: `pub fn` is canonical Echo spelling, while `public function` remains PHP-compatible spelling for the same method model. Under strict Echo semantics, PHP-compatible declaration spelling such as `public function` is not allowed.

Strict-mode rejection happens after parsing. The AST should still represent PHP-compatible spelling truthfully, and the semantics pass decides whether that spelling is allowed under the file's active semantic policy.

```echo
class UserController {
    pub fn show($request) {
        return response($request)
    }
}
```

Echo-native `fn` parameters use suffix type annotations, matching `let` bindings. PHP-compatible `function` declarations keep PHP-style prefix parameter types.

Function and method return types use `: Type` after the parameter list.

Nullable types may use either concise `?T` spelling or explicit union spelling, but `?T` is preferred for the single-type nullable case. General unions use `A|B|C` spelling.

```echo
let $user: ?User = null
let $fallback: User|null = null
let $id: int|string = "guest"
fn find_user($id: int): ?User {
    return null
}
```

Preserve author order in union type syntax. The compiler may canonicalize unions internally for type equality, but a formatter should not reorder source unions because order can communicate intent.

```echo
fn parse_user_id($raw: int|string): ?int {
    return null
}

type SaveUserResult = SavedUser|ValidationError|PermissionDenied
```

Generic type arguments use angle brackets with no space before `<` and spaces after commas.

```echo
let $users: list<User> = {}
let $counts: array<string, int> = []
let $result: Result<SavedUser, ValidationError> = save_user($input)
```

## Collections

Collection delimiters have distinct meanings.

```echo
let $php_array = ["id" => 1, "name" => "Echo"]
let $list: list<int> = {1, 2, 3}
let $user = { id: 1, name: "Echo" }
let $pair = (1, "Echo")
let $rgb: array<int>[3] = [255, 128, 0]
```

Use `[]` for PHP arrays, `{}` for Echo lists, `{ field: value }` for Echo structural objects, `()` for tuples, and fixed-size array types for fixed-size arrays.

PHP-compatible source keeps PHP `[]` syntax and PHP array quirks for compatibility. Under strict Echo semantics, `[]` cannot be used as an associative PHP array; use it only for regular Echo arrays or fixed Echo arrays, and use typed structural objects for record-like data.

```echo
let $payload = ["id" => 1, "name" => "Echo"]
let $user: UserPayload = { id: 1, name: "Echo" }
```

Strict Echo arrays are contiguous, index-addressable arrays, not associative maps. Use `array<T>` for regular arrays, `array<T>[N]` for fixed arrays, and `list<T>` for Echo lists.

```echo
let $ids: array<int> = [1, 2, 3]
let $fixed: array<int>[3] = [1, 2, 3]
let $list: list<int> = {1, 2, 3}
```

Maps, dictionaries, sets, ordered maps, and similar containers are standard library types, not core literal forms. The runtime may provide efficient backing data structures for those stdlib containers without adding dedicated syntax or special semantic lowering.

Import std container types as normal exported symbols from `std.containers`. Use direct imports for one type and grouped imports for several.

```echo
use std.containers.Set
from std.containers use Dict, Map, Set
```

Use explicit commas between literal elements and fields in both single-line and multiline literals. Multiline literals may use a trailing comma.

```echo
let $user = {
    name: "Echo",
    email: "echo@example.com",
}

let $ids = {
    1,
    2,
    3,
}
```

Type object fields are declarations, not runtime literal fields, so they stay newline-separated without commas.

Parentheses without commas are grouping. A one-element tuple requires a trailing comma.

```echo
let $value = (1)
let $single = (1,)
let $pair = (1, "Echo")
```

An untyped `{}` literal is an empty list. A typed `{}` literal may be an empty structural object only when the target object type permits all fields to be omitted.

```echo
let $items = {}
let $options: Options = {}
```

The `Options` example is valid only if every required field is provided by defaults or marked optional.

Optional fields and nullable values are distinct. A `?` after the field name means the field may be omitted; a `?` in the type means the value may be null.

```echo
type Options = {
    timeout?: int
    label: ?string
    description?: ?string
}
```

Here `timeout` may be omitted, `label` is required but may be null, and `description` may be omitted or present with a string or null value.

Object literals may use shorthand for simple variables. The field name is the variable name without `$`.

```echo
let $name = "Echo"
let $email = "echo@example.com"
let $user = { $name, $email }
```

This is equivalent to:

```echo
let $user = { name: $name, email: $email }
```

Only simple variables use shorthand. Expression fields stay explicit.

```echo
let $user = {
    name: $name.trim(),
    id: $request->id,
}
```

## Imports

Canonical Echo files order top-level syntax as module declaration, semantics declaration, compile declaration, imports, declarations, then executable statements. `semantics` is file-wide and prelude-only: it appears after `module` and before `compile`, imports, declarations, and executable statements. A perfect format run should keep `use ...` imports and `from ... use ...` imports in separate blocks, and should separate std, relative, and package imports within those forms.

Each file may have at most one `semantics` block. If a future file needs multiple semantic flags, they belong in the same block.

Semantic options are bare flags, one per line.

```echo
semantics {
    strict
}
```

Do not use strings or key/value pairs for mode flags unless a future semantic option actually needs a value.

A `semantics` block after `compile`, imports, declarations, or executable statements is invalid.

```echo
module app.http.router

semantics {
    strict
}

compile {
    "./routes/*.php"
}

use std.http
use app.support.request_id
use illuminate.routing.Router

from std use time
from "./contracts.echo" use Middleware
from illuminate.http use Request, Response

type RouteConfig = {
    method: string
    path: string
}

fn handle($request) {
    return http.responseText("ok")
}
```

This order puts source policy and graph-shaping declarations before name binding, then puts reusable declarations before executable code.

Within each import form, canonical grouping is std imports first, relative path imports second, and package/module imports third. Filesystem paths use quoted strings such as `"./contracts.echo"`; bare module-name syntax uses the module system. Absolute host-path imports are accepted for edge cases but discouraged in package code because they make source less portable.

Direct `use ...` imports are for module and package identities, not filesystem paths. File-path imports use the grouped form.

```echo
from "./support/request_id.echo" use request_id
```

When a file declares a module, prefer importing that module by identity.

```echo
use app.support.request_id
```

Package imports use canonical Echo module identity, not Composer package-name spelling. Package names such as `"vendor/package"` are for acquisition and graph admission, while imports use dot-separated module paths supplied by package metadata.

```echo
use illuminate.console.Command
from illuminate.console use Command, InputOption
```

The final segment disambiguates module imports from exported symbols by canonical naming. Lowercase `snake_case` final segments bind modules, while `PascalCase` or other exported member names bind declarations exported from an Echo module or PHP namespace.

```echo
use std.process
use illuminate.console.Command
```

Here `process` is a module binding and `Command` is an exported symbol binding. PHP namespace imports follow the same exported-symbol rule; `PascalCase\PascalCase` spelling is a namespace plus member path, not an Echo module path.

Prefer direct `use` when importing one symbol.

```echo
use illuminate.console.Command
```

Use grouped `from ... use ...` imports when importing multiple symbols from the same module.

```echo
from illuminate.console use Command, InputOption, OutputStyle
```

Aliases are allowed for symbol conflicts or clearer local names.

```echo
use illuminate.console.Command as LaravelCommand
from illuminate.console use Command, InputOption as Option
```

Whole-module imports bind the module under its final segment, or under an explicit alias.

```echo
use std.process
use std.time as clock

process.run("php", {"--version"})
clock.sleep(100)
```

This keeps single-symbol imports compact while keeping larger imports scannable.

## Compile Declarations

Put `compile` declarations after the optional module or namespace declaration and before executable statements. Prefer one string entry per line.

Each file may have at most one `compile` block. Put every graph admission entry for that file in the same block.

A perfect format run groups `compile` entries by relative paths, absolute paths, then packages, and sorts entries lexicographically within each group. Absolute host paths are accepted but discouraged for portable package code.

```echo
module app.bootstrap

compile {
    "./routes/*.php"
    "./plugins/**/*.php"
    "modoterra/laravel-echo"
}

let $target = __DIR__ . "/routes/web.php"
require $target
```

The declaration makes dynamic include targets part of the closed compilation graph before execution.

## Variables And Inference

Echo variables keep PHP's `$` sigil permanently. Use `let` with inference for Echo examples, and omit semicolons unless the documented mode specifically requires PHP syntax.

```echo
let $name = "Echo"
let $port = 8080

echo $name . " listening on " . $port
```

This shows the current Echo surface directly instead of mixing in older typed-variable sketches or PHP-only statement style.

Do not remove `$` from local or runtime variables in Echo-native syntax.

Use `let` for first binding and plain assignment for rebinding.

```echo
let $count = 0
$count = $count + 1
```

Under strict Echo semantics, assignment to an unbound variable is invalid; PHP-compatible source may still use assignment-created variables.

Typed `let` bindings always put the type after the variable name.

```echo
let $users: list<User> = {}
let $count: int = 0
let $user: ?User = null
```

Do not write prefix-typed `let` bindings.
