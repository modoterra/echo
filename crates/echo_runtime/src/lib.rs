pub mod abi;
mod assertions;
mod callable;
mod collections;
mod crypto;
mod encoding;
mod environment;
pub mod error;
mod execution;
mod filesystem;
mod gc;
pub mod io;
mod math;
pub mod net;
mod output;
pub mod poll;
pub mod process;
mod reflection;
mod require;
pub mod sched;
mod string;
pub mod task;
pub mod task_group;
pub mod thread;
pub mod time;
pub mod value;

pub use assertions::{echo_std_assert_equals, echo_std_assert_ok};
pub use callable::{
    EchoCallable, EchoSymbol, echo_call, echo_call_function, echo_normalize_callable,
    echo_php_constant, echo_php_define, echo_php_defined, echo_php_get_defined_constants,
};
pub use collections::{
    EchoArray, EchoList, echo_php_array_change_key_case, echo_php_array_chunk,
    echo_php_array_column, echo_php_array_combine, echo_php_array_count_values,
    echo_php_array_diff, echo_php_array_diff_assoc, echo_php_array_diff_key, echo_php_array_fill,
    echo_php_array_fill_keys, echo_php_array_filter, echo_php_array_first, echo_php_array_flip,
    echo_php_array_intersect, echo_php_array_intersect_assoc, echo_php_array_intersect_key,
    echo_php_array_is_list, echo_php_array_key_exists, echo_php_array_key_first,
    echo_php_array_key_last, echo_php_array_keys, echo_php_array_last, echo_php_array_merge,
    echo_php_array_pad, echo_php_array_pop, echo_php_array_product, echo_php_array_push,
    echo_php_array_replace, echo_php_array_reverse, echo_php_array_search, echo_php_array_shift,
    echo_php_array_slice, echo_php_array_splice, echo_php_array_sum, echo_php_array_unique,
    echo_php_array_unshift, echo_php_array_values, echo_php_arsort, echo_php_asort, echo_php_count,
    echo_php_current, echo_php_end, echo_php_in_array, echo_php_key, echo_php_krsort,
    echo_php_ksort, echo_php_natcasesort, echo_php_natsort, echo_php_next, echo_php_prev,
    echo_php_range, echo_php_reset, echo_php_rsort, echo_php_sort, echo_value_array_append,
    echo_value_array_key_at, echo_value_array_len, echo_value_array_new, echo_value_array_set,
    echo_value_array_value_at, echo_value_index_get, echo_value_list_append, echo_value_list_new,
};
pub use crypto::{
    echo_php_crypt, echo_php_hash, echo_php_hash_algos, echo_php_hash_copy, echo_php_hash_equals,
    echo_php_hash_file, echo_php_hash_final, echo_php_hash_hkdf, echo_php_hash_hmac,
    echo_php_hash_hmac_algos, echo_php_hash_hmac_file, echo_php_hash_init, echo_php_hash_pbkdf2,
    echo_php_hash_update, echo_php_hash_update_file, echo_php_hash_update_stream,
    echo_php_md5_file, echo_php_password_algos, echo_php_password_get_info, echo_php_password_hash,
    echo_php_password_needs_rehash, echo_php_password_verify, echo_php_random_bytes,
    echo_php_random_int, echo_php_sha1_file,
};
pub use encoding::{
    echo_php_base64_decode, echo_php_base64_encode, echo_php_bin2hex, echo_php_convert_uudecode,
    echo_php_convert_uuencode, echo_php_crc32, echo_php_escapeshellarg, echo_php_escapeshellcmd,
    echo_php_hex2bin, echo_php_http_build_query, echo_php_md5, echo_php_parse_url,
    echo_php_rawurldecode, echo_php_rawurlencode, echo_php_sha1, echo_php_urldecode,
    echo_php_urlencode, echo_php_utf8_decode, echo_php_utf8_encode,
};
pub use environment::*;
pub use error::{
    EchoError, echo_php_error_clear_last, echo_php_error_get_last, echo_php_error_log,
    echo_php_error_reporting, echo_php_get_error_handler, echo_php_get_exception_handler,
    echo_php_restore_error_handler, echo_php_restore_exception_handler, echo_php_set_error_handler,
    echo_php_set_exception_handler, echo_php_trigger_error, echo_php_user_error,
};
pub use execution::echo_join;
pub use filesystem::{
    echo_php_basename, echo_php_chdir, echo_php_chmod, echo_php_clearstatcache, echo_php_copy,
    echo_php_dirname, echo_php_disk_free_space, echo_php_disk_total_space, echo_php_fclose,
    echo_php_fdatasync, echo_php_feof, echo_php_fflush, echo_php_fgetc, echo_php_fgets,
    echo_php_file_exists, echo_php_file_get_contents, echo_php_file_put_contents,
    echo_php_fileatime, echo_php_filectime, echo_php_filegroup, echo_php_fileinode,
    echo_php_filemtime, echo_php_fileowner, echo_php_fileperms, echo_php_filesize,
    echo_php_filetype, echo_php_fnmatch, echo_php_fopen, echo_php_fpassthru, echo_php_fread,
    echo_php_fseek, echo_php_fstat, echo_php_fsync, echo_php_ftell, echo_php_ftruncate,
    echo_php_fwrite, echo_php_getcwd, echo_php_glob, echo_php_is_dir, echo_php_is_executable,
    echo_php_is_file, echo_php_is_link, echo_php_is_readable, echo_php_is_uploaded_file,
    echo_php_is_writable, echo_php_link, echo_php_linkinfo, echo_php_lstat, echo_php_mkdir,
    echo_php_move_uploaded_file, echo_php_pathinfo, echo_php_readfile, echo_php_readlink,
    echo_php_realpath, echo_php_realpath_cache_get, echo_php_realpath_cache_size, echo_php_rename,
    echo_php_rewind, echo_php_rmdir, echo_php_scandir, echo_php_stat, echo_php_stream_get_contents,
    echo_php_stream_get_filters, echo_php_stream_get_transports, echo_php_stream_get_wrappers,
    echo_php_stream_is_local, echo_php_stream_isatty, echo_php_stream_set_blocking,
    echo_php_stream_set_chunk_size, echo_php_stream_set_read_buffer,
    echo_php_stream_set_write_buffer, echo_php_stream_supports_lock, echo_php_symlink,
    echo_php_sys_get_temp_dir, echo_php_tempnam, echo_php_tmpfile, echo_php_touch, echo_php_uniqid,
    echo_php_unlink,
};
pub use gc::{
    echo_php_gc_collect_cycles, echo_php_gc_disable, echo_php_gc_enable, echo_php_gc_enabled,
    echo_php_gc_mem_caches, echo_php_gc_status,
};
pub use math::{
    echo_php_acos, echo_php_acosh, echo_php_asin, echo_php_asinh, echo_php_atan, echo_php_atan2,
    echo_php_atanh, echo_php_ceil, echo_php_cos, echo_php_cosh, echo_php_deg2rad, echo_php_exp,
    echo_php_expm1, echo_php_fdiv, echo_php_floor, echo_php_fmod, echo_php_fpow, echo_php_hypot,
    echo_php_intdiv, echo_php_is_finite, echo_php_is_infinite, echo_php_is_nan, echo_php_log,
    echo_php_log1p, echo_php_log10, echo_php_max, echo_php_min, echo_php_number_format,
    echo_php_pi, echo_php_pow, echo_php_rad2deg, echo_php_round, echo_php_sin, echo_php_sinh,
    echo_php_sqrt, echo_php_tan, echo_php_tanh,
};
pub use net::{
    echo_std_http_read_request, echo_std_http_response_text, echo_std_net_accept,
    echo_std_net_close, echo_std_net_connect, echo_std_net_listen, echo_std_net_read,
    echo_std_net_write,
};
use output::reset_output_runtime;
pub(crate) use output::write_runtime_output;
pub use output::{
    OutputRuntime, echo_php_flush, echo_php_ob_clean, echo_php_ob_end_clean, echo_php_ob_end_flush,
    echo_php_ob_flush, echo_php_ob_get_clean, echo_php_ob_get_contents, echo_php_ob_get_flush,
    echo_php_ob_get_length, echo_php_ob_get_level, echo_php_ob_get_status,
    echo_php_ob_implicit_flush, echo_php_ob_list_handlers, echo_php_ob_start,
    echo_php_ob_start_value, echo_php_output_add_rewrite_var, echo_php_output_reset_rewrite_vars,
    echo_shutdown, echo_write, echo_write_i64, echo_write_i64_or_false, echo_write_string,
    echo_write_value,
};
pub use process::{echo_process_join, echo_process_spawn};
pub use reflection::{
    echo_php_class_exists, echo_php_debug_backtrace, echo_php_debug_print_backtrace,
    echo_php_enum_exists, echo_php_function_exists, echo_php_get_declared_classes,
    echo_php_get_declared_interfaces, echo_php_get_declared_traits, echo_php_get_defined_functions,
    echo_php_interface_exists, echo_php_is_a, echo_php_is_callable, echo_php_is_subclass_of,
    echo_php_method_exists, echo_php_property_exists, echo_php_trait_exists,
    echo_reflection_register_function, echo_std_reflect_exists, echo_std_reflect_params,
    echo_std_reflect_return_type, echo_std_reflect_type_of,
};
pub use require::{
    echo_php_get_included_files, echo_php_get_required_files, echo_php_register_included_file,
    echo_php_require, echo_php_require_once,
};
pub use string::{
    echo_php_addcslashes, echo_php_addslashes, echo_php_chr, echo_php_chunk_split,
    echo_php_count_chars, echo_php_decbin, echo_php_dechex, echo_php_decoct, echo_php_explode,
    echo_php_hebrev, echo_php_html_entity_decode, echo_php_htmlentities, echo_php_htmlspecialchars,
    echo_php_htmlspecialchars_decode, echo_php_implode, echo_php_lcfirst, echo_php_levenshtein,
    echo_php_ltrim, echo_php_nl2br, echo_php_ord, echo_php_quoted_printable_decode,
    echo_php_quoted_printable_encode, echo_php_quotemeta, echo_php_rtrim, echo_php_similar_text,
    echo_php_soundex, echo_php_str_contains, echo_php_str_decrement, echo_php_str_ends_with,
    echo_php_str_getcsv, echo_php_str_increment, echo_php_str_ireplace, echo_php_str_pad,
    echo_php_str_repeat, echo_php_str_replace, echo_php_str_rot13, echo_php_str_split,
    echo_php_str_starts_with, echo_php_str_word_count, echo_php_strcasecmp, echo_php_strcmp,
    echo_php_strcspn, echo_php_strip_tags, echo_php_stripcslashes, echo_php_stripos,
    echo_php_stripslashes, echo_php_stristr, echo_php_strlen, echo_php_strnatcasecmp,
    echo_php_strnatcmp, echo_php_strncasecmp, echo_php_strncmp, echo_php_strpbrk, echo_php_strpos,
    echo_php_strrchr, echo_php_strrev, echo_php_strripos, echo_php_strrpos, echo_php_strspn,
    echo_php_strstr, echo_php_strtolower, echo_php_strtoupper, echo_php_strtr, echo_php_strval,
    echo_php_substr, echo_php_substr_compare, echo_php_substr_count, echo_php_substr_replace,
    echo_php_trim, echo_php_ucfirst, echo_php_ucwords, echo_php_wordwrap, echo_value_concat,
    echo_value_string,
};
pub use task::{echo_task_defer, echo_task_join, echo_task_run, echo_task_sleep_current};
pub use task_group::{echo_task_group_add, echo_task_group_new, echo_task_group_run_and_join};
pub use thread::{echo_thread_fork, echo_thread_fork_task, echo_thread_join};
pub use time::{
    echo_php_gettimeofday, echo_php_hrtime, echo_php_microtime, echo_php_set_time_limit,
    echo_php_sleep, echo_php_time_nanosleep, echo_php_time_sleep_until, echo_php_usleep,
    echo_time_sleep,
};
pub(crate) use value::{
    ECHO_VALUE_ARRAY, ECHO_VALUE_BOOL, ECHO_VALUE_ERROR, ECHO_VALUE_FLOAT, ECHO_VALUE_INT,
    ECHO_VALUE_LIST, ECHO_VALUE_NULL, ECHO_VALUE_PENDING, ECHO_VALUE_PROCESS, ECHO_VALUE_STRING,
    ECHO_VALUE_TASK, ECHO_VALUE_THREAD,
};
pub use value::{
    EchoObject, EchoString, EchoValue, echo_php_abs, echo_php_base_convert, echo_php_bindec,
    echo_php_boolval, echo_php_floatval, echo_php_get_debug_type, echo_php_get_resource_id,
    echo_php_get_resource_type, echo_php_get_resources, echo_php_gettype, echo_php_hexdec,
    echo_php_intval, echo_php_is_array, echo_php_is_bool, echo_php_is_countable, echo_php_is_float,
    echo_php_is_int, echo_php_is_iterable, echo_php_is_null, echo_php_is_numeric,
    echo_php_is_object, echo_php_is_resource, echo_php_is_scalar, echo_php_is_string,
    echo_php_octdec, echo_php_serialize, echo_value_add, echo_value_bool, echo_value_div,
    echo_value_exit_status, echo_value_identical, echo_value_less_than, echo_value_mod,
    echo_value_mul, echo_value_not, echo_value_object_get, echo_value_object_new,
    echo_value_object_set, echo_value_or, echo_value_pow, echo_value_string_equals_ptr,
    echo_value_sub, echo_value_unary_minus, echo_value_unary_plus,
};
pub(crate) use value::{
    PhpNumber, echo_values_equal, format_php_float, php_float_cast, php_number_add, php_number_mul,
    php_values_equal,
};

pub fn echo_is_callable(value: EchoValue) -> bool {
    echo_normalize_callable(value).is_ok_and(|callback| callback.is_some())
}

pub fn reset_execution_state() {
    reset_output_runtime();
    execution::reset();
    assertions::reset();
    error::reset();
}

pub fn capture_stdout<T>(repl_inspect: bool, f: impl FnOnce() -> T) -> (T, Vec<u8>) {
    reset_execution_state();
    execution::begin_capture(repl_inspect);
    let result = f();
    (result, execution::finish_capture())
}

pub(crate) fn echo_runtime_string(bytes: Vec<u8>) -> EchoValue {
    EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes))))
}

#[cfg(test)]
mod runtime_tests;
