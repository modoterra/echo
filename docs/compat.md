# Echo PHP Surface Inventory

This document tracks Echo PHP Surface function parity at function granularity.
It is a planning inventory, not a promise that all rows have equal
implementation cost. Some functions are syntax-adjacent, runtime-global,
filesystem/process backed, or require classes/resources before they can be
implemented correctly.

Snapshot source: `get_defined_functions()["internal"]` from local PHP `8.5.6`
on 2026-06-17. PHP reports many ordinary-looking functions through the
`standard` extension; Echo treats `Core` plus `standard` as the baseline
surface and keeps other PHP extensions optional unless we explicitly promote
one.

Echo's PHP compatibility target is locked to PHP 8.5. PHP syntax or runtime
features introduced after PHP 8.5 are out of scope unless Echo adopts them as
Echo-native extensions.

## Syntax-Adjacent Compatibility

These PHP constructs are not reported by `get_defined_functions()`, but they
belong in the same Echo PHP Surface planning area because they interact with
filesystem loading, runtime process state, diagnostics, and related functions.

### Include/Require Family

| Construct | Status | Notes |
| --- | --- | --- |
| `include` | missing | Language construct. Returns `false` and raises a warning on failure; successful includes return `1` unless the included file returns another value. Source: https://www.php.net/manual/en/function.include.php |
| `include_once` | missing | Language construct. Includes and evaluates a file once per process include set; returns `true` when the file was already included. Source: https://www.php.net/manual/en/function.include-once.php |
| `require` | partial | Language construct. Echo now checks required files and treats missing files as fatal, but does not evaluate loaded PHP source yet. Source: https://www.php.net/manual/en/function.require.php |
| `require_once` | partial | Echo now tracks a process-local once set and checks required files, but does not evaluate loaded PHP source yet. Source: https://www.php.net/manual/en/function.require-once.php |

Related baseline functions tracked below: `get_included_files`,
`get_required_files`, `get_include_path`, `set_include_path`,
`restore_include_path`, and `stream_resolve_include_path`.

### Bootstrap Syntax

| Construct | Status | Notes |
| --- | --- | --- |
| attributes | partial | Parser/AST support covers PHP 8 `#[...]` attributes on classes, traits, interfaces, enums, enum cases, functions, methods, parameters, properties, and class/interface constants, including grouped attributes and argument lists. Reflection exposure and target validation are deferred. Source: https://www.php.net/manual/en/language.attributes.overview.php |
| assignment expressions | partial | Assignments are expressions and evaluate to the assigned value. Source: https://www.php.net/manual/en/language.operators.assignment.php |
| `__DIR__` | partial | File-backed `xo` compilation resolves it from the canonical source path. Source: https://www.php.net/manual/en/language.constants.magic.php |
| anonymous classes | partial | Parser/AST support covers `new class`, constructor arguments, `readonly class`, `extends`, `implements`, trait-use members, properties, constants, and methods. Runtime anonymous class identity/object semantics are not implemented yet. Source: https://www.php.net/manual/en/language.oop5.anonymous.php |
| class and method modifiers | partial | Parser/AST support covers `abstract`, `final`, and `readonly` class modifiers plus `abstract` and `final` method modifiers. Parser/AST support also covers PHP 8.4 property hooks on properties; abstract/final property validation and runtime hook semantics are still deferred. Sources: https://www.php.net/manual/en/language.oop5.abstract.php, https://www.php.net/manual/en/language.oop5.final.php, https://www.php.net/manual/en/language.oop5.basic.php, and https://www.php.net/manual/en/language.oop5.property-hooks.php |
| `clone` | partial | Parser/AST support covers the `clone` unary expression and member access on a parenthesized clone expression. Runtime object-copy and `__clone()` semantics are not implemented yet. Source: https://www.php.net/manual/en/language.oop5.cloning.php |
| `declare` | partial | Parser/AST support covers `declare(directive=value);`, braced `declare(directive=value) { ... }` bodies, a single following statement, and alternate `declare(directive=value): ... enddeclare;` bodies with comma-separated directives. Literal-only directive validation and runtime directive behavior are deferred. Source: https://www.php.net/manual/en/control-structures.declare.php |
| `define` | partial | Echo accepts runtime constant definitions for bootstrap compatibility, but constant lookup is not implemented yet. Source: https://www.php.net/manual/en/function.define.php |
| `do-while` statements | partial | Parser/AST support covers PHP's `do { ... } while (expr);` loop syntax with block bodies and parenthesized conditions. Runtime loop execution is deferred. Source: https://www.php.net/manual/en/control-structures.do.while.php |
| enum declarations | partial | Parser/AST support covers pure enums, backed enums, implemented interfaces, cases, methods, and trait-use members. Runtime enum object semantics are not implemented yet. Sources: https://www.php.net/manual/en/language.enumerations.php and https://www.php.net/manual/en/language.enumerations.backed.php |
| `exit`/`die` | partial | Parser/AST support covers bare `exit;`/`die;`, expression forms such as `exit 1;`, and parenthesized forms such as `die("done");`. Codegen exits through Echo's existing shutdown path; full PHP output/status edge cases are deferred. Source: https://www.php.net/manual/en/function.exit.php |
| `for` statements | partial | Parser/AST support covers standard parenthesized `for` loops with empty or comma-separated initialization, condition, and increment expression segments plus braced and alternate `for (...): ... endfor;` bodies. Runtime loop execution is deferred. Source: https://www.php.net/manual/en/control-structures.for.php |
| `foreach` statements | partial | Parser/AST support covers `foreach (iterable as $value)`, `foreach (iterable as &$value)`, and `foreach (iterable as $key => &$value)` forms with braced and alternate `foreach (...): ... endforeach;` bodies. Destructuring targets, iterable diagnostics, runtime reference binding, and runtime iteration are deferred. Source: https://www.php.net/manual/en/control-structures.foreach.php |
| `goto` and labels | partial | Parser/AST support covers `goto label;` and case-sensitive `label:` targets. Same-file/context restrictions and runtime control transfer are deferred. Source: https://www.php.net/manual/en/control-structures.goto.php |
| `global` and static variables | partial | Parser/AST support covers `global $a, $b` and function-scope `static $a = expr, $b` declarations, including PHP 8.3+ dynamic static initializers. Runtime global binding and static persistence semantics are not implemented yet. Source: https://www.php.net/manual/en/language.variables.scope.php |
| `if` statements | partial | Parser/AST support covers braced `if`/`elseif`/`else` and alternate `if (...): ... elseif (...): ... else: ... endif;` bodies. Split `else if` remains accepted for braced syntax only, matching PHP's colon-syntax restriction. Runtime conditional execution is deferred. Sources: https://www.php.net/manual/en/control-structures.elseif.php and https://www.php.net/manual/en/control-structures.alternative-syntax.php |
| interface declarations | partial | Parser/AST support covers interface declarations, multiple parent interfaces, method signatures, constants, and PHP 8.4 property declarations with `get`/`set` hooks. Runtime property-hook contract enforcement is deferred. Sources: https://www.php.net/manual/en/language.oop5.interfaces.php and https://www.php.net/manual/en/language.oop5.property-hooks.php |
| `microtime` | implemented | Supports string and float forms for current wall-clock time. Source: https://www.php.net/manual/en/function.microtime.php |
| `print` | partial | Parser/AST and codegen support covers `print` as an expression that writes one value and returns `1`. Source: https://www.php.net/manual/en/function.print.php |
| PHP tags | partial | Parser/AST support covers `<?php` open tags, final `?>` close tags as statement terminators, interleaved inline HTML chunks as `PhpInlineHtml` output statements, and standalone or interleaved `<?= expr ?>` short echo tags lowered to `EchoStmt`. More complex template whitespace parity and non-output runtime edge cases are deferred. Sources: https://www.php.net/manual/en/language.basic-syntax.phptags.php and https://www.php.net/manual/en/language.basic-syntax.instruction-separation.php |
| `switch` statements | partial | Parser/AST support covers braced and alternate `switch (...): ... endswitch;` statements with expression `case` labels, `default`, empty fall-through case bodies, and both `:` and `;` label separators. Runtime execution is deferred. Source: https://www.php.net/manual/en/control-structures.switch.php |
| `while` statements | partial | Parser/AST support covers parenthesized `while` conditions with braced and alternate `while (...): ... endwhile;` bodies. Runtime loop execution is deferred. Source: https://www.php.net/manual/en/control-structures.while.php |

## Totals

| Surface | Functions | Implemented | Remaining |
| --- | ---: | ---: | ---: |
| Baseline (`Core` + `standard`) | 607 | 205 | 402 |
| Loaded local PHP internals, including extensions | 1516 | 205 | 1311 |

## Baseline Functions

### Core (6/62)

| Function | Status | Notes |
| --- | --- | --- |
| `class_alias` | missing |  |
| `class_exists` | partial | Returns `false` until Echo has class metadata and autoload semantics. Source: https://www.php.net/manual/en/function.class-exists.php |
| `clone` | missing |  |
| `debug_backtrace` | partial | Returns an empty array for the top-level/no-frame baseline; stack frame capture plus `options` and `limit` handling are deferred. Source: https://www.php.net/manual/en/function.debug-backtrace.php |
| `debug_print_backtrace` | partial | Prints nothing for the top-level/no-frame baseline; stack frame rendering plus `options` and `limit` handling are deferred. Source: https://www.php.net/manual/en/function.debug-print-backtrace.php |
| `define` | missing |  |
| `defined` | partial | Recognizes compiler-owned PHP compatibility constants that Echo already lowers; user-defined constants from `define()` are deferred. Source: https://www.php.net/manual/en/function.defined.php |
| `die` | missing |  |
| `enum_exists` | partial | Returns `false` until Echo has enum metadata and autoload semantics. Source: https://www.php.net/manual/en/function.enum-exists.php |
| `error_reporting` | partial | Stores and returns the process-local reporting level; integration with emitted PHP warnings/notices is deferred. Source: https://www.php.net/manual/en/function.error-reporting.php |
| `exit` | missing |  |
| `extension_loaded` | implemented | Accepts extension names and returns `false` because Echo does not model PHP extensions yet. Source: https://www.php.net/manual/en/function.extension-loaded.php |
| `func_get_arg` | missing |  |
| `func_get_args` | missing |  |
| `func_num_args` | missing |  |
| `function_exists` | implemented | Recognizes Echo's supported internal Echo PHP Surface function names, case-insensitively; user-defined function registry support is deferred. Source: https://www.php.net/manual/en/function.function-exists.php |
| `gc_collect_cycles` | implemented | Returns `0` because Echo does not expose PHP cyclic-GC internals. Source: https://www.php.net/manual/en/function.gc-collect-cycles.php |
| `gc_disable` | partial | Toggles Echo's PHP GC compatibility flag off; no engine collector behavior is changed. Source: https://www.php.net/manual/en/function.gc-disable.php |
| `gc_enable` | partial | Toggles Echo's PHP GC compatibility flag on; no engine collector behavior is changed. Source: https://www.php.net/manual/en/function.gc-enable.php |
| `gc_enabled` | partial | Reports Echo's PHP GC compatibility flag. Source: https://www.php.net/manual/en/function.gc-enabled.php |
| `gc_mem_caches` | implemented | Returns `0` because Echo does not expose PHP allocator cache internals. Source: https://www.php.net/manual/en/function.gc-mem-caches.php |
| `gc_status` | partial | Returns PHP's expected status keys with Echo-owned baseline values. Source: https://www.php.net/manual/en/function.gc-status.php |
| `get_called_class` | missing |  |
| `get_class` | missing |  |
| `get_class_methods` | missing |  |
| `get_class_vars` | missing |  |
| `get_declared_classes` | partial | Returns an empty array until Echo has runtime class metadata. Source: https://www.php.net/manual/en/function.get-declared-classes.php |
| `get_declared_interfaces` | partial | Returns an empty array until Echo has runtime interface metadata. Source: https://www.php.net/manual/en/function.get-declared-interfaces.php |
| `get_declared_traits` | partial | Returns an empty array until Echo has runtime trait metadata. Source: https://www.php.net/manual/en/function.get-declared-traits.php |
| `get_defined_constants` | partial | Returns Echo's compiler-owned PHP compatibility constants, optionally categorized under `Core`; dynamically defined constants and full extension inventories are deferred. Source: https://www.php.net/manual/en/function.get-defined-constants.php |
| `get_defined_functions` | partial | Returns supported PHP builtin names under `internal` and compiler-registered userland names under `user`; disabled-function filtering and full PHP internal inventory parity are deferred. Source: https://www.php.net/manual/en/function.get-defined-functions.php |
| `get_defined_vars` | missing |  |
| `get_error_handler` | partial | Returns the current stored error-handler callback or `null` when none is installed; dispatching PHP errors through the stored callback is deferred. Source: https://www.php.net/manual/en/function.get-error-handler.php |
| `get_exception_handler` | partial | Returns the current stored exception-handler callback or `null` when none is installed; dispatching uncaught exceptions through the stored callback is deferred. Source: https://www.php.net/manual/en/function.get-exception-handler.php |
| `get_extension_funcs` | implemented | Returns `false` for named extensions because Echo does not model PHP extension function metadata yet. Source: https://www.php.net/manual/en/function.get-extension-funcs.php |
| `get_included_files` | partial | Returns the entry script path for file-based CLI programs; tracking evaluated include/require paths is deferred. Source: https://www.php.net/manual/en/function.get-included-files.php |
| `get_loaded_extensions` | implemented | Returns an empty array because Echo does not model PHP extension metadata yet; accepts the optional Zend-extension flag. Source: https://www.php.net/manual/en/function.get-loaded-extensions.php |
| `get_mangled_object_vars` | missing |  |
| `get_object_vars` | missing |  |
| `get_parent_class` | missing |  |
| `get_required_files` | partial | Alias of `get_included_files()` for the current include metadata baseline. Source: https://www.php.net/manual/en/function.get-required-files.php |
| `get_resource_id` | partial | Returns a stable integer handle for Echo runtime resources; PHP's exact per-process resource numbering is not modeled. Source: https://www.php.net/manual/en/function.get-resource-id.php |
| `get_resource_type` | partial | Returns Echo's runtime resource type name such as `stream`; PHP's full resource type catalog is deferred. Source: https://www.php.net/manual/en/function.get-resource-type.php |
| `get_resources` | partial | Returns an empty array for the current resource enumeration baseline; live resource registry and invalid-type `ValueError` parity are deferred. Source: https://www.php.net/manual/en/function.get-resources.php |
| `interface_exists` | partial | Returns `false` until Echo has interface metadata and autoload semantics. Source: https://www.php.net/manual/en/function.interface-exists.php |
| `is_a` | partial | Returns `false` until Echo has class identity and inheritance metadata. Source: https://www.php.net/manual/en/function.is-a.php |
| `is_subclass_of` | partial | Returns `false` until Echo has class/interface inheritance metadata. Source: https://www.php.net/manual/en/function.is-subclass-of.php |
| `method_exists` | partial | Returns `false` until Echo has runtime method metadata. Source: https://www.php.net/manual/en/function.method-exists.php |
| `property_exists` | partial | Returns `false` until Echo has runtime property metadata. Source: https://www.php.net/manual/en/function.property-exists.php |
| `restore_error_handler` | partial | Restores the previous stored error-handler callback and returns `true`; dispatching PHP errors through the stored callback is deferred. Source: https://www.php.net/manual/en/function.restore-error-handler.php |
| `restore_exception_handler` | partial | Restores the previous stored exception-handler callback and returns `true`; dispatching uncaught exceptions through the stored callback is deferred. Source: https://www.php.net/manual/en/function.restore-exception-handler.php |
| `set_error_handler` | partial | Stores callable error handlers, returns the previous handler or `null`, and cooperates with `get_error_handler()`/`restore_error_handler()`. Actual error dispatch is deferred. Source: https://www.php.net/manual/en/function.set-error-handler.php |
| `set_exception_handler` | partial | Stores callable exception handlers, returns the previous handler or `null`, and cooperates with `get_exception_handler()`/`restore_exception_handler()`. Actual exception dispatch is deferred. Source: https://www.php.net/manual/en/function.set-exception-handler.php |
| `strcasecmp` | implemented | Performs binary-safe ASCII case-insensitive string comparison and returns the PHP-style ordering sign. Source: https://www.php.net/manual/en/function.strcasecmp.php |
| `strcmp` | implemented | Performs binary-safe string comparison and returns the PHP-style ordering sign. Source: https://www.php.net/manual/en/function.strcmp.php |
| `strlen` | implemented | Returns byte length for strings, matching PHP's byte-oriented string model rather than character count. Source: https://www.php.net/manual/en/function.strlen.php |
| `strncasecmp` | implemented | Source: https://www.php.net/manual/en/function.strncasecmp.php |
| `strncmp` | implemented | Source: https://www.php.net/manual/en/function.strncmp.php |
| `trait_exists` | partial | Returns `false` until Echo has trait metadata and autoload semantics. Source: https://www.php.net/manual/en/function.trait-exists.php |
| `trigger_error` | partial | Validates message and level and returns `true`; warning emission, custom error-handler dispatch, and omitted-level defaulting are deferred. Source: https://www.php.net/manual/en/function.trigger-error.php |
| `user_error` | partial | Alias of Echo's current `trigger_error` baseline; warning emission, custom error-handler dispatch, and omitted-level defaulting are deferred. Source: https://www.php.net/manual/en/function.user-error.php |
| `zend_version` | implemented | Returns Echo's Zend Engine compatibility version string. Source: https://www.php.net/manual/en/function.zend-version.php |

### standard (175/528)

| Function | Status | Notes |
| --- | --- | --- |
| `abs` | implemented | Supports current Echo integer values; float payloads are deferred. Source: https://www.php.net/manual/en/function.abs.php |
| `acos` | implemented | Returns the arc cosine in radians using PHP-compatible float coercion. Source: https://www.php.net/manual/en/function.acos.php |
| `acosh` | implemented | Returns inverse hyperbolic cosine as a float with PHP-compatible numeric coercion and `NAN` outside the domain. Source: https://www.php.net/manual/en/function.acosh.php |
| `addcslashes` | missing |  |
| `addslashes` | implemented | Escapes single quote, double quote, backslash, and NUL bytes for legacy quoted-string compatibility. Source: https://www.php.net/manual/en/function.addslashes.php |
| `array_all` | missing |  |
| `array_any` | missing |  |
| `array_change_key_case` | partial | Changes string keys to ASCII lowercase or uppercase while preserving integer keys; omitted-case defaulting and non-ASCII casing are deferred. Source: https://www.php.net/manual/en/function.array-change-key-case.php |
| `array_chunk` | implemented | Splits arrays into numerically indexed chunks, optionally preserving original keys inside each chunk; invalid chunk lengths surface as runtime errors. Source: https://www.php.net/manual/en/function.array-chunk.php |
| `array_column` | partial | Extracts values from array rows by integer or string column key, returning numerically reindexed results and skipping rows missing the key; `null` column keys return whole rows, while `index_key` support is deferred. Source: https://www.php.net/manual/en/function.array-column.php |
| `array_combine` | implemented | Creates an array from one array of keys and one array of values, using PHP array-key coercion; duplicate keys keep the latest value, and mismatched input lengths surface as runtime errors. Source: https://www.php.net/manual/en/function.array-combine.php |
| `array_count_values` | implemented | Counts occurrences of int/string values using PHP array-key coercion; unsupported value types are skipped without PHP warning emission for now. Source: https://www.php.net/manual/en/function.array-count-values.php |
| `array_diff` | partial | Compares values against one other array using PHP's string comparison semantics and preserves left-array keys for unmatched values; variadic comparisons are deferred. Source: https://www.php.net/manual/en/function.array-diff.php |
| `array_diff_assoc` | partial | Compares keys and values against one other array using PHP's string value comparison and preserves unmatched left-array entries; variadic comparisons are deferred. Source: https://www.php.net/manual/en/function.array-diff-assoc.php |
| `array_diff_key` | partial | Compares keys against one other array and preserves left-array keys and values that are absent from the right array; variadic comparisons are deferred. Source: https://www.php.net/manual/en/function.array-diff-key.php |
| `array_diff_uassoc` | missing |  |
| `array_diff_ukey` | missing |  |
| `array_fill` | implemented | Creates arrays with repeated values and sequential integer keys starting at the requested index, including PHP 8 negative-key increment behavior; out-of-range counts surface as runtime errors. Source: https://www.php.net/manual/en/function.array-fill.php |
| `array_fill_keys` | implemented | Uses the input values as keys with PHP array-key coercion and fills every key with the requested value. Source: https://www.php.net/manual/en/function.array-fill-keys.php |
| `array_filter` | partial | Supports explicit `null` callback with mode `0`, preserving original keys and filtering by PHP truthiness; callbacks, omitted defaults, and key/both modes are deferred. Source: https://www.php.net/manual/en/function.array-filter.php |
| `array_find` | missing |  |
| `array_find_key` | missing |  |
| `array_first` | implemented | Returns the first insertion-order value, or `null` for an empty array. Source: https://www.php.net/manual/en/function.array-first.php |
| `array_flip` | implemented | Exchanges int/string values with their original keys; duplicate values keep the latest key, while unsupported value-key types are skipped without PHP warning emission for now. Source: https://www.php.net/manual/en/function.array-flip.php |
| `array_intersect` | partial | Compares values against one other array using PHP's string comparison semantics and preserves left-array keys for matched values; variadic comparisons are deferred. Source: https://www.php.net/manual/en/function.array-intersect.php |
| `array_intersect_assoc` | partial | Compares keys and values against one other array using PHP's string value comparison and preserves matched left-array entries; variadic comparisons are deferred. Source: https://www.php.net/manual/en/function.array-intersect-assoc.php |
| `array_intersect_key` | partial | Compares keys against one other array and preserves left-array keys and values that also exist in the right array; variadic comparisons are deferred. Source: https://www.php.net/manual/en/function.array-intersect-key.php |
| `array_intersect_uassoc` | missing |  |
| `array_intersect_ukey` | missing |  |
| `array_is_list` | implemented | Echo PHP arrays are currently contiguous vectors; associative/key-gap arrays are deferred. Source: https://www.php.net/manual/en/function.array-is-list.php |
| `array_key_exists` | implemented | Checks first-dimension keys, including keys whose value is `null`, with PHP array-key coercion for supported scalar keys. Source: https://www.php.net/manual/en/function.array-key-exists.php |
| `array_key_first` | implemented | Returns the first insertion-order key as `int|string`, or `null` for an empty array. Source: https://www.php.net/manual/en/function.array-key-first.php |
| `array_key_last` | implemented | Returns the last insertion-order key as `int|string`, or `null` for an empty array. Source: https://www.php.net/manual/en/function.array-key-last.php |
| `array_keys` | implemented | Returns numeric and string keys, with optional loose or strict value filtering. Source: https://www.php.net/manual/en/function.array-keys.php |
| `array_last` | implemented | Returns the last insertion-order value, or `null` for an empty array. Source: https://www.php.net/manual/en/function.array-last.php |
| `array_map` | missing |  |
| `array_merge` | implemented | Merges any number of arrays, overwriting duplicate string keys while appending and renumbering numeric keys from zero; zero arguments return an empty array. Source: https://www.php.net/manual/en/function.array-merge.php |
| `array_merge_recursive` | missing |  |
| `array_multisort` | missing |  |
| `array_pad` | implemented | Pads copies of arrays to positive lengths on the right or negative lengths on the left; numeric keys are reindexed when padding occurs, while no-op pads preserve keys. Source: https://www.php.net/manual/en/function.array-pad.php |
| `array_pop` | partial | Removes and returns the last value from Echo's mutable PHP array storage; warning behavior for non-arrays and full by-reference diagnostics are deferred. Source: https://www.php.net/manual/en/function.array-pop.php |
| `array_product` | implemented | Multiplies array values with PHP-compatible numeric coercion for current scalar values; empty arrays return `1`. Source: https://www.php.net/manual/en/function.array-product.php |
| `array_push` | partial | Appends one value to Echo's mutable PHP array storage using PHP integer append-key rules and returns the new element count; variadic append and full by-reference diagnostics are deferred. Source: https://www.php.net/manual/en/function.array-push.php |
| `array_rand` | missing |  |
| `array_reduce` | missing |  |
| `array_replace` | implemented | Returns a copy of the first array with right-most replacement values assigned by key, preserving numeric and string keys. Source: https://www.php.net/manual/en/function.array-replace.php |
| `array_replace_recursive` | missing |  |
| `array_reverse` | implemented | Returns elements in reverse order, reindexing numeric keys by default and preserving string keys; optional `preserve_keys` keeps numeric keys too. Source: https://www.php.net/manual/en/function.array-reverse.php |
| `array_search` | implemented | Returns the first key for a matching value using loose comparison by default, optional strict comparison, and `false` on misses. Source: https://www.php.net/manual/en/function.array-search.php |
| `array_shift` | partial | Removes and returns the first value from Echo's mutable PHP array storage, preserving string keys and reindexing remaining integer keys; warning behavior for non-arrays and full by-reference diagnostics are deferred. Source: https://www.php.net/manual/en/function.array-shift.php |
| `array_slice` | implemented | Extracts offset/length windows by array position, reindexing integer keys by default while always preserving string keys. Source: https://www.php.net/manual/en/function.array-slice.php |
| `array_splice` | partial | Removes a positional segment from Echo's mutable PHP array storage, returns the removed values reindexed from zero, and reindexes remaining integer keys; omitted length, replacement insertion, and full by-reference diagnostics are deferred. Source: https://www.php.net/manual/en/function.array-splice.php |
| `array_sum` | implemented | Sums array values with PHP-compatible numeric coercion for current scalar values; empty arrays return `0`. Source: https://www.php.net/manual/en/function.array-sum.php |
| `array_udiff` | missing |  |
| `array_udiff_assoc` | missing |  |
| `array_udiff_uassoc` | missing |  |
| `array_uintersect` | missing |  |
| `array_uintersect_assoc` | missing |  |
| `array_uintersect_uassoc` | missing |  |
| `array_unique` | partial | Removes duplicate values using explicit `SORT_STRING` comparison while preserving original keys; omitted flag defaulting and other sort modes are deferred. Source: https://www.php.net/manual/en/function.array-unique.php |
| `array_unshift` | partial | Prepends one value to Echo's mutable PHP array storage, reindexing integer keys from zero while preserving string keys, and returns the new element count; variadic prepend and full by-reference diagnostics are deferred. Source: https://www.php.net/manual/en/function.array-unshift.php |
| `array_values` | implemented | Returns values in insertion order and reindexes the result numerically. Source: https://www.php.net/manual/en/function.array-values.php |
| `array_walk` | missing |  |
| `array_walk_recursive` | missing |  |
| `arsort` | missing |  |
| `asin` | implemented | Returns the arc sine in radians using PHP-compatible float coercion. Source: https://www.php.net/manual/en/function.asin.php |
| `asinh` | implemented | Returns inverse hyperbolic sine as a float with PHP-compatible numeric coercion. Source: https://www.php.net/manual/en/function.asinh.php |
| `asort` | missing |  |
| `assert` | missing |  |
| `assert_options` | missing |  |
| `atan` | implemented | Returns the arc tangent in radians using PHP-compatible float coercion. Source: https://www.php.net/manual/en/function.atan.php |
| `atan2` | implemented | Returns the quadrant-aware arc tangent of y/x in radians using PHP-compatible float coercion. Source: https://www.php.net/manual/en/function.atan2.php |
| `atanh` | implemented | Returns inverse hyperbolic tangent as a float with PHP-compatible numeric coercion and `NAN` outside the domain. Source: https://www.php.net/manual/en/function.atanh.php |
| `base64_decode` | implemented | Decodes Base64 strings using PHP's default non-strict path for current scalar strings. Source: https://www.php.net/manual/en/function.base64-decode.php |
| `base64_encode` | implemented | Encodes string bytes using standard Base64 output. Source: https://www.php.net/manual/en/function.base64-encode.php |
| `base_convert` | implemented | Converts strings between bases 2 through 36, ignoring characters outside the source base for current supported values. Source: https://www.php.net/manual/en/function.base-convert.php |
| `basename` | implemented | Supports Unix-style `/` separators and optional suffix stripping; Windows `\` separator behavior is deferred. Source: https://www.php.net/manual/en/function.basename.php |
| `bin2hex` | implemented | Converts each input byte to a two-character lowercase hexadecimal representation. Source: https://www.php.net/manual/en/function.bin2hex.php |
| `bindec` | implemented | Converts binary strings to unsigned decimal int or float values while ignoring non-binary characters. Source: https://www.php.net/manual/en/function.bindec.php |
| `boolval` | implemented | Converts current scalar values with PHP truthiness rules, including empty string and `"0"` as false. Source: https://www.php.net/manual/en/function.boolval.php |
| `call_user_func` | missing |  |
| `call_user_func_array` | missing |  |
| `ceil` | implemented | Rounds numeric values up while returning a float, including PHP-compatible scalar coercion and negative zero behavior. Source: https://www.php.net/manual/en/function.ceil.php |
| `chdir` | implemented | Changes the process current working directory and returns a bool success value; PHP warning emission on failure is deferred. Source: https://www.php.net/manual/en/function.chdir.php |
| `checkdnsrr` | missing |  |
| `chgrp` | missing |  |
| `chmod` | missing |  |
| `chop` | implemented | Alias of `rtrim()` for default trailing whitespace stripping; optional character mask support is deferred with `rtrim()`. Source: https://www.php.net/manual/en/function.chop.php |
| `chown` | missing |  |
| `chr` | implemented | Returns a one-byte string from an integer code modulo 256. Source: https://www.php.net/manual/en/function.chr.php |
| `chroot` | missing |  |
| `chunk_split` | implemented | Splits byte strings into fixed-size chunks and appends the requested separator after every chunk, including empty input. Source: https://www.php.net/manual/en/function.chunk-split.php |
| `clearstatcache` | implemented | No-op because Echo currently reads local filesystem metadata directly and does not model PHP's stat or realpath cache yet. Source: https://www.php.net/manual/en/function.clearstatcache.php |
| `cli_get_process_title` | implemented | Returns the process-local title last set through `cli_set_process_title()`, or `null` before one is set. Source: https://www.php.net/manual/en/function.cli-get-process-title.php |
| `cli_set_process_title` | implemented | Stores a process-local title string and returns `true`; host OS process-title mutation is deferred. Source: https://www.php.net/manual/en/function.cli-set-process-title.php |
| `closedir` | missing |  |
| `closelog` | missing |  |
| `compact` | missing |  |
| `connection_aborted` | implemented | Returns `0` because Echo currently uses CLI-style execution and does not model an abortable client connection. Source: https://www.php.net/manual/en/function.connection-aborted.php |
| `connection_status` | implemented | Returns `0` (`CONNECTION_NORMAL`) because Echo currently uses CLI-style execution and does not model an abortable client connection. Source: https://www.php.net/manual/en/function.connection-status.php |
| `constant` | partial | Returns Echo's compiler-owned PHP compatibility constants by name; dynamically defined constants and PHP's missing-constant `Error` parity are deferred. Source: https://www.php.net/manual/en/function.constant.php |
| `convert_uudecode` | missing |  |
| `convert_uuencode` | missing |  |
| `copy` | implemented | Copies local files and returns a bool success value; stream contexts, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.copy.php |
| `cos` | implemented | Returns the cosine of a radian value using PHP-compatible float coercion. Source: https://www.php.net/manual/en/function.cos.php |
| `cosh` | implemented | Returns hyperbolic cosine as a float with PHP-compatible numeric coercion. Source: https://www.php.net/manual/en/function.cosh.php |
| `count` | implemented | Supports PHP array/list counting; recursive mode and Countable objects are deferred. Source: https://www.php.net/manual/en/function.count.php |
| `count_chars` | missing |  |
| `crc32` | implemented | Calculates a CRC32 checksum over the string bytes and returns the positive integer result used by 64-bit PHP. Source: https://www.php.net/manual/en/function.crc32.php |
| `crypt` | implemented | Uses Echo's bcrypt implementation through `crypt()` salt prefix dispatch for supported variants; unsupported salts and algorithm formats return `false`. Source: https://www.php.net/manual/en/function.crypt.php |
| `current` | missing |  |
| `debug_zval_dump` | missing |  |
| `decbin` | implemented | Converts integers to unsigned binary strings; current runtime follows the 64-bit target width for negative integers. Source: https://www.php.net/manual/en/function.decbin.php |
| `dechex` | implemented | Converts integers to lowercase unsigned hexadecimal strings; current runtime follows the 64-bit target width for negative integers. Source: https://www.php.net/manual/en/function.dechex.php |
| `decoct` | implemented | Converts integers to unsigned octal strings; current runtime follows the 64-bit target width for negative integers. Source: https://www.php.net/manual/en/function.decoct.php |
| `deg2rad` | implemented | Converts degrees to radians using PHP-compatible float coercion for current scalar values. Source: https://www.php.net/manual/en/function.deg2rad.php |
| `dir` | missing |  |
| `dirname` | implemented | Supports Unix-style `/` separators and positive `levels`; Windows `\` separator behavior and direct PHP ValueError diagnostics are deferred. Source: https://www.php.net/manual/en/function.dirname.php |
| `disk_free_space` | missing |  |
| `disk_total_space` | missing |  |
| `diskfreespace` | missing |  |
| `dl` | missing |  |
| `dns_check_record` | missing |  |
| `dns_get_mx` | missing |  |
| `dns_get_record` | missing |  |
| `doubleval` | implemented | Alias of `floatval()`. Source: https://www.php.net/manual/en/function.doubleval.php |
| `end` | missing |  |
| `error_clear_last` | missing |  |
| `error_get_last` | missing |  |
| `error_log` | missing |  |
| `escapeshellarg` | implemented | Supports Unix/POSIX single-quote wrapping and embedded single-quote escaping; Windows-specific quoting behavior is deferred. Source: https://www.php.net/manual/en/function.escapeshellarg.php |
| `escapeshellcmd` | implemented | Supports Unix/POSIX backslash escaping for shell metacharacters and unpaired quotes; Windows caret escaping is deferred. Source: https://www.php.net/manual/en/function.escapeshellcmd.php |
| `exec` | missing |  |
| `exp` | implemented | Calculates e raised to a numeric power with PHP-compatible scalar coercion. Source: https://www.php.net/manual/en/function.exp.php |
| `explode` | implemented | Splits byte strings into PHP arrays with default, positive, zero, and negative limit behavior; empty-separator `ValueError` is currently surfaced as a runtime error. Source: https://www.php.net/manual/en/function.explode.php |
| `expm1` | implemented | Calculates exp(num) - 1 with a small-value path that preserves precision near zero. Source: https://www.php.net/manual/en/function.expm1.php |
| `extract` | missing |  |
| `fclose` | implemented | Closes Echo's local file stream resources and returns a bool success value; broader PHP stream wrappers are deferred. Source: https://www.php.net/manual/en/function.fclose.php |
| `fdatasync` | missing |  |
| `fdiv` | implemented | Divides two numeric values as IEEE 754 floats, returning `INF`, `-INF`, or `NAN` for zero-divisor cases instead of raising division errors. Source: https://www.php.net/manual/en/function.fdiv.php |
| `feof` | missing |  |
| `fflush` | missing |  |
| `fgetc` | missing |  |
| `fgetcsv` | missing |  |
| `fgets` | missing |  |
| `file` | missing |  |
| `file_exists` | implemented | Checks local filesystem paths for files or directories; stat cache, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.file-exists.php |
| `file_get_contents` | implemented | Reads local files into byte strings with offset and nullable length support, including negative offsets; include path lookup, stream contexts, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.file-get-contents.php |
| `file_put_contents` | implemented | Writes local files and returns byte counts, with append flag support; array data, stream resources, `LOCK_EX`, stream contexts, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.file-put-contents.php |
| `fileatime` | implemented | Returns the local file's last access time as a Unix timestamp and `false` when metadata cannot be read; stat cache, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.fileatime.php |
| `filectime` | implemented | Returns the local file's inode change time as a Unix timestamp and `false` when metadata cannot be read; stat cache, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.filectime.php |
| `filegroup` | implemented | Returns the local file's numeric group ID and `false` when metadata cannot be read; stat cache, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.filegroup.php |
| `fileinode` | implemented | Returns the local file's inode number and `false` when metadata cannot be read; stat cache, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.fileinode.php |
| `filemtime` | implemented | Returns the local file's content modification time as a Unix timestamp and `false` when metadata cannot be read; stat cache, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.filemtime.php |
| `fileowner` | implemented | Returns the local file's numeric owner ID and `false` when metadata cannot be read; stat cache, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.fileowner.php |
| `fileperms` | implemented | Returns the local file's numeric mode bits and `false` when metadata cannot be read; stat cache, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.fileperms.php |
| `filesize` | implemented | Returns the local file size as an integer and `false` when metadata cannot be read; stat cache, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.filesize.php |
| `filetype` | implemented | Returns local file type strings such as `file`, `dir`, and `link`, or `false` when metadata cannot be read; stat cache, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.filetype.php |
| `floatval` | implemented | Gets the float value of current scalar values, including numeric-prefix string parsing. Source: https://www.php.net/manual/en/function.floatval.php |
| `flock` | missing |  |
| `floor` | implemented | Rounds numeric values down while returning a float, using PHP-compatible scalar coercion. Source: https://www.php.net/manual/en/function.floor.php |
| `flush` | implemented | Flushes the system output layer without flushing active user-level output buffers. Source: https://www.php.net/manual/en/function.flush.php |
| `fmod` | implemented | Returns a floating-point remainder with the dividend sign and `NAN` for zero divisors. Source: https://www.php.net/manual/en/function.fmod.php |
| `fnmatch` | missing |  |
| `fopen` | implemented | Opens local filesystem streams for common read/write modes; include path lookup, URL wrappers, stream contexts, and warning emission are deferred. Source: https://www.php.net/manual/en/function.fopen.php |
| `forward_static_call` | missing |  |
| `forward_static_call_array` | missing |  |
| `fpassthru` | missing |  |
| `fpow` | implemented | Raises a numeric base to a numeric exponent and always returns a float, including `INF` for zero raised to a negative exponent and `NAN` for unsupported negative fractional powers. Source: https://www.php.net/manual/en/function.fpow.php |
| `fprintf` | missing |  |
| `fputcsv` | missing |  |
| `fputs` | missing |  |
| `fread` | implemented | Reads up to the requested byte count from Echo local file stream resources and advances the cursor. Source: https://www.php.net/manual/en/function.fread.php |
| `fscanf` | missing |  |
| `fseek` | missing |  |
| `fsockopen` | missing |  |
| `fstat` | missing |  |
| `fsync` | missing |  |
| `ftell` | missing |  |
| `ftok` | missing |  |
| `ftruncate` | missing |  |
| `fwrite` | missing |  |
| `get_browser` | missing |  |
| `get_cfg_var` | implemented | Returns `false` for configuration options because Echo does not load PHP configuration values. Source: https://www.php.net/manual/en/function.get-cfg-var.php |
| `get_current_user` | missing |  |
| `get_debug_type` | missing |  |
| `get_headers` | missing |  |
| `get_html_translation_table` | missing |  |
| `get_include_path` | implemented | Equivalent to `ini_get("include_path")`; returns `false` because Echo does not model PHP ini option values. Source: https://www.php.net/manual/en/function.get-include-path.php |
| `get_meta_tags` | missing |  |
| `getcwd` | implemented | Returns the current working directory as a string or `false` if the host cannot report it. Source: https://www.php.net/manual/en/function.getcwd.php |
| `getenv` | implemented | Returns a named environment value, all environment variables as an associative array when omitted/null, or `false` for a missing name; SAPI-local distinctions are not modeled. Source: https://www.php.net/manual/en/function.getenv.php |
| `gethostbyaddr` | missing |  |
| `gethostbyname` | missing |  |
| `gethostbynamel` | missing |  |
| `gethostname` | implemented | Returns the local host name as a string when the host can report one, otherwise `false`. Source: https://www.php.net/manual/en/function.gethostname.php |
| `getimagesize` | missing |  |
| `getimagesizefromstring` | missing |  |
| `getlastmod` | missing |  |
| `getmxrr` | missing |  |
| `getmygid` | missing |  |
| `getmyinode` | missing |  |
| `getmypid` | implemented | Returns the current process ID as an integer; like PHP, this is process metadata and not a secure entropy source. Source: https://www.php.net/manual/en/function.getmypid.php |
| `getmyuid` | missing |  |
| `getopt` | missing |  |
| `getprotobyname` | missing |  |
| `getprotobynumber` | missing |  |
| `getrusage` | missing |  |
| `getservbyname` | missing |  |
| `getservbyport` | missing |  |
| `gettimeofday` | implemented | Returns an associative array with `sec`, `usec`, `minuteswest`, and `dsttime` by default, or a float timestamp when `as_float` is true. Source: https://www.php.net/manual/en/function.gettimeofday.php |
| `gettype` | implemented | Returns PHP type names for Echo's current value tags. Source: https://www.php.net/manual/en/function.gettype.php |
| `glob` | missing |  |
| `header` | implemented | No-op because Echo currently uses CLI-style execution and does not model an HTTP header layer. Source: https://www.php.net/manual/en/function.header.php |
| `header_register_callback` | missing |  |
| `header_remove` | implemented | No-op because Echo currently uses CLI-style execution and does not model an HTTP header layer. Source: https://www.php.net/manual/en/function.header-remove.php |
| `headers_list` | implemented | Returns an empty array because Echo currently uses CLI-style execution and does not model an HTTP header layer. Source: https://www.php.net/manual/en/function.headers-list.php |
| `headers_sent` | implemented | Returns `false` because Echo currently uses CLI-style execution and does not model an HTTP header layer; optional filename/line reference outputs are deferred. Source: https://www.php.net/manual/en/function.headers-sent.php |
| `hebrev` | missing |  |
| `hex2bin` | implemented | Converts even-length hexadecimal strings to raw bytes and returns `false` for invalid hex input. Source: https://www.php.net/manual/en/function.hex2bin.php |
| `hexdec` | implemented | Converts hexadecimal strings to unsigned decimal int or float values while ignoring non-hexadecimal characters. Source: https://www.php.net/manual/en/function.hexdec.php |
| `highlight_file` | missing |  |
| `highlight_string` | missing |  |
| `hrtime` | implemented | Returns a two-element `[seconds, nanoseconds]` array by default or an integer/float nanosecond count when `as_number` is true. Source: https://www.php.net/manual/en/function.hrtime.php |
| `html_entity_decode` | missing |  |
| `htmlentities` | missing |  |
| `htmlspecialchars` | implemented | Escapes `&`, `"`, `'`, `<`, and `>` using PHP's default `ENT_QUOTES | ENT_SUBSTITUTE | ENT_HTML401` shape; optional flags, encoding, and double-encode control are deferred. Source: https://www.php.net/manual/en/function.htmlspecialchars.php |
| `htmlspecialchars_decode` | implemented | Decodes the default `htmlspecialchars()` entity set for `&amp;`, `&quot;`, `&#039;`, `&lt;`, and `&gt;`; optional flags are deferred. Source: https://www.php.net/manual/en/function.htmlspecialchars-decode.php |
| `http_build_query` | missing |  |
| `http_clear_last_response_headers` | missing |  |
| `http_get_last_response_headers` | missing |  |
| `http_response_code` | implemented | Tracks the process-local response code; returns `false` before a code is set, `true` for the first set, and the previous/current code as documented for CLI-style use. Source: https://www.php.net/manual/en/function.http-response-code.php |
| `hypot` | implemented | Returns the Euclidean distance for two numeric values using PHP-compatible float coercion. Source: https://www.php.net/manual/en/function.hypot.php |
| `ignore_user_abort` | implemented | Tracks process-local ignore-user-abort state and returns the previous/current `0` or `1` setting; Echo does not model an abortable client connection yet. Source: https://www.php.net/manual/en/function.ignore-user-abort.php |
| `image_type_to_extension` | missing |  |
| `image_type_to_mime_type` | missing |  |
| `implode` | implemented | Joins PHP array values in order, supports the optional empty-string separator form, and uses PHP string coercion for scalar elements. Source: https://www.php.net/manual/en/function.implode.php |
| `in_array` | implemented | Searches array values with loose comparison by default and strict same-type comparison when the third argument is true. Source: https://www.php.net/manual/en/function.in-array.php |
| `inet_ntop` | missing |  |
| `inet_pton` | missing |  |
| `ini_alter` | implemented | Alias of `ini_set`; returns `false` because Echo does not model mutable PHP ini option values. Source: https://www.php.net/manual/en/function.ini-alter.php |
| `ini_get` | implemented | Returns `false` for configuration options because Echo does not model PHP ini option values. Source: https://www.php.net/manual/en/function.ini-get.php |
| `ini_get_all` | implemented | Returns an empty array for the core ini registry and `false` for named extensions because Echo does not model PHP ini option values. Source: https://www.php.net/manual/en/function.ini-get-all.php |
| `ini_parse_quantity` | implemented | Parses PHP ini shorthand quantities with decimal, binary, octal, hexadecimal, and `K`/`M`/`G` multipliers; invalid leading text returns `0` and unknown suffixes keep the parsed number. Source: https://www.php.net/manual/en/function.ini-parse-quantity.php |
| `ini_restore` | implemented | No-op because Echo does not model mutable PHP ini option values. Source: https://www.php.net/manual/en/function.ini-restore.php |
| `ini_set` | implemented | Returns `false` because Echo does not model mutable PHP ini option values. Source: https://www.php.net/manual/en/function.ini-set.php |
| `intdiv` | implemented | Divides integer-compatible operands and returns the quotient truncated toward zero; division by zero and overflow currently surface as runtime errors. Source: https://www.php.net/manual/en/function.intdiv.php |
| `intval` | implemented | Converts current scalar values to integers using PHP-style bool, null, float, and numeric-string coercion. Source: https://www.php.net/manual/en/function.intval.php |
| `ip2long` | missing |  |
| `iptcembed` | missing |  |
| `iptcparse` | missing |  |
| `is_array` | implemented | Supports Echo list values as PHP arrays. Source: https://www.php.net/manual/en/function.is-array.php |
| `is_bool` | implemented | Source: https://www.php.net/manual/en/function.is-bool.php |
| `is_callable` | implemented | Supports string function names in the runtime function registry; callable arrays/objects and optional arguments are deferred. Source: https://www.php.net/manual/en/function.is-callable.php |
| `is_countable` | implemented | Supports arrays; Countable objects deferred. Source: https://www.php.net/manual/en/function.is-countable.php |
| `is_dir` | implemented | Checks local filesystem paths and returns true only for existing directories; stat cache, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.is-dir.php |
| `is_double` | implemented | Alias of `is_float()`. Source: https://www.php.net/manual/en/function.is-float.php |
| `is_executable` | implemented | Checks local filesystem paths for executable files; Unix mode-bit behavior is supported and Windows extension behavior is approximate. Source: https://www.php.net/manual/en/function.is-executable.php |
| `is_file` | implemented | Checks local filesystem paths and returns true only for existing regular files; stat cache, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.is-file.php |
| `is_finite` | implemented | Supports current numeric scalar values; Echo float payloads are deferred. Source: https://www.php.net/manual/en/function.is-finite.php |
| `is_float` | implemented | Echo has no float values yet, so this is false for all currently representable values. Source: https://www.php.net/manual/en/function.is-float.php |
| `is_infinite` | implemented | Supports current numeric scalar values; Echo float payloads are deferred. Source: https://www.php.net/manual/en/function.is-infinite.php |
| `is_int` | implemented | Source: https://www.php.net/manual/en/function.is-int.php |
| `is_integer` | implemented | Alias of `is_int()`. Source: https://www.php.net/manual/en/function.is-int.php |
| `is_iterable` | implemented | Supports arrays; Traversable objects deferred. Source: https://www.php.net/manual/en/function.is-iterable.php |
| `is_link` | implemented | Checks local filesystem paths and returns true only for existing symbolic links; stat cache, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.is-link.php |
| `is_long` | implemented | Alias of `is_int()`. Source: https://www.php.net/manual/en/function.is-int.php |
| `is_nan` | implemented | Supports current numeric scalar values; Echo float payloads are deferred. Source: https://www.php.net/manual/en/function.is-nan.php |
| `is_null` | implemented | Source: https://www.php.net/manual/en/function.is-null.php |
| `is_numeric` | implemented | Supports Echo integers and PHP numeric strings, including decimal/exponent forms and ASCII edge whitespace. Source: https://www.php.net/manual/en/function.is-numeric.php |
| `is_object` | implemented | Supports Echo structural object values. Source: https://www.php.net/manual/en/function.is-object.php |
| `is_readable` | implemented | Checks local filesystem paths by probing file open or directory listing access; stat cache, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.is-readable.php |
| `is_resource` | implemented | Reports Echo runtime resource handles such as TCP listeners/connections. Source: https://www.php.net/manual/en/function.is-resource.php |
| `is_scalar` | implemented | Supports current scalar values: bool, int, string. Source: https://www.php.net/manual/en/function.is-scalar.php |
| `is_string` | implemented | Source: https://www.php.net/manual/en/function.is-string.php |
| `is_uploaded_file` | missing |  |
| `is_writable` | implemented | Checks local filesystem paths by probing append access or temporary creation inside directories; stat cache, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.is-writable.php |
| `is_writeable` | implemented | Alias of `is_writable()`. Source: https://www.php.net/manual/en/function.is-writable.php |
| `join` | implemented | Alias of `implode()`. Source: https://www.php.net/manual/en/function.join.php |
| `key` | missing |  |
| `key_exists` | implemented | Alias of `array_key_exists()`. Source: https://www.php.net/manual/en/function.key-exists.php |
| `krsort` | missing |  |
| `ksort` | missing |  |
| `lcfirst` | implemented | Lowercases the first ASCII alphabetic byte of a string and leaves the rest unchanged. Source: https://www.php.net/manual/en/function.lcfirst.php |
| `lchgrp` | missing |  |
| `lchown` | missing |  |
| `levenshtein` | implemented | Calculates byte-string edit distance with optional insertion, replacement, and deletion costs. Source: https://www.php.net/manual/en/function.levenshtein.php |
| `link` | implemented | Creates local hard links and returns a bool success value; PHP warning emission and platform-specific filesystem edge cases are deferred. Source: https://www.php.net/manual/en/function.link.php |
| `linkinfo` | missing |  |
| `localeconv` | missing |  |
| `log` | implemented | Calculates natural logarithms by default and supports an optional positive base; non-positive bases are surfaced as runtime errors for now. Source: https://www.php.net/manual/en/function.log.php |
| `log10` | implemented | Calculates base-10 logarithms with PHP-compatible scalar coercion. Source: https://www.php.net/manual/en/function.log10.php |
| `log1p` | implemented | Calculates log(1 + num) with a small-value path that preserves precision near zero. Source: https://www.php.net/manual/en/function.log1p.php |
| `long2ip` | missing |  |
| `lstat` | missing |  |
| `ltrim` | implemented | Removes PHP's default leading ASCII whitespace bytes; custom character masks are deferred. Source: https://www.php.net/manual/en/function.ltrim.php |
| `mail` | missing |  |
| `hash` | implemented | Dispatches supported hash algorithms and returns raw bytes with `raw_output=true`, otherwise lowercase hex. Source: https://www.php.net/manual/en/function.hash.php |
| `hash_algos` | implemented | Returns supported hash algorithm names from runtime registry. Source: https://www.php.net/manual/en/function.hash-algos.php |
| `hash_copy` | implemented | Clones an active hash context object. Source: https://www.php.net/manual/en/function.hash-copy.php |
| `hash_equals` | implemented | Compares strings in timing-safe manner and returns `false` for length mismatch. Source: https://www.php.net/manual/en/function.hash-equals.php |
| `hash_file` | implemented | Hashes file contents via filename input and supports optional raw output. Source: https://www.php.net/manual/en/function.hash-file.php |
| `hash_final` | implemented | Finalizes active hash contexts and marks them finalized. Source: https://www.php.net/manual/en/function.hash-final.php |
| `hash_hkdf` | implemented | Derives key material from HKDF inputs with optional raw hex output. Source: https://www.php.net/manual/en/function.hash-hkdf.php |
| `hash_hmac` | implemented | Computes keyed HMAC digests for configured algorithms. Source: https://www.php.net/manual/en/function.hash-hmac.php |
| `hash_hmac_algos` | implemented | Returns algorithms accepted by hash_hmac. Source: https://www.php.net/manual/en/function.hash-hmac-algos.php |
| `hash_hmac_file` | implemented | Computes keyed HMAC from file contents. Source: https://www.php.net/manual/en/function.hash-hmac-file.php |
| `hash_init` | implemented | Creates hash contexts with optional options/key initialization values. Source: https://www.php.net/manual/en/function.hash-init.php |
| `hash_pbkdf2` | implemented | Derives keys with PBKDF2 using HMAC iterations and optional raw output. Source: https://www.php.net/manual/en/function.hash-pbkdf2.php |
| `hash_update` | implemented | Updates an existing hash context with string data. Source: https://www.php.net/manual/en/function.hash-update.php |
| `hash_update_file` | implemented | Appends file bytes to an active hash context. Source: https://www.php.net/manual/en/function.hash-update-file.php |
| `hash_update_stream` | implemented | Updates hash contexts from Echo local file stream resources and advances the stream cursor; non-file stream wrappers are deferred. Source: https://www.php.net/manual/en/function.hash-update-stream.php |
| `max` | missing |  |
| `md5` | implemented | Returns a lowercase 32-character MD5 digest by default and raw 16-byte output when the optional binary flag is true; not suitable for password storage. Source: https://www.php.net/manual/en/function.md5.php |
| `md5_file` | implemented | Hashes local files with optional raw output and returns `false` for missing files. Source: https://www.php.net/manual/en/function.md5-file.php |
| `memory_get_peak_usage` | missing |  |
| `memory_get_usage` | missing |  |
| `memory_reset_peak_usage` | missing |  |
| `metaphone` | missing |  |
| `microtime` | implemented | Supports string and float forms for current wall-clock time. Source: https://www.php.net/manual/en/function.microtime.php |
| `min` | missing |  |
| `mkdir` | implemented | Creates local directories, including recursive creation and Unix mode hints; stream contexts, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.mkdir.php |
| `move_uploaded_file` | missing |  |
| `natcasesort` | missing |  |
| `natsort` | missing |  |
| `net_get_interfaces` | missing |  |
| `next` | missing |  |
| `nl2br` | implemented | Inserts `<br />` or `<br>` before newline sequences while preserving the original newline bytes. Source: https://www.php.net/manual/en/function.nl2br.php |
| `nl_langinfo` | missing |  |
| `number_format` | implemented | Formats a number with grouped thousands, half-up rounding, PHP's 1-, 2-, and 4-argument separator forms, and PHP 8.3 negative decimal precision. Source: https://www.php.net/manual/en/function.number-format.php |
| `ob_clean` | implemented | Discards the active output buffer contents while keeping the buffer open. Source: https://www.php.net/manual/en/function.ob-clean.php |
| `ob_end_clean` | implemented | Discards and closes the active output buffer, returning a bool success value. Source: https://www.php.net/manual/en/function.ob-end-clean.php |
| `ob_end_flush` | implemented | Flushes and closes the active output buffer, returning a bool success value. Source: https://www.php.net/manual/en/function.ob-end-flush.php |
| `ob_flush` | implemented | Flushes the active output buffer to the next output layer while keeping it open. Source: https://www.php.net/manual/en/function.ob-flush.php |
| `ob_get_clean` | implemented | Returns active buffer contents and closes the buffer, or `false` when no buffer is active. Source: https://www.php.net/manual/en/function.ob-get-clean.php |
| `ob_get_contents` | implemented | Returns active buffer contents without closing it, or `false` when no buffer is active. Source: https://www.php.net/manual/en/function.ob-get-contents.php |
| `ob_get_flush` | implemented | Returns active buffer contents, flushes them, and closes the buffer, or `false` when no buffer is active. Source: https://www.php.net/manual/en/function.ob-get-flush.php |
| `ob_get_length` | implemented | Returns the active output buffer length in bytes, or `false` when no buffer is active. Source: https://www.php.net/manual/en/function.ob-get-length.php |
| `ob_get_level` | implemented | Returns the current output buffering nesting depth. Source: https://www.php.net/manual/en/function.ob-get-level.php |
| `ob_get_status` | missing |  |
| `ob_implicit_flush` | implemented | Toggles implicit system flushing after output writes without flushing active user-level buffers. Source: https://www.php.net/manual/en/function.ob-implicit-flush.php |
| `ob_list_handlers` | missing |  |
| `ob_start` | implemented | Starts a new output buffer and stores optional callback metadata; callback invocation is deferred. Source: https://www.php.net/manual/en/function.ob-start.php |
| `octdec` | implemented | Converts octal strings to unsigned decimal int or float values while ignoring non-octal characters. Source: https://www.php.net/manual/en/function.octdec.php |
| `opendir` | missing |  |
| `openlog` | missing |  |
| `ord` | implemented | Returns the integer value of the first byte in a string. Source: https://www.php.net/manual/en/function.ord.php |
| `output_add_rewrite_var` | missing |  |
| `output_reset_rewrite_vars` | missing |  |
| `pack` | missing |  |
| `parse_ini_file` | missing |  |
| `parse_ini_string` | missing |  |
| `parse_str` | missing |  |
| `parse_url` | missing |  |
| `passthru` | missing |  |
| `password_algos` | implemented | Returns the supported password algorithm IDs for `password_hash()` validation and `password_get_info()`. Source: https://www.php.net/manual/en/password.constants.php |
| `password_get_info` | implemented | Returns a map with `algo`, `algoName`, and `options` for bcrypt-compatible hashes. Source: https://www.php.net/manual/en/function.password-get-info.php |
| `password_hash` | implemented | Generates bcrypt hashes using the selected algorithm and options while surfacing invalid inputs as `false`. Source: https://www.php.net/manual/en/function.password-hash.php |
| `password_needs_rehash` | implemented | Compares a hash against requested algorithm/options to determine when rehashing is required. Source: https://www.php.net/manual/en/function.password-needs-rehash.php |
| `password_verify` | implemented | Verifies candidate passwords against generated and stored password hashes in a timing-safe comparison path. Source: https://www.php.net/manual/en/function.password-verify.php |
| `pathinfo` | missing |  |
| `pclose` | missing |  |
| `pfsockopen` | missing |  |
| `php_ini_loaded_file` | implemented | Returns `false` because Echo does not load a PHP configuration file. Source: https://www.php.net/manual/en/function.php-ini-loaded-file.php |
| `php_ini_scanned_files` | implemented | Returns `false` because Echo does not scan PHP configuration directories. Source: https://www.php.net/manual/en/function.php-ini-scanned-files.php |
| `php_sapi_name` | implemented | Returns Echo's PHP compatibility Server API name, currently `cli`, matching the `PHP_SAPI` constant. Source: https://www.php.net/manual/en/function.php-sapi-name.php |
| `php_strip_whitespace` | missing |  |
| `php_uname` | missing |  |
| `phpcredits` | missing |  |
| `phpinfo` | missing |  |
| `phpversion` | implemented | Returns Echo's PHP compatibility version for no extension or `null`; named extension versions are not modeled yet and return `false`. Source: https://www.php.net/manual/en/function.phpversion.php |
| `pi` | implemented | Returns an approximation of pi as a float. Source: https://www.php.net/manual/en/function.pi.php |
| `popen` | missing |  |
| `pos` | missing |  |
| `pow` | implemented | Raises a numeric base to a numeric exponent, returning int for representable non-negative integer powers and float otherwise. Source: https://www.php.net/manual/en/function.pow.php |
| `prev` | missing |  |
| `print_r` | missing |  |
| `printf` | missing |  |
| `proc_close` | missing |  |
| `proc_get_status` | missing |  |
| `proc_nice` | missing |  |
| `proc_open` | missing |  |
| `proc_terminate` | missing |  |
| `putenv` | implemented | Sets `NAME=value` entries in the current process environment and removes a variable when passed a bare name; request-lifetime restoration is deferred to process lifetime. Source: https://www.php.net/manual/en/function.putenv.php |
| `quoted_printable_decode` | implemented | Decodes quoted-printable `=XX` byte escapes and soft line breaks according to the scalar string path. Source: https://www.php.net/manual/en/function.quoted-printable-decode.php |
| `quoted_printable_encode` | implemented | Encodes bytes outside the printable quoted-printable range as uppercase `=XX`; RFC line wrapping and trailing-space line handling are deferred. Source: https://www.php.net/manual/en/function.quoted-printable-encode.php |
| `quotemeta` | implemented | Prefixes regular expression metacharacters with backslashes for literal matching tasks. Source: https://www.php.net/manual/en/function.quotemeta.php |
| `rad2deg` | implemented | Converts radians to degrees using PHP-compatible float coercion for current scalar values. Source: https://www.php.net/manual/en/function.rad2deg.php |
| `range` | missing |  |
| `rawurldecode` | implemented | Decodes `%XX` byte escapes without converting `+` to a space. Source: https://www.php.net/manual/en/function.rawurldecode.php |
| `rawurlencode` | implemented | Encodes bytes according to RFC 3986, preserving alphanumerics and `-_.~`. Source: https://www.php.net/manual/en/function.rawurlencode.php |
| `readdir` | missing |  |
| `readfile` | implemented | Streams a local file through Echo's output buffer and returns the byte count; include path lookup, stream contexts, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.readfile.php |
| `readlink` | implemented | Reads the stored target of a local symbolic link and returns `false` when it cannot be read; PHP warning emission is deferred. Source: https://www.php.net/manual/en/function.readlink.php |
| `realpath` | implemented | Resolves existing local paths through OS canonicalization and returns `false` for missing paths; realpath cache APIs and URL wrappers are deferred. Source: https://www.php.net/manual/en/function.realpath.php |
| `realpath_cache_get` | missing |  |
| `realpath_cache_size` | missing |  |
| `register_shutdown_function` | missing |  |
| `register_tick_function` | missing |  |
| `rename` | implemented | Renames or moves local files/directories using host filesystem semantics; stream contexts, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.rename.php |
| `request_parse_body` | missing |  |
| `restore_include_path` | missing | Source: https://www.php.net/manual/en/function.restore-include-path.php |
| `reset` | missing |  |
| `rewind` | missing |  |
| `rewinddir` | missing |  |
| `rmdir` | implemented | Removes empty local directories and returns a bool success value; stream contexts, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.rmdir.php |
| `round` | implemented | Rounds numeric values to a precision using PHP's default half-away-from-zero mode; explicit rounding mode constants are deferred. Source: https://www.php.net/manual/en/function.round.php |
| `rsort` | partial | Sorts arrays containing string-compatible values in descending byte order, reindexes keys from zero, and returns `true`; flags and non-string comparison modes are deferred. Source: https://www.php.net/manual/en/function.rsort.php |
| `rtrim` | implemented | Removes PHP's default trailing ASCII whitespace bytes; custom character masks are deferred. Source: https://www.php.net/manual/en/function.rtrim.php |
| `scandir` | missing |  |
| `serialize` | missing |  |
| `set_file_buffer` | missing |  |
| `set_include_path` | missing |  |
| `set_time_limit` | missing |  |
| `setcookie` | missing |  |
| `setlocale` | missing |  |
| `setrawcookie` | missing |  |
| `settype` | missing |  |
| `sha1` | implemented | Returns a lowercase 40-character SHA-1 digest by default and raw 20-byte output when the optional binary flag is true; not suitable for password storage. Source: https://www.php.net/manual/en/function.sha1.php |
| `sha1_file` | implemented | Hashes local files with optional raw output and returns `false` for missing files. Source: https://www.php.net/manual/en/function.sha1-file.php |
| `shell_exec` | missing |  |
| `show_source` | missing |  |
| `random_bytes` | implemented | Generates cryptographically secure random bytes using OS entropy and returns `false` on failure or invalid length. Source: https://www.php.net/manual/en/function.random-bytes.php |
| `random_int` | implemented | Returns uniformly distributed integers in `[min,max]` and returns `false` for invalid argument ranges. Source: https://www.php.net/manual/en/function.random-int.php |
| `shuffle` | missing |  |
| `similar_text` | missing |  |
| `sin` | implemented | Returns the sine of a radian value using PHP-compatible float coercion. Source: https://www.php.net/manual/en/function.sin.php |
| `sinh` | implemented | Returns hyperbolic sine as a float with PHP-compatible numeric coercion. Source: https://www.php.net/manual/en/function.sinh.php |
| `sizeof` | implemented | Alias of `count()`. Source: https://www.php.net/manual/en/function.sizeof.php |
| `sleep` | missing |  |
| `socket_get_status` | missing |  |
| `socket_set_blocking` | missing |  |
| `socket_set_timeout` | missing |  |
| `sort` | partial | Sorts arrays containing string-compatible values in ascending byte order, reindexes keys from zero, and returns `true`; flags and non-string comparison modes are deferred. Source: https://www.php.net/manual/en/function.sort.php |
| `soundex` | implemented | Returns a four-character Soundex key using PHP's Knuth Soundex behavior for ASCII-compatible names, or `0000` when no letters are present. Source: https://www.php.net/manual/en/function.soundex.php |
| `sprintf` | missing |  |
| `sqrt` | implemented | Returns a square root as float and `NAN` for negative inputs using PHP-compatible float coercion. Source: https://www.php.net/manual/en/function.sqrt.php |
| `sscanf` | missing |  |
| `stat` | missing |  |
| `str_contains` | implemented | Performs binary-safe substring detection, including PHP's empty-needle true behavior. Source: https://www.php.net/manual/en/function.str-contains.php |
| `str_decrement` | missing |  |
| `str_ends_with` | implemented | Performs binary-safe suffix checks, including PHP's empty-needle true behavior. Source: https://www.php.net/manual/en/function.str-ends-with.php |
| `str_getcsv` | missing |  |
| `str_increment` | missing |  |
| `str_ireplace` | implemented | Performs ASCII case-insensitive scalar string search and replacement; array operands and by-reference count reporting are deferred. Source: https://www.php.net/manual/en/function.str-ireplace.php |
| `str_pad` | implemented | Pads byte strings on the left, right, or both sides with PHP's default right padding and pad-string truncation behavior. Source: https://www.php.net/manual/en/function.str-pad.php |
| `str_repeat` | implemented | Repeats a string a non-negative number of times and returns the concatenated byte string. Source: https://www.php.net/manual/en/function.str-repeat.php |
| `str_replace` | implemented | Performs scalar string search and replacement, including empty-search no-op behavior; array operands and by-reference count reporting are deferred. Source: https://www.php.net/manual/en/function.str-replace.php |
| `str_rot13` | implemented | Applies ROT13 to ASCII alphabetic bytes while leaving other bytes unchanged. Source: https://www.php.net/manual/en/function.str-rot13.php |
| `str_shuffle` | missing |  |
| `str_split` | implemented | Splits byte strings into an array of fixed-size chunks, with a default chunk length of one byte. Source: https://www.php.net/manual/en/function.str-split.php |
| `str_starts_with` | implemented | Performs binary-safe prefix checks, including PHP's empty-needle true behavior. Source: https://www.php.net/manual/en/function.str-starts-with.php |
| `str_word_count` | partial | Default count mode for ASCII words is implemented; PHP array return modes and the `characters` parameter are not implemented yet. See <https://www.php.net/manual/en/function.str-word-count.php>. |
| `strchr` | implemented | Alias of `strstr`. |
| `strcoll` | missing |  |
| `strcspn` | implemented | Counts the initial byte span containing none of the bytes from the mask string. Source: https://www.php.net/manual/en/function.strcspn.php |
| `stream_bucket_append` | missing |  |
| `stream_bucket_make_writeable` | missing |  |
| `stream_bucket_new` | missing |  |
| `stream_bucket_prepend` | missing |  |
| `stream_context_create` | missing |  |
| `stream_context_get_default` | missing |  |
| `stream_context_get_options` | missing |  |
| `stream_context_get_params` | missing |  |
| `stream_context_set_default` | missing |  |
| `stream_context_set_option` | missing |  |
| `stream_context_set_options` | missing |  |
| `stream_context_set_params` | missing |  |
| `stream_copy_to_stream` | missing |  |
| `stream_filter_append` | missing |  |
| `stream_filter_prepend` | missing |  |
| `stream_filter_register` | missing |  |
| `stream_filter_remove` | missing |  |
| `stream_get_contents` | implemented | Reads remaining bytes, or an optional bounded range, from Echo local file stream resources. Source: https://www.php.net/manual/en/function.stream-get-contents.php |
| `stream_get_filters` | missing |  |
| `stream_get_line` | missing |  |
| `stream_get_meta_data` | missing |  |
| `stream_get_transports` | missing |  |
| `stream_get_wrappers` | missing |  |
| `stream_is_local` | missing |  |
| `stream_isatty` | missing |  |
| `stream_register_wrapper` | missing |  |
| `stream_resolve_include_path` | missing |  |
| `stream_select` | missing |  |
| `stream_set_blocking` | missing |  |
| `stream_set_chunk_size` | missing |  |
| `stream_set_read_buffer` | missing |  |
| `stream_set_timeout` | missing |  |
| `stream_set_write_buffer` | missing |  |
| `stream_socket_accept` | missing |  |
| `stream_socket_client` | missing |  |
| `stream_socket_enable_crypto` | missing |  |
| `stream_socket_get_name` | missing |  |
| `stream_socket_pair` | missing |  |
| `stream_socket_recvfrom` | missing |  |
| `stream_socket_sendto` | missing |  |
| `stream_socket_server` | missing |  |
| `stream_socket_shutdown` | missing |  |
| `stream_supports_lock` | missing |  |
| `stream_wrapper_register` | missing |  |
| `stream_wrapper_restore` | missing |  |
| `stream_wrapper_unregister` | missing |  |
| `strip_tags` | implemented | Removes NUL bytes and complete `<...>` tag/comment regions for the default no-allowed-tags path; allowed tag preservation and broken-tag edge behavior are deferred. Source: https://www.php.net/manual/en/function.strip-tags.php |
| `stripcslashes` | implemented | Decodes C-style backslash escapes, including common control escapes, octal byte escapes, and two-digit hex byte escapes. Source: https://www.php.net/manual/en/function.stripcslashes.php |
| `stripos` | implemented | Finds the first case-insensitive ASCII byte occurrence and returns an offset or `false`. Source: https://www.php.net/manual/en/function.stripos.php |
| `stripslashes` | implemented | Removes backslash quoting from escaped strings, including PHP's `\0` NUL handling. Source: https://www.php.net/manual/en/function.stripslashes.php |
| `stristr` | implemented | Case-insensitive `strstr()` that returns the matching tail or `false`; before-needle mode is deferred. Source: https://www.php.net/manual/en/function.stristr.php |
| `strnatcasecmp` | implemented | Compares strings in natural order with ASCII case folding. Source: https://www.php.net/manual/en/function.strnatcasecmp.php |
| `strnatcmp` | implemented | Compares strings in natural order so digit runs sort by numeric value. Source: https://www.php.net/manual/en/function.strnatcmp.php |
| `strpbrk` | implemented | Searches for the first byte from a character mask and returns the matching tail or `false`. Source: https://www.php.net/manual/en/function.strpbrk.php |
| `strpos` | implemented | Finds the first binary-safe byte occurrence and returns an offset or `false`. Source: https://www.php.net/manual/en/function.strpos.php |
| `strptime` | missing |  |
| `strrchr` | implemented | Finds the last byte occurrence and returns the matching tail or `false`. Source: https://www.php.net/manual/en/function.strrchr.php |
| `strrev` | implemented | Reverses string bytes without Unicode character interpretation. Source: https://www.php.net/manual/en/function.strrev.php |
| `strripos` | implemented | Finds the last case-insensitive ASCII byte occurrence and returns an offset or `false`. Source: https://www.php.net/manual/en/function.strripos.php |
| `strrpos` | implemented | Finds the last binary-safe byte occurrence and returns an offset or `false`. Source: https://www.php.net/manual/en/function.strrpos.php |
| `strspn` | implemented | Counts the initial byte span containing only bytes from the mask string. Source: https://www.php.net/manual/en/function.strspn.php |
| `strstr` | implemented | Finds a byte string and returns the matching tail or `false`; before-needle mode is deferred. Source: https://www.php.net/manual/en/function.strstr.php |
| `strtok` | missing |  |
| `strtolower` | implemented | Lowercases ASCII alphabetic bytes and leaves non-ASCII bytes unchanged. Source: https://www.php.net/manual/en/function.strtolower.php |
| `strtoupper` | implemented | Uppercases ASCII alphabetic bytes and leaves non-ASCII bytes unchanged. Source: https://www.php.net/manual/en/function.strtoupper.php |
| `strtr` | implemented | Supports the three-argument byte translation form and ignores extra bytes in longer `from` or `to` strings; two-argument array replacement is deferred. Source: https://www.php.net/manual/en/function.strtr.php |
| `strval` | implemented | Converts current scalar values to PHP-style strings, including bool and null stringification. Source: https://www.php.net/manual/en/function.strval.php |
| `substr` | implemented | Returns byte substrings with positive and negative offsets; optional length support is deferred in the scalar path. Source: https://www.php.net/manual/en/function.substr.php |
| `substr_compare` | implemented | Compares a substring window against another string with optional length and ASCII case-insensitive mode. Source: https://www.php.net/manual/en/function.substr-compare.php |
| `substr_count` | implemented | Counts non-overlapping byte substring occurrences. Source: https://www.php.net/manual/en/function.substr-count.php |
| `substr_replace` | implemented | Supports scalar string replacement with positive and negative offsets, omitted length, zero-length insertion, and negative length windows; array operands are deferred. Source: https://www.php.net/manual/en/function.substr-replace.php |
| `symlink` | implemented | Creates local symbolic links and returns a bool success value; Windows-specific target-type behavior, privilege differences, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.symlink.php |
| `sys_get_temp_dir` | implemented | Returns the host process temporary directory path; PHP INI overrides and virtual-host/open_basedir behavior are deferred. Source: https://www.php.net/manual/en/function.sys-get-temp-dir.php |
| `sys_getloadavg` | missing |  |
| `syslog` | missing |  |
| `system` | missing |  |
| `tan` | implemented | Returns the tangent of a radian value using PHP-compatible float coercion. Source: https://www.php.net/manual/en/function.tan.php |
| `tanh` | implemented | Returns hyperbolic tangent as a float with PHP-compatible numeric coercion. Source: https://www.php.net/manual/en/function.tanh.php |
| `tempnam` | implemented | Creates a local temporary file with a unique name and requested prefix, falling back to the host temp directory when the requested directory cannot be used; PHP notices and Windows prefix truncation are deferred. Source: https://www.php.net/manual/en/function.tempnam.php |
| `time_nanosleep` | missing |  |
| `time_sleep_until` | missing |  |
| `tmpfile` | implemented | Creates a process-local temporary file stream and removes the backing file when closed. Source: https://www.php.net/manual/en/function.tmpfile.php |
| `touch` | implemented | Creates missing local files and sets modification/access timestamps with PHP's default timestamp behavior; PHP warning emission is deferred. Source: https://www.php.net/manual/en/function.touch.php |
| `trim` | implemented | Removes PHP's default leading and trailing ASCII whitespace bytes; custom character masks are deferred. Source: https://www.php.net/manual/en/function.trim.php |
| `uasort` | missing |  |
| `ucfirst` | implemented | Uppercases the first ASCII alphabetic byte of a string and leaves the rest unchanged. Source: https://www.php.net/manual/en/function.ucfirst.php |
| `ucwords` | implemented | Uppercases the first ASCII alphabetic byte of each word using PHP's default separators. Source: https://www.php.net/manual/en/function.ucwords.php |
| `uksort` | missing |  |
| `umask` | missing |  |
| `uniqid` | implemented | Generates a PHP-shaped time-based identifier with optional prefix and entropy suffix; it is not cryptographically secure and does not guarantee uniqueness. Source: https://www.php.net/manual/en/function.uniqid.php |
| `unlink` | implemented | Deletes local file names or symlinks and returns a bool success value; stream contexts and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.unlink.php |
| `unpack` | missing |  |
| `unregister_tick_function` | missing |  |
| `unserialize` | missing |  |
| `urldecode` | implemented | Decodes `%XX` byte escapes and converts `+` to a space for form/query strings. Source: https://www.php.net/manual/en/function.urldecode.php |
| `urlencode` | implemented | Encodes form/query strings with spaces as `+` and non-alphanumerics except `-_.` as `%XX`. Source: https://www.php.net/manual/en/function.urlencode.php |
| `usleep` | missing |  |
| `usort` | missing |  |
| `utf8_decode` | missing |  |
| `utf8_encode` | missing |  |
| `var_dump` | missing |  |
| `var_export` | missing |  |
| `version_compare` | missing |  |
| `vfprintf` | missing |  |
| `vprintf` | missing |  |
| `vsprintf` | missing |  |
| `wordwrap` | implemented | Wraps strings at word boundaries with optional width, break string, and long-word cutting. Source: https://www.php.net/manual/en/function.wordwrap.php |

## Optional Extension Counts

These are excluded from the baseline estimate for now. They become separate
compatibility tracks if Echo chooses to support the corresponding extension
surface.

| Extension | Functions |
| --- | ---: |
| `PDO` | 1 |
| `SPL` | 15 |
| `SimpleXML` | 3 |
| `Zend OPcache` | 8 |
| `bcmath` | 14 |
| `ctype` | 11 |
| `curl` | 35 |
| `date` | 48 |
| `dom` | 2 |
| `fileinfo` | 6 |
| `filter` | 7 |
| `hash` | 20 |
| `iconv` | 10 |
| `igbinary` | 2 |
| `intl` | 187 |
| `json` | 5 |
| `libxml` | 8 |
| `mbstring` | 65 |
| `openssl` | 66 |
| `pcntl` | 29 |
| `pcre` | 11 |
| `pgsql` | 123 |
| `posix` | 41 |
| `random` | 9 |
| `readline` | 13 |
| `session` | 23 |
| `tokenizer` | 2 |
| `xdebug` | 41 |
| `xml` | 22 |
| `xmlwriter` | 42 |
| `zip` | 10 |
| `zlib` | 30 |
