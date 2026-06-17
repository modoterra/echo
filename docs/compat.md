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

## Totals

| Surface | Functions | Implemented | Remaining |
| --- | ---: | ---: | ---: |
| Baseline (`Core` + `standard`) | 607 | 41 | 566 |
| Loaded local PHP internals, including extensions | 1516 | 41 | 1475 |

## Baseline Functions

### Core (3/62)

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
| `function_exists` | missing |  |
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
| `strncasecmp` | missing |  |
| `strncmp` | missing |  |
| `trait_exists` | missing |  |
| `trigger_error` | missing |  |
| `user_error` | missing |  |
| `zend_version` | missing |  |

### standard (38/545)

| Function | Status | Notes |
| --- | --- | --- |
| `abs` | missing |  |
| `acos` | missing |  |
| `acosh` | missing |  |
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
| `array_is_list` | missing |  |
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
| `asin` | missing |  |
| `asinh` | missing |  |
| `asort` | missing |  |
| `assert` | missing |  |
| `assert_options` | missing |  |
| `atan` | missing |  |
| `atan2` | missing |  |
| `atanh` | missing |  |
| `base64_decode` | missing |  |
| `base64_encode` | missing |  |
| `base_convert` | missing |  |
| `basename` | missing |  |
| `bin2hex` | implemented |  |
| `bindec` | missing |  |
| `boolval` | missing |  |
| `call_user_func` | missing |  |
| `call_user_func_array` | missing |  |
| `ceil` | missing |  |
| `chdir` | missing |  |
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
| `cos` | missing |  |
| `cosh` | missing |  |
| `count` | missing |  |
| `count_chars` | missing |  |
| `crc32` | missing |  |
| `crypt` | missing |  |
| `current` | missing |  |
| `debug_zval_dump` | missing |  |
| `decbin` | missing |  |
| `dechex` | missing |  |
| `decoct` | missing |  |
| `deg2rad` | missing |  |
| `dir` | missing |  |
| `dirname` | missing |  |
| `disk_free_space` | missing |  |
| `disk_total_space` | missing |  |
| `diskfreespace` | missing |  |
| `dl` | missing |  |
| `dns_check_record` | missing |  |
| `dns_get_mx` | missing |  |
| `dns_get_record` | missing |  |
| `doubleval` | missing |  |
| `end` | missing |  |
| `error_clear_last` | missing |  |
| `error_get_last` | missing |  |
| `error_log` | missing |  |
| `escapeshellarg` | missing |  |
| `escapeshellcmd` | missing |  |
| `exec` | missing |  |
| `exp` | missing |  |
| `explode` | missing |  |
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
| `file_exists` | missing |  |
| `file_get_contents` | missing |  |
| `file_put_contents` | missing |  |
| `fileatime` | missing |  |
| `filectime` | missing |  |
| `filegroup` | missing |  |
| `fileinode` | missing |  |
| `filemtime` | missing |  |
| `fileowner` | missing |  |
| `fileperms` | missing |  |
| `filesize` | missing |  |
| `filetype` | missing |  |
| `floatval` | missing |  |
| `flock` | missing |  |
| `floor` | missing |  |
| `flush` | missing |  |
| `fmod` | missing |  |
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
| `getcwd` | missing |  |
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
| `gettype` | missing |  |
| `glob` | missing |  |
| `header` | missing |  |
| `header_register_callback` | missing |  |
| `header_remove` | missing |  |
| `headers_list` | missing |  |
| `headers_sent` | missing |  |
| `hebrev` | missing |  |
| `hex2bin` | implemented |  |
| `hexdec` | missing |  |
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
| `hypot` | missing |  |
| `ignore_user_abort` | missing |  |
| `image_type_to_extension` | missing |  |
| `image_type_to_mime_type` | missing |  |
| `implode` | missing |  |
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
| `intval` | missing |  |
| `ip2long` | missing |  |
| `iptcembed` | missing |  |
| `iptcparse` | missing |  |
| `is_array` | missing |  |
| `is_bool` | missing |  |
| `is_callable` | missing |  |
| `is_countable` | missing |  |
| `is_dir` | missing |  |
| `is_double` | missing |  |
| `is_executable` | missing |  |
| `is_file` | missing |  |
| `is_finite` | missing |  |
| `is_float` | missing |  |
| `is_infinite` | missing |  |
| `is_int` | missing |  |
| `is_integer` | missing |  |
| `is_iterable` | missing |  |
| `is_link` | missing |  |
| `is_long` | missing |  |
| `is_nan` | missing |  |
| `is_null` | missing |  |
| `is_numeric` | missing |  |
| `is_object` | missing |  |
| `is_readable` | missing |  |
| `is_resource` | missing |  |
| `is_scalar` | missing |  |
| `is_string` | missing |  |
| `is_uploaded_file` | missing |  |
| `is_writable` | missing |  |
| `is_writeable` | missing |  |
| `join` | missing |  |
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
| `octdec` | missing |  |
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
| `pi` | missing |  |
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
| `rad2deg` | missing |  |
| `range` | missing |  |
| `rawurldecode` | missing |  |
| `rawurlencode` | missing |  |
| `readdir` | missing |  |
| `readfile` | missing |  |
| `readlink` | missing |  |
| `realpath` | missing |  |
| `realpath_cache_get` | missing |  |
| `realpath_cache_size` | missing |  |
| `register_shutdown_function` | missing |  |
| `register_tick_function` | missing |  |
| `rename` | missing |  |
| `request_parse_body` | missing |  |
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
| `sin` | missing |  |
| `sinh` | missing |  |
| `sizeof` | missing |  |
| `sleep` | missing |  |
| `socket_get_status` | missing |  |
| `socket_set_blocking` | missing |  |
| `socket_set_timeout` | missing |  |
| `sort` | missing |  |
| `soundex` | missing |  |
| `sprintf` | missing |  |
| `sqrt` | missing |  |
| `sscanf` | missing |  |
| `stat` | missing |  |
| `str_contains` | implemented |  |
| `str_decrement` | missing |  |
| `str_ends_with` | implemented |  |
| `str_getcsv` | missing |  |
| `str_increment` | missing |  |
| `str_ireplace` | missing |  |
| `str_pad` | missing |  |
| `str_repeat` | implemented |  |
| `str_replace` | missing |  |
| `str_rot13` | implemented |  |
| `str_shuffle` | missing |  |
| `str_split` | missing |  |
| `str_starts_with` | implemented |  |
| `str_word_count` | missing |  |
| `strchr` | missing |  |
| `strcoll` | missing |  |
| `strcspn` | missing |  |
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
| `strpbrk` | missing |  |
| `strpos` | implemented |  |
| `strptime` | missing |  |
| `strrchr` | implemented |  |
| `strrev` | implemented |  |
| `strripos` | implemented |  |
| `strrpos` | implemented |  |
| `strspn` | missing |  |
| `strstr` | implemented |  |
| `strtok` | missing |  |
| `strtolower` | implemented |  |
| `strtoupper` | implemented |  |
| `strtr` | missing |  |
| `strval` | missing |  |
| `substr` | implemented |  |
| `substr_compare` | missing |  |
| `substr_count` | missing |  |
| `substr_replace` | missing |  |
| `symlink` | missing |  |
| `sys_get_temp_dir` | missing |  |
| `sys_getloadavg` | missing |  |
| `syslog` | missing |  |
| `system` | missing |  |
| `tan` | missing |  |
| `tanh` | missing |  |
| `tempnam` | missing |  |
| `time_nanosleep` | missing |  |
| `time_sleep_until` | missing |  |
| `tmpfile` | missing |  |
| `touch` | missing |  |
| `trim` | implemented |  |
| `uasort` | missing |  |
| `ucfirst` | implemented |  |
| `ucwords` | missing |  |
| `uksort` | missing |  |
| `umask` | missing |  |
| `uniqid` | missing |  |
| `unlink` | missing |  |
| `unpack` | missing |  |
| `unregister_tick_function` | missing |  |
| `unserialize` | missing |  |
| `urldecode` | missing |  |
| `urlencode` | missing |  |
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
