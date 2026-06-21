# PHP Compatibility Inventory

This document tracks PHP builtin function parity at function granularity. It is
a planning inventory, not a promise that all rows have equal implementation
cost. Some functions are syntax-adjacent, runtime-global, filesystem/process
backed, or require classes/resources before they can be implemented correctly.

Snapshot source: `get_defined_functions()["internal"]` from local PHP `8.5.6`
on 2026-06-17. PHP reports many ordinary-looking functions through the
`standard` extension; Echo treats `Core` plus `standard` as the baseline
compatibility surface and keeps other PHP extensions optional unless we
explicitly promote one.

## Syntax-Adjacent Compatibility

These PHP constructs are not reported by `get_defined_functions()`, but they
belong in the same compatibility planning surface because they interact with
filesystem loading, runtime process state, diagnostics, and related builtins.

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
| assignment expressions | partial | Assignments are expressions and evaluate to the assigned value. Source: https://www.php.net/manual/en/language.operators.assignment.php |
| `__DIR__` | partial | File-backed `xo` compilation resolves it from the canonical source path. Source: https://www.php.net/manual/en/language.constants.magic.php |
| `define` | partial | Echo accepts runtime constant definitions for bootstrap compatibility, but constant lookup is not implemented yet. Source: https://www.php.net/manual/en/function.define.php |
| `microtime` | implemented | Supports string and float forms for current wall-clock time. Source: https://www.php.net/manual/en/function.microtime.php |

## Totals

| Surface | Functions | Implemented | Remaining |
| --- | ---: | ---: | ---: |
| Baseline (`Core` + `standard`) | 607 | 132 | 475 |
| Loaded local PHP internals, including extensions | 1516 | 132 | 1384 |

## Baseline Functions

### Core (6/62)

| Function | Status | Notes |
| --- | --- | --- |
| `class_alias` | missing |  |
| `class_exists` | missing |  |
| `clone` | missing |  |
| `debug_backtrace` | missing |  |
| `debug_print_backtrace` | missing |  |
| `define` | missing |  |
| `defined` | missing |  |
| `die` | missing |  |
| `enum_exists` | missing |  |
| `error_reporting` | missing |  |
| `exit` | missing |  |
| `extension_loaded` | missing |  |
| `func_get_arg` | missing |  |
| `func_get_args` | missing |  |
| `func_num_args` | missing |  |
| `function_exists` | implemented | Recognizes Echo's supported internal PHP builtin names, case-insensitively; user-defined function registry support is deferred. Source: https://www.php.net/manual/en/function.function-exists.php |
| `gc_collect_cycles` | missing |  |
| `gc_disable` | missing |  |
| `gc_enable` | missing |  |
| `gc_enabled` | missing |  |
| `gc_mem_caches` | missing |  |
| `gc_status` | missing |  |
| `get_called_class` | missing |  |
| `get_class` | missing |  |
| `get_class_methods` | missing |  |
| `get_class_vars` | missing |  |
| `get_declared_classes` | missing |  |
| `get_declared_interfaces` | missing |  |
| `get_declared_traits` | missing |  |
| `get_defined_constants` | missing |  |
| `get_defined_functions` | missing |  |
| `get_defined_vars` | missing |  |
| `get_error_handler` | missing |  |
| `get_exception_handler` | missing |  |
| `get_extension_funcs` | missing |  |
| `get_included_files` | missing |  |
| `get_loaded_extensions` | missing |  |
| `get_mangled_object_vars` | missing |  |
| `get_object_vars` | missing |  |
| `get_parent_class` | missing |  |
| `get_required_files` | missing |  |
| `get_resource_id` | missing |  |
| `get_resource_type` | missing |  |
| `get_resources` | missing |  |
| `interface_exists` | missing |  |
| `is_a` | missing |  |
| `is_subclass_of` | missing |  |
| `method_exists` | missing |  |
| `property_exists` | missing |  |
| `restore_error_handler` | missing |  |
| `restore_exception_handler` | missing |  |
| `set_error_handler` | missing |  |
| `set_exception_handler` | missing |  |
| `strcasecmp` | implemented |  |
| `strcmp` | implemented |  |
| `strlen` | implemented |  |
| `strncasecmp` | implemented | Source: https://www.php.net/manual/en/function.strncasecmp.php |
| `strncmp` | implemented | Source: https://www.php.net/manual/en/function.strncmp.php |
| `trait_exists` | missing |  |
| `trigger_error` | missing |  |
| `user_error` | missing |  |
| `zend_version` | missing |  |

### standard (109/545)

| Function | Status | Notes |
| --- | --- | --- |
| `abs` | implemented | Supports current Echo integer values; float payloads are deferred. Source: https://www.php.net/manual/en/function.abs.php |
| `acos` | implemented | Returns the arc cosine in radians using PHP-compatible float coercion. Source: https://www.php.net/manual/en/function.acos.php |
| `acosh` | implemented | Returns inverse hyperbolic cosine as a float with PHP-compatible numeric coercion and `NAN` outside the domain. Source: https://www.php.net/manual/en/function.acosh.php |
| `addcslashes` | missing |  |
| `addslashes` | implemented |  |
| `array_all` | missing |  |
| `array_any` | missing |  |
| `array_change_key_case` | missing |  |
| `array_chunk` | missing |  |
| `array_column` | missing |  |
| `array_combine` | missing |  |
| `array_count_values` | missing |  |
| `array_diff` | missing |  |
| `array_diff_assoc` | missing |  |
| `array_diff_key` | missing |  |
| `array_diff_uassoc` | missing |  |
| `array_diff_ukey` | missing |  |
| `array_fill` | missing |  |
| `array_fill_keys` | missing |  |
| `array_filter` | missing |  |
| `array_find` | missing |  |
| `array_find_key` | missing |  |
| `array_first` | missing |  |
| `array_flip` | missing |  |
| `array_intersect` | missing |  |
| `array_intersect_assoc` | missing |  |
| `array_intersect_key` | missing |  |
| `array_intersect_uassoc` | missing |  |
| `array_intersect_ukey` | missing |  |
| `array_is_list` | implemented | Echo PHP arrays are currently contiguous vectors; associative/key-gap arrays are deferred. Source: https://www.php.net/manual/en/function.array-is-list.php |
| `array_key_exists` | missing |  |
| `array_key_first` | missing |  |
| `array_key_last` | missing |  |
| `array_keys` | missing |  |
| `array_last` | missing |  |
| `array_map` | missing |  |
| `array_merge` | missing |  |
| `array_merge_recursive` | missing |  |
| `array_multisort` | missing |  |
| `array_pad` | missing |  |
| `array_pop` | missing |  |
| `array_product` | missing |  |
| `array_push` | missing |  |
| `array_rand` | missing |  |
| `array_reduce` | missing |  |
| `array_replace` | missing |  |
| `array_replace_recursive` | missing |  |
| `array_reverse` | missing |  |
| `array_search` | missing |  |
| `array_shift` | missing |  |
| `array_slice` | missing |  |
| `array_splice` | missing |  |
| `array_sum` | missing |  |
| `array_udiff` | missing |  |
| `array_udiff_assoc` | missing |  |
| `array_udiff_uassoc` | missing |  |
| `array_uintersect` | missing |  |
| `array_uintersect_assoc` | missing |  |
| `array_uintersect_uassoc` | missing |  |
| `array_unique` | missing |  |
| `array_unshift` | missing |  |
| `array_values` | missing |  |
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
| `base64_decode` | implemented |  |
| `base64_encode` | implemented |  |
| `base_convert` | implemented | Converts strings between bases 2 through 36, ignoring characters outside the source base for current supported values. Source: https://www.php.net/manual/en/function.base-convert.php |
| `basename` | implemented | Supports Unix-style `/` separators and optional suffix stripping; Windows `\` separator behavior is deferred. Source: https://www.php.net/manual/en/function.basename.php |
| `bin2hex` | implemented |  |
| `bindec` | implemented | Converts binary strings to unsigned decimal int or float values while ignoring non-binary characters. Source: https://www.php.net/manual/en/function.bindec.php |
| `boolval` | implemented |  |
| `call_user_func` | missing |  |
| `call_user_func_array` | missing |  |
| `ceil` | implemented | Rounds numeric values up while returning a float, including PHP-compatible scalar coercion and negative zero behavior. Source: https://www.php.net/manual/en/function.ceil.php |
| `chdir` | implemented | Changes the process current working directory and returns a bool success value; PHP warning emission on failure is deferred. Source: https://www.php.net/manual/en/function.chdir.php |
| `checkdnsrr` | missing |  |
| `chgrp` | missing |  |
| `chmod` | missing |  |
| `chop` | missing |  |
| `chown` | missing |  |
| `chr` | implemented |  |
| `chroot` | missing |  |
| `chunk_split` | missing |  |
| `clearstatcache` | missing |  |
| `cli_get_process_title` | missing |  |
| `cli_set_process_title` | missing |  |
| `closedir` | missing |  |
| `closelog` | missing |  |
| `compact` | missing |  |
| `connection_aborted` | missing |  |
| `connection_status` | missing |  |
| `constant` | missing |  |
| `convert_uudecode` | missing |  |
| `convert_uuencode` | missing |  |
| `copy` | missing |  |
| `cos` | implemented | Returns the cosine of a radian value using PHP-compatible float coercion. Source: https://www.php.net/manual/en/function.cos.php |
| `cosh` | implemented | Returns hyperbolic cosine as a float with PHP-compatible numeric coercion. Source: https://www.php.net/manual/en/function.cosh.php |
| `count` | implemented | Supports PHP array/list counting; recursive mode and Countable objects are deferred. Source: https://www.php.net/manual/en/function.count.php |
| `count_chars` | missing |  |
| `crc32` | missing |  |
| `crypt` | missing |  |
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
| `exp` | missing |  |
| `explode` | implemented | Splits byte strings into PHP arrays with default, positive, zero, and negative limit behavior; empty-separator `ValueError` is currently surfaced as a runtime error. Source: https://www.php.net/manual/en/function.explode.php |
| `expm1` | missing |  |
| `extract` | missing |  |
| `fclose` | missing |  |
| `fdatasync` | missing |  |
| `fdiv` | missing |  |
| `feof` | missing |  |
| `fflush` | missing |  |
| `fgetc` | missing |  |
| `fgetcsv` | missing |  |
| `fgets` | missing |  |
| `file` | missing |  |
| `file_exists` | implemented | Checks local filesystem paths for files or directories; stat cache, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.file-exists.php |
| `file_get_contents` | missing |  |
| `file_put_contents` | missing |  |
| `fileatime` | missing |  |
| `filectime` | missing |  |
| `filegroup` | missing |  |
| `fileinode` | missing |  |
| `filemtime` | missing |  |
| `fileowner` | missing |  |
| `fileperms` | missing |  |
| `filesize` | implemented | Returns the local file size as an integer and `false` when metadata cannot be read; stat cache, URL wrappers, and PHP warning emission are deferred. Source: https://www.php.net/manual/en/function.filesize.php |
| `filetype` | missing |  |
| `floatval` | implemented | Gets the float value of current scalar values, including numeric-prefix string parsing. Source: https://www.php.net/manual/en/function.floatval.php |
| `flock` | missing |  |
| `floor` | implemented | Rounds numeric values down while returning a float, using PHP-compatible scalar coercion. Source: https://www.php.net/manual/en/function.floor.php |
| `flush` | missing |  |
| `fmod` | implemented | Returns a floating-point remainder with the dividend sign and `NAN` for zero divisors. Source: https://www.php.net/manual/en/function.fmod.php |
| `fnmatch` | missing |  |
| `fopen` | missing |  |
| `forward_static_call` | missing |  |
| `forward_static_call_array` | missing |  |
| `fpassthru` | missing |  |
| `fpow` | missing |  |
| `fprintf` | missing |  |
| `fputcsv` | missing |  |
| `fputs` | missing |  |
| `fread` | missing |  |
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
| `get_cfg_var` | missing |  |
| `get_current_user` | missing |  |
| `get_debug_type` | missing |  |
| `get_headers` | missing |  |
| `get_html_translation_table` | missing |  |
| `get_include_path` | missing |  |
| `get_meta_tags` | missing |  |
| `getcwd` | implemented | Returns the current working directory as a string or `false` if the host cannot report it. Source: https://www.php.net/manual/en/function.getcwd.php |
| `getenv` | missing |  |
| `gethostbyaddr` | missing |  |
| `gethostbyname` | missing |  |
| `gethostbynamel` | missing |  |
| `gethostname` | missing |  |
| `getimagesize` | missing |  |
| `getimagesizefromstring` | missing |  |
| `getlastmod` | missing |  |
| `getmxrr` | missing |  |
| `getmygid` | missing |  |
| `getmyinode` | missing |  |
| `getmypid` | missing |  |
| `getmyuid` | missing |  |
| `getopt` | missing |  |
| `getprotobyname` | missing |  |
| `getprotobynumber` | missing |  |
| `getrusage` | missing |  |
| `getservbyname` | missing |  |
| `getservbyport` | missing |  |
| `gettimeofday` | missing |  |
| `gettype` | implemented | Returns PHP type names for Echo's current value tags. Source: https://www.php.net/manual/en/function.gettype.php |
| `glob` | missing |  |
| `header` | missing |  |
| `header_register_callback` | missing |  |
| `header_remove` | missing |  |
| `headers_list` | missing |  |
| `headers_sent` | missing |  |
| `hebrev` | missing |  |
| `hex2bin` | implemented |  |
| `hexdec` | implemented | Converts hexadecimal strings to unsigned decimal int or float values while ignoring non-hexadecimal characters. Source: https://www.php.net/manual/en/function.hexdec.php |
| `highlight_file` | missing |  |
| `highlight_string` | missing |  |
| `hrtime` | missing |  |
| `html_entity_decode` | missing |  |
| `htmlentities` | missing |  |
| `htmlspecialchars` | missing |  |
| `htmlspecialchars_decode` | missing |  |
| `http_build_query` | missing |  |
| `http_clear_last_response_headers` | missing |  |
| `http_get_last_response_headers` | missing |  |
| `http_response_code` | missing |  |
| `hypot` | implemented | Returns the Euclidean distance for two numeric values using PHP-compatible float coercion. Source: https://www.php.net/manual/en/function.hypot.php |
| `ignore_user_abort` | missing |  |
| `image_type_to_extension` | missing |  |
| `image_type_to_mime_type` | missing |  |
| `implode` | implemented | Joins PHP array values in order, supports the optional empty-string separator form, and uses PHP string coercion for scalar elements. Source: https://www.php.net/manual/en/function.implode.php |
| `in_array` | missing |  |
| `inet_ntop` | missing |  |
| `inet_pton` | missing |  |
| `ini_alter` | missing |  |
| `ini_get` | missing |  |
| `ini_get_all` | missing |  |
| `ini_parse_quantity` | missing |  |
| `ini_restore` | missing |  |
| `ini_set` | missing |  |
| `intdiv` | missing |  |
| `intval` | implemented |  |
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
| `key_exists` | missing |  |
| `krsort` | missing |  |
| `ksort` | missing |  |
| `lcfirst` | implemented |  |
| `lchgrp` | missing |  |
| `lchown` | missing |  |
| `levenshtein` | missing |  |
| `link` | missing |  |
| `linkinfo` | missing |  |
| `localeconv` | missing |  |
| `log` | missing |  |
| `log10` | missing |  |
| `log1p` | missing |  |
| `long2ip` | missing |  |
| `lstat` | missing |  |
| `ltrim` | implemented |  |
| `mail` | missing |  |
| `max` | missing |  |
| `md5` | missing |  |
| `md5_file` | missing |  |
| `memory_get_peak_usage` | missing |  |
| `memory_get_usage` | missing |  |
| `memory_reset_peak_usage` | missing |  |
| `metaphone` | missing |  |
| `microtime` | missing |  |
| `min` | missing |  |
| `mkdir` | missing |  |
| `move_uploaded_file` | missing |  |
| `natcasesort` | missing |  |
| `natsort` | missing |  |
| `net_get_interfaces` | missing |  |
| `next` | missing |  |
| `nl2br` | missing |  |
| `nl_langinfo` | missing |  |
| `number_format` | missing |  |
| `ob_clean` | implemented |  |
| `ob_end_clean` | implemented |  |
| `ob_end_flush` | implemented |  |
| `ob_flush` | implemented |  |
| `ob_get_clean` | implemented |  |
| `ob_get_contents` | implemented |  |
| `ob_get_flush` | implemented |  |
| `ob_get_length` | implemented |  |
| `ob_get_level` | implemented |  |
| `ob_get_status` | missing |  |
| `ob_implicit_flush` | missing |  |
| `ob_list_handlers` | missing |  |
| `ob_start` | implemented |  |
| `octdec` | implemented | Converts octal strings to unsigned decimal int or float values while ignoring non-octal characters. Source: https://www.php.net/manual/en/function.octdec.php |
| `opendir` | missing |  |
| `openlog` | missing |  |
| `ord` | implemented |  |
| `output_add_rewrite_var` | missing |  |
| `output_reset_rewrite_vars` | missing |  |
| `pack` | missing |  |
| `parse_ini_file` | missing |  |
| `parse_ini_string` | missing |  |
| `parse_str` | missing |  |
| `parse_url` | missing |  |
| `passthru` | missing |  |
| `password_algos` | missing |  |
| `password_get_info` | missing |  |
| `password_hash` | missing |  |
| `password_needs_rehash` | missing |  |
| `password_verify` | missing |  |
| `pathinfo` | missing |  |
| `pclose` | missing |  |
| `pfsockopen` | missing |  |
| `php_ini_loaded_file` | missing |  |
| `php_ini_scanned_files` | missing |  |
| `php_sapi_name` | missing |  |
| `php_strip_whitespace` | missing |  |
| `php_uname` | missing |  |
| `phpcredits` | missing |  |
| `phpinfo` | missing |  |
| `phpversion` | missing |  |
| `pi` | implemented | Returns an approximation of pi as a float. Source: https://www.php.net/manual/en/function.pi.php |
| `popen` | missing |  |
| `pos` | missing |  |
| `pow` | missing |  |
| `prev` | missing |  |
| `print_r` | missing |  |
| `printf` | missing |  |
| `proc_close` | missing |  |
| `proc_get_status` | missing |  |
| `proc_nice` | missing |  |
| `proc_open` | missing |  |
| `proc_terminate` | missing |  |
| `putenv` | missing |  |
| `quoted_printable_decode` | missing |  |
| `quoted_printable_encode` | missing |  |
| `quotemeta` | implemented |  |
| `rad2deg` | implemented | Converts radians to degrees using PHP-compatible float coercion for current scalar values. Source: https://www.php.net/manual/en/function.rad2deg.php |
| `range` | missing |  |
| `rawurldecode` | implemented | Decodes `%XX` byte escapes without converting `+` to a space. Source: https://www.php.net/manual/en/function.rawurldecode.php |
| `rawurlencode` | implemented | Encodes bytes according to RFC 3986, preserving alphanumerics and `-_.~`. Source: https://www.php.net/manual/en/function.rawurlencode.php |
| `readdir` | missing |  |
| `readfile` | missing |  |
| `readlink` | missing |  |
| `realpath` | implemented | Resolves existing local paths through OS canonicalization and returns `false` for missing paths; realpath cache APIs and URL wrappers are deferred. Source: https://www.php.net/manual/en/function.realpath.php |
| `realpath_cache_get` | missing |  |
| `realpath_cache_size` | missing |  |
| `register_shutdown_function` | missing |  |
| `register_tick_function` | missing |  |
| `rename` | missing |  |
| `request_parse_body` | missing |  |
| `restore_include_path` | missing | Source: https://www.php.net/manual/en/function.restore-include-path.php |
| `reset` | missing |  |
| `rewind` | missing |  |
| `rewinddir` | missing |  |
| `rmdir` | missing |  |
| `round` | missing |  |
| `rsort` | missing |  |
| `rtrim` | implemented |  |
| `scandir` | missing |  |
| `serialize` | missing |  |
| `set_file_buffer` | missing |  |
| `set_include_path` | missing |  |
| `set_time_limit` | missing |  |
| `setcookie` | missing |  |
| `setlocale` | missing |  |
| `setrawcookie` | missing |  |
| `settype` | missing |  |
| `sha1` | missing |  |
| `sha1_file` | missing |  |
| `shell_exec` | missing |  |
| `show_source` | missing |  |
| `shuffle` | missing |  |
| `similar_text` | missing |  |
| `sin` | implemented | Returns the sine of a radian value using PHP-compatible float coercion. Source: https://www.php.net/manual/en/function.sin.php |
| `sinh` | implemented | Returns hyperbolic sine as a float with PHP-compatible numeric coercion. Source: https://www.php.net/manual/en/function.sinh.php |
| `sizeof` | implemented | Alias of `count()`. Source: https://www.php.net/manual/en/function.sizeof.php |
| `sleep` | missing |  |
| `socket_get_status` | missing |  |
| `socket_set_blocking` | missing |  |
| `socket_set_timeout` | missing |  |
| `sort` | missing |  |
| `soundex` | missing |  |
| `sprintf` | missing |  |
| `sqrt` | implemented | Returns a square root as float and `NAN` for negative inputs using PHP-compatible float coercion. Source: https://www.php.net/manual/en/function.sqrt.php |
| `sscanf` | missing |  |
| `stat` | missing |  |
| `str_contains` | implemented |  |
| `str_decrement` | missing |  |
| `str_ends_with` | implemented |  |
| `str_getcsv` | missing |  |
| `str_increment` | missing |  |
| `str_ireplace` | missing |  |
| `str_pad` | implemented | Pads byte strings on the left, right, or both sides with PHP's default right padding and pad-string truncation behavior. Source: https://www.php.net/manual/en/function.str-pad.php |
| `str_repeat` | implemented |  |
| `str_replace` | missing |  |
| `str_rot13` | implemented |  |
| `str_shuffle` | missing |  |
| `str_split` | missing |  |
| `str_starts_with` | implemented |  |
| `str_word_count` | missing |  |
| `strchr` | implemented | Alias of `strstr`. |
| `strcoll` | missing |  |
| `strcspn` | implemented |  |
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
| `stream_get_contents` | missing |  |
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
| `strip_tags` | missing |  |
| `stripcslashes` | missing |  |
| `stripos` | implemented |  |
| `stripslashes` | implemented |  |
| `stristr` | implemented |  |
| `strnatcasecmp` | missing |  |
| `strnatcmp` | missing |  |
| `strpbrk` | implemented |  |
| `strpos` | implemented |  |
| `strptime` | missing |  |
| `strrchr` | implemented |  |
| `strrev` | implemented |  |
| `strripos` | implemented |  |
| `strrpos` | implemented |  |
| `strspn` | implemented |  |
| `strstr` | implemented |  |
| `strtok` | missing |  |
| `strtolower` | implemented |  |
| `strtoupper` | implemented |  |
| `strtr` | missing |  |
| `strval` | implemented |  |
| `substr` | implemented |  |
| `substr_compare` | implemented |  |
| `substr_count` | implemented |  |
| `substr_replace` | missing |  |
| `symlink` | missing |  |
| `sys_get_temp_dir` | missing |  |
| `sys_getloadavg` | missing |  |
| `syslog` | missing |  |
| `system` | missing |  |
| `tan` | implemented | Returns the tangent of a radian value using PHP-compatible float coercion. Source: https://www.php.net/manual/en/function.tan.php |
| `tanh` | implemented | Returns hyperbolic tangent as a float with PHP-compatible numeric coercion. Source: https://www.php.net/manual/en/function.tanh.php |
| `tempnam` | missing |  |
| `time_nanosleep` | missing |  |
| `time_sleep_until` | missing |  |
| `tmpfile` | missing |  |
| `touch` | missing |  |
| `trim` | implemented |  |
| `uasort` | missing |  |
| `ucfirst` | implemented |  |
| `ucwords` | implemented |  |
| `uksort` | missing |  |
| `umask` | missing |  |
| `uniqid` | missing |  |
| `unlink` | missing |  |
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
| `wordwrap` | missing |  |

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
