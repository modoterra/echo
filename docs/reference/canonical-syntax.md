# Canonical Echo Syntax

Echo accepts PHP-compatible source forms, but documentation, examples, generated code, and formatter output should prefer the canonical Echo forms below.

## File And Module Names

Echo source files use `snake_case.echo` file names, and directories use lowercase names with underscores when needed. Echo-native package files declare lower-dot module names.

```echo
module modoterra.laravel_echo.console

let $command_name = "echo:start"
```

This keeps Echo package structure readable without carrying PHP class-file naming conventions into Echo source.

## Types And Declarations

Classes, traits, interfaces, enums, and type names use `PascalCase`. Functions, variables, modules, and file/module identifiers use `snake_case`.

```echo
module app.console

class StartServerCommand {
    pub function handle($server_name) {
        echo $server_name
    }
}

function normalize_path($input) {
    return $input
}
```

This style separates type-like names from value-like names and keeps Echo examples consistent across packages.

## Imports

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

Use `let` with inference for Echo examples. Omit semicolons unless the documented mode specifically requires PHP syntax.

```echo
let $name = "Echo"
let $port = 8080

echo $name . " listening on " . $port
```

This shows the current Echo surface directly instead of mixing in older typed-variable sketches or PHP-only statement style.
