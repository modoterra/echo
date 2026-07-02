# PHP 8.5 Support Status

This document tracks Echo's compatibility progress against PHP 8.5. It is a
planning checklist, not a release promise. Detailed function-by-function status
lives in [`docs/compat.md`](compat.md).

Compatibility target: PHP 8.5, as recorded in [`CONTEXT.md`](../CONTEXT.md).

Primary upstream references:

- PHP 8.5 release page: https://www.php.net/releases/8.5/en.php
- PHP 8.5 migration guide: https://www.php.net/manual/en/migration85.php
- PHP 8 changelog: https://www.php.net/ChangeLog-8.php
- PHP.Watch 8.5 overview: https://php.watch/versions/8.5

## Status Legend

- `[x]` Supported through parse/AST plus runtime/codegen where applicable.
- `[~]` Partially supported; see notes for the missing layer.
- `[ ]` Not implemented or not yet tracked by a fixture.
- `[n/a]` Out of scope for Echo's current compatibility baseline.

## Current Snapshot

- Function inventory source: local PHP `8.5.6` snapshot in
  [`docs/compat.md`](compat.md).
- PHP compatibility fixtures: 314 `tests/php/*/program.php` files.
- Echo fixtures: 90 `tests/echo/*/program.echo` files.
- Core + standard PHP functions in inventory: 607.
- Implemented Core + standard functions in inventory: 354.
- Remaining Core + standard functions in inventory: 253.

## Estimated Completion

Overall PHP 8.5 compatibility estimate: **about 20% complete**.

This is a rough engineering estimate, not a mechanically exact score. Function
coverage alone is `354 / 607`, or about 58%, for the Core + standard baseline,
but language compatibility is weighted lower because many syntax forms parse
without executable semantics yet. The estimate uses this model:

| Area | Weight | Current estimate | Notes |
| --- | ---: | ---: | --- |
| Syntax and AST coverage | 25% | ~46% | Many PHP declarations and statements parse, but expression grammar and PHP 8.5-specific forms still have gaps. |
| Semantic analysis and lowering | 25% | ~10% | Most PHP-specific declarations, objects, references, constants, and call semantics are not executable end to end. |
| Runtime behavior and built-ins | 35% | ~35% | Core + standard function coverage is 354/607, with deeper object/error/extension behavior still missing. |
| Tooling, diagnostics, and fixtures | 15% | ~15% | Fixture coverage is growing, but compatibility diagnostics and broad real-world app coverage are still early. |

Treat this as a prioritization signal: Echo has a meaningful parser/runtime
base, but it is not close to full PHP 8.5 compatibility until classes,
functions, constants, callables, references, exceptions, and PHP 8.5 headline
features run through the shared pipeline.

## Common Implementation Gaps

Track recurring blockers here when a missing language/runtime capability prevents
full support for an entire family of PHP functions.

- Ternary expressions parse, but do not lower through LLVM codegen yet. This
  blocks using idiomatic PHP conditional expressions in end-to-end fixtures.
- Output buffering stores callback metadata, but does not invoke output
  callbacks or render callback-specific handler names yet. This keeps
  `ob_start()` callback behavior, `ob_list_handlers()`, `ob_get_status()`, and
  related handler status APIs partial.
- URL rewriting for output buffers is not modeled yet. This keeps
  `output_add_rewrite_var()`, `output_reset_rewrite_vars()`, and the
  `URL-Rewriter` output handler partial or missing.
- PHP warning/error emission is not modeled as observable runtime state yet.
  This keeps many failure paths partial even when their return values match PHP.

## Fixture Workflow

Start every user-observable PHP compatibility item with a PHP fixture and let
system PHP define the expected bytes:

```sh
fixture=tests/php/191_pipe_operator
mkdir -p "$fixture"
printf '%s\n' '<?php echo " PHP 8.5 " |> trim(...) |> strtoupper(...);' > "$fixture/program.php"
: > "$fixture/stdin.txt"
php "$fixture/program.php" < "$fixture/stdin.txt" > "$fixture/stdout.txt"
scripts/check-fast parser 191_pipe_operator
scripts/check-fast fixture 191_pipe_operator
```

Use the parser check first when the slice is syntax-only. Use the fixture check
when the behavior must pass through `xo ast`, `xo ir`, `xo run`, and `xo build`.

## PHP 8.5 Headline Features

### Syntax and Language Semantics

- `[~]` Pipe operator `|>`.
  - Parser/AST support preserves pipe expressions as `BinaryOp::Pipe`.
  - Covered by parser-only fixture `tests/php/192_pipe_operator`.
  - TODO: lower to callable invocation semantics rather than stringing
    together nested call syntax.
  - TODO: cover first-class callables, closures, named functions, and error
    cases end to end.

- `[~]` Clone expressions.
  - Existing status: parser/AST supports unary `clone`.
  - Parser/AST support covers PHP 8.5
    `clone($object, ["property" => $value])`.
  - Covered by parser-only fixture `tests/php/193_clone_with`.
  - TODO: implement object-copy semantics, `__clone()` dispatch, readonly
    property update rules, and diagnostics for invalid property updates.

- `[~]` `(void)` cast.
  - Parser/AST support records it as a PHP-specific cast form.
  - TODO: lower to value discard for `#[\NoDiscard]` suppression.
  - Keep deprecated non-canonical casts separately tracked.

- `[ ]` `#[\NoDiscard]` predefined attribute.
  - Parse already covered by generic attributes.
  - TODO: recognize the predefined attribute symbol.
  - TODO: warn when a marked function or method return value is unused.
  - TODO: treat `(void) marked_call()` as explicit discard.

- `[~]` Constant expressions.
  - Existing status: several constant-expression surfaces parse, but constant
    evaluation is not complete.
  - TODO: allow static closures in constant expressions.
  - TODO: allow casts in constant expressions.
  - TODO: allow first-class callables in constant expressions.
  - TODO: prove attribute arguments, defaults, class constants, and property
    defaults through shared constant evaluation.

- `[~]` Ternary operator.
  - Existing status: parser/AST accepts PHP ternary expressions.
  - TODO: lower ternary expressions through semantic analysis, MIR, and LLVM
    codegen so compatibility fixtures can use conditional expressions end to
    end.

- `[~]` Attributes.
  - Existing status: parser/AST supports PHP attribute syntax on major
    declaration targets.
  - TODO: constants as attribute targets for PHP 8.5.
  - TODO: predefined attributes `NoDiscard` and `DelayedTargetValidation`.
  - TODO: PHP 8.5 target expansion for `Override` on properties.
  - TODO: PHP 8.5 target expansion for `Deprecated` on traits and constants.

- `[~]` Static properties and promoted properties.
  - Existing status: class/property parser support is partial.
  - TODO: asymmetric visibility for static properties.
  - TODO: `final` promoted properties.
  - TODO: validation and runtime behavior for property hooks and visibility.

- `[~]` Fatal error backtraces.
  - Existing status: diagnostics and runtime errors are still Echo-owned
    shapes, not PHP stack traces.
  - TODO: carry call stack information through runtime fatal errors.
  - TODO: format fatal output compatibly for PHP fixture comparisons.

### New Core and Standard Functions

- `[x]` `array_first(array $array): mixed`.
  - Returns the first insertion-order value or `null` for an empty array.
  - Covered by `tests/php/128_array_lookup_builtins`.

- `[x]` `array_last(array $array): mixed`.
  - Returns the last insertion-order value or `null` for an empty array.
  - Covered by `tests/php/128_array_lookup_builtins`.

- `[~]` `get_error_handler(): ?callable`.
  - Returns `null` before a custom handler is installed and returns the
    current stored handler after `set_error_handler()`.
  - Covered by `tests/php/195_get_error_handler_initial` and
    `tests/php/197_error_handler_registry`.
  - TODO: dispatch PHP errors through the stored handler.

- `[~]` `get_exception_handler(): ?callable`.
  - Returns `null` before a custom handler is installed and returns the
    current stored handler after `set_exception_handler()`.
  - Covered by `tests/php/196_get_exception_handler_initial` and
    `tests/php/198_exception_handler_registry`.
  - TODO: dispatch uncaught exceptions through the stored handler.

- `[ ]` `Closure::getCurrent()`.
  - Requires first-class closure object/runtime identity.
  - Should not be faked in the REPL or codegen.

### New Predefined Constants

- `[x]` `PHP_BUILD_DATE`.
  - Echo exposes a stable PHP-shaped compatibility value.
  - Covered by `tests/php/194_php_build_date_constant`.

- `[ ]` `PHP_BUILD_PROVIDER`.
  - PHP documents this as build-dependent and it is absent from the local PHP
    8.5 build when no provider is configured.
  - TODO: decide whether Echo omits it for parity with source-built PHP or
    exposes an Echo-owned provider value.

### URI Extension

- `[ ]` Built-in `Uri` extension.
  - PHP 8.5 includes an always-available URI extension for RFC 3986 and WHATWG
    URL handling.
  - TODO: decide whether this belongs in Echo PHP Surface, Echo `std.uri`, or
    both.
  - TODO: model namespaces/classes such as `Uri\Rfc3986\Uri` before adding
    runtime-only helpers.
  - TODO: choose Rust URI/URL parser crates and document any observable
    differences from PHP's uriparser/Lexbor-backed behavior.

### Extension and SAPI Surfaces

- `[n/a]` Persistent cURL share handles.
  - Echo does not currently model cURL. Track only if cURL becomes part of the
    supported PHP extension surface.

- `[n/a]` `IntlListFormatter`, `locale_is_right_to_left()`,
  `Locale::isRightToLeft()`, and `grapheme_levenshtein()`.
  - These belong to optional intl/grapheme surfaces unless Echo promotes them
    into the baseline.

- `[n/a]` DOM additions such as `Dom\Element::getElementsByClassName()` and
  `Dom\Element::insertAdjacentHTML()`.
  - Track under DOM extension support, not the core baseline.

- `[n/a]` CLI-only changes such as `php --ini=diff`.
  - Echo's CLI is `xo`; only mirror this if a PHP-compatible CLI mode becomes
    a product goal.

## PHP 8.5 Deprecations and Compatibility Breaks

- `[ ]` Deprecated pipe-adjacent syntax policy.
  - Backtick operator as `shell_exec()` alias is deprecated upstream.
  - TODO: decide whether Echo warns, accepts silently, or rejects in a future
    PHP-compat lint profile.

- `[ ]` Non-canonical scalar casts.
  - PHP 8.5 deprecates `(boolean)`, `(integer)`, `(double)`, and `(binary)`.
  - TODO: parse and represent these as PHP-specific casts.
  - TODO: emit compatibility deprecation diagnostics separately from canonical
    `(bool)`, `(int)`, `(float)`, and `(string)`.

- `[~]` Switch case separators.
  - Existing status: parser accepts `case expr:` and `case expr;`.
  - TODO: emit PHP 8.5 deprecation diagnostic for semicolon case separators.

- `[ ]` Null array offset deprecation.
  - TODO: preserve current PHP behavior where required, then add warning
    diagnostics for `null` offsets and `array_key_exists(null, ...)`.

- `[ ]` Class alias reserved names.
  - TODO: reject or warn for `class_alias()` to `array` and `callable` once
    class alias semantics exist.

- `[ ]` `__sleep()` and `__wakeup()` soft deprecation.
  - TODO: track once serialization magic methods exist.

- `[ ]` Numeric cast warnings.
  - TODO: warn when casting `NAN` to other types.
  - TODO: warn when float or float-like string to int cannot be represented.

- `[ ]` Destructuring warning updates.
  - TODO: warn when destructuring non-array, non-null values using `[]` or
    `list()`.

## Existing PHP Surface Priorities

### Parser/AST

- `[~]` PHP tags and inline HTML.
- `[~]` `declare`.
- `[~]` attributes.
- `[~]` class, trait, interface, enum, and anonymous class declarations.
- `[~]` property hooks and modifiers.
- `[~]` control flow statements including `if`, loops, `switch`, `goto`.
- `[~]` `global`, function-scope `static`, and assignment expressions.
- `[ ]` Complete PHP expression grammar including all precedence corner cases.
- `[ ]` Complete PHP type grammar including class-string-like PHPDoc boundaries.
- `[ ]` Namespace/import resolution parity through `echo_resolver`.

### Semantics, HIR, MIR, and Codegen

- `[ ]` Execute parsed PHP control flow constructs end to end.
- `[ ]` Execute user-defined functions and methods with PHP call semantics.
- `[ ]` Implement classes, inheritance, traits, interfaces, properties, and
  method dispatch.
- `[ ]` Implement object identity, cloning, readonly behavior, visibility, and
  magic methods.
- `[ ]` Implement constants and constant lookup.
- `[ ]` Implement references where PHP observable behavior requires them.
- `[ ]` Implement closures, first-class callables, and callable resolution.
- `[ ]` Implement exceptions and PHP error levels.
- `[ ]` Move project-wide namespace/package resolution into `echo_resolver`.

### Runtime Builtins

- `[~]` Core + standard function inventory is tracked in `docs/compat.md`.
- `[ ]` Raise implemented baseline count beyond 237/607.
- `[ ]` Prioritize missing functions that unblock common Laravel/Symfony style
  bootstrap code: class/reflection helpers, error handlers, include metadata,
  environment/config helpers, and array mutation helpers.
- `[ ]` Keep optional extension surfaces explicit rather than silently claiming
  support through stubs.

## Next Useful Slices

1. `NoDiscard` diagnostics for explicit discard.
   - Recognize the predefined attribute symbol.
   - Treat `(void) marked_call()` as explicit discard.

2. Pipe operator lowering.
   - Invoke the right-hand callable with the left-hand value.
   - Support first-class callable placeholder syntax such as `trim(...)`.
   - Keep AST truthful: do not lower to nested calls inside the parser.

3. Clone-with runtime semantics.
   - Copy the source object, apply property updates, dispatch `__clone()`, and
     enforce visibility/readonly update rules.

4. Constant expression evaluator.
   - Needed for PHP 8.5 closure/cast/callable constant expressions and for
     attribute correctness.
