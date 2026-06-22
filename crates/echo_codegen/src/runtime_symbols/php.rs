pub(super) fn symbols() -> Vec<(&'static str, usize)> {
    vec![
        (
            "echo_php_abs",
            echo_runtime::echo_php_abs
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_flush",
            echo_runtime::echo_php_flush as extern "C" fn() as usize,
        ),
        (
            "echo_php_ob_implicit_flush",
            echo_runtime::echo_php_ob_implicit_flush as extern "C" fn(echo_runtime::EchoValue)
                as usize,
        ),
        (
            "echo_php_ob_start",
            echo_runtime::echo_php_ob_start as extern "C" fn() -> bool as usize,
        ),
        (
            "echo_php_ob_start_value",
            echo_runtime::echo_php_ob_start_value as extern "C" fn(echo_runtime::EchoValue) -> bool
                as usize,
        ),
        (
            "echo_php_ob_flush",
            echo_runtime::echo_php_ob_flush as extern "C" fn() -> bool as usize,
        ),
        (
            "echo_php_ob_clean",
            echo_runtime::echo_php_ob_clean as extern "C" fn() -> bool as usize,
        ),
        (
            "echo_php_ob_end_flush",
            echo_runtime::echo_php_ob_end_flush as extern "C" fn() -> bool as usize,
        ),
        (
            "echo_php_ob_end_clean",
            echo_runtime::echo_php_ob_end_clean as extern "C" fn() -> bool as usize,
        ),
        (
            "echo_php_ob_get_clean",
            echo_runtime::echo_php_ob_get_clean as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ob_get_contents",
            echo_runtime::echo_php_ob_get_contents as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ob_get_flush",
            echo_runtime::echo_php_ob_get_flush as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ob_get_level",
            echo_runtime::echo_php_ob_get_level as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ob_get_length",
            echo_runtime::echo_php_ob_get_length as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_strlen",
            echo_runtime::echo_php_strlen
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_define",
            echo_runtime::echo_php_define
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_microtime",
            echo_runtime::echo_php_microtime
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_getenv",
            echo_runtime::echo_php_getenv
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_gethostname",
            echo_runtime::echo_php_gethostname as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_getmypid",
            echo_runtime::echo_php_getmypid as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_putenv",
            echo_runtime::echo_php_putenv
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_count",
            echo_runtime::echo_php_count
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_array_values",
            echo_runtime::echo_php_array_values
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_array_keys",
            echo_runtime::echo_php_array_keys
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_array_fill",
            echo_runtime::echo_php_array_fill
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_array_fill_keys",
            echo_runtime::echo_php_array_fill_keys
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_array_combine",
            echo_runtime::echo_php_array_combine
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_array_pad",
            echo_runtime::echo_php_array_pad
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_array_reverse",
            echo_runtime::echo_php_array_reverse
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_array_slice",
            echo_runtime::echo_php_array_slice
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_array_chunk",
            echo_runtime::echo_php_array_chunk
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_array_merge",
            echo_runtime::echo_php_array_merge
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_array_replace",
            echo_runtime::echo_php_array_replace
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_array_flip",
            echo_runtime::echo_php_array_flip
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_array_count_values",
            echo_runtime::echo_php_array_count_values
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_array_key_exists",
            echo_runtime::echo_php_array_key_exists
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_array_key_first",
            echo_runtime::echo_php_array_key_first
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_array_key_last",
            echo_runtime::echo_php_array_key_last
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_in_array",
            echo_runtime::echo_php_in_array
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_array_search",
            echo_runtime::echo_php_array_search
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_array_sum",
            echo_runtime::echo_php_array_sum
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_array_product",
            echo_runtime::echo_php_array_product
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_function_exists",
            echo_runtime::echo_php_function_exists
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_gettype",
            echo_runtime::echo_php_gettype
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_array_is_list",
            echo_runtime::echo_php_array_is_list
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_array",
            echo_runtime::echo_php_is_array
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_countable",
            echo_runtime::echo_php_is_countable
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_iterable",
            echo_runtime::echo_php_is_iterable
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_numeric",
            echo_runtime::echo_php_is_numeric
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_null",
            echo_runtime::echo_php_is_null
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_bool",
            echo_runtime::echo_php_is_bool
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_callable",
            echo_runtime::echo_php_is_callable
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_int",
            echo_runtime::echo_php_is_int
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_float",
            echo_runtime::echo_php_is_float
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_finite",
            echo_runtime::echo_php_is_finite
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_infinite",
            echo_runtime::echo_php_is_infinite
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_nan",
            echo_runtime::echo_php_is_nan
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_object",
            echo_runtime::echo_php_is_object
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_resource",
            echo_runtime::echo_php_is_resource
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_string",
            echo_runtime::echo_php_is_string
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_scalar",
            echo_runtime::echo_php_is_scalar
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_strval",
            echo_runtime::echo_php_strval
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_boolval",
            echo_runtime::echo_php_boolval
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_intval",
            echo_runtime::echo_php_intval
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_floatval",
            echo_runtime::echo_php_floatval
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_strtoupper",
            echo_runtime::echo_php_strtoupper
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_strtolower",
            echo_runtime::echo_php_strtolower
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ucwords",
            echo_runtime::echo_php_ucwords
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_strrev",
            echo_runtime::echo_php_strrev
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ucfirst",
            echo_runtime::echo_php_ucfirst
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_lcfirst",
            echo_runtime::echo_php_lcfirst
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ord",
            echo_runtime::echo_php_ord
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_str_rot13",
            echo_runtime::echo_php_str_rot13
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_chr",
            echo_runtime::echo_php_chr
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_decbin",
            echo_runtime::echo_php_decbin
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_dechex",
            echo_runtime::echo_php_dechex
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_decoct",
            echo_runtime::echo_php_decoct
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_crc32",
            echo_runtime::echo_php_crc32
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_bindec",
            echo_runtime::echo_php_bindec
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_hexdec",
            echo_runtime::echo_php_hexdec
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_octdec",
            echo_runtime::echo_php_octdec
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_base_convert",
            echo_runtime::echo_php_base_convert
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_deg2rad",
            echo_runtime::echo_php_deg2rad
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_rad2deg",
            echo_runtime::echo_php_rad2deg
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_sin",
            echo_runtime::echo_php_sin
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_cos",
            echo_runtime::echo_php_cos
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_tan",
            echo_runtime::echo_php_tan
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_asin",
            echo_runtime::echo_php_asin
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_acos",
            echo_runtime::echo_php_acos
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_atan",
            echo_runtime::echo_php_atan
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_atan2",
            echo_runtime::echo_php_atan2
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_ceil",
            echo_runtime::echo_php_ceil
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_floor",
            echo_runtime::echo_php_floor
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_sqrt",
            echo_runtime::echo_php_sqrt
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_exp",
            echo_runtime::echo_php_exp
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_expm1",
            echo_runtime::echo_php_expm1
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_log",
            echo_runtime::echo_php_log
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_log10",
            echo_runtime::echo_php_log10
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_log1p",
            echo_runtime::echo_php_log1p
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_pow",
            echo_runtime::echo_php_pow
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_fdiv",
            echo_runtime::echo_php_fdiv
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_fpow",
            echo_runtime::echo_php_fpow
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_hypot",
            echo_runtime::echo_php_hypot
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_pi",
            echo_runtime::echo_php_pi as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_fmod",
            echo_runtime::echo_php_fmod
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_bin2hex",
            echo_runtime::echo_php_bin2hex
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_md5",
            echo_runtime::echo_php_md5
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_sha1",
            echo_runtime::echo_php_sha1
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_base64_encode",
            echo_runtime::echo_php_base64_encode
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_base64_decode",
            echo_runtime::echo_php_base64_decode
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_rawurlencode",
            echo_runtime::echo_php_rawurlencode
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_rawurldecode",
            echo_runtime::echo_php_rawurldecode
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_urlencode",
            echo_runtime::echo_php_urlencode
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_urldecode",
            echo_runtime::echo_php_urldecode
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_sinh",
            echo_runtime::echo_php_sinh
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_cosh",
            echo_runtime::echo_php_cosh
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_tanh",
            echo_runtime::echo_php_tanh
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_asinh",
            echo_runtime::echo_php_asinh
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_acosh",
            echo_runtime::echo_php_acosh
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_atanh",
            echo_runtime::echo_php_atanh
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_basename",
            echo_runtime::echo_php_basename
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_dirname",
            echo_runtime::echo_php_dirname
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_hex2bin",
            echo_runtime::echo_php_hex2bin
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_escapeshellarg",
            echo_runtime::echo_php_escapeshellarg
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_escapeshellcmd",
            echo_runtime::echo_php_escapeshellcmd
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_explode",
            echo_runtime::echo_php_explode
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_implode",
            echo_runtime::echo_php_implode
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_file_exists",
            echo_runtime::echo_php_file_exists
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_chdir",
            echo_runtime::echo_php_chdir
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_getcwd",
            echo_runtime::echo_php_getcwd as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_is_dir",
            echo_runtime::echo_php_is_dir
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_file",
            echo_runtime::echo_php_is_file
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_link",
            echo_runtime::echo_php_is_link
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_readable",
            echo_runtime::echo_php_is_readable
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_writable",
            echo_runtime::echo_php_is_writable
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_executable",
            echo_runtime::echo_php_is_executable
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_filesize",
            echo_runtime::echo_php_filesize
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_fileatime",
            echo_runtime::echo_php_fileatime
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_filectime",
            echo_runtime::echo_php_filectime
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_filemtime",
            echo_runtime::echo_php_filemtime
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_fileinode",
            echo_runtime::echo_php_fileinode
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_fileowner",
            echo_runtime::echo_php_fileowner
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_filegroup",
            echo_runtime::echo_php_filegroup
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_fileperms",
            echo_runtime::echo_php_fileperms
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_filetype",
            echo_runtime::echo_php_filetype
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_file_get_contents",
            echo_runtime::echo_php_file_get_contents
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_file_put_contents",
            echo_runtime::echo_php_file_put_contents
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_readfile",
            echo_runtime::echo_php_readfile
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_readlink",
            echo_runtime::echo_php_readlink
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_link",
            echo_runtime::echo_php_link
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_symlink",
            echo_runtime::echo_php_symlink
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_sys_get_temp_dir",
            echo_runtime::echo_php_sys_get_temp_dir as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_tempnam",
            echo_runtime::echo_php_tempnam
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_uniqid",
            echo_runtime::echo_php_uniqid
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_touch",
            echo_runtime::echo_php_touch
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_copy",
            echo_runtime::echo_php_copy
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_rename",
            echo_runtime::echo_php_rename
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_unlink",
            echo_runtime::echo_php_unlink
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_mkdir",
            echo_runtime::echo_php_mkdir
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_rmdir",
            echo_runtime::echo_php_rmdir
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_realpath",
            echo_runtime::echo_php_realpath
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_trim",
            echo_runtime::echo_php_trim
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ltrim",
            echo_runtime::echo_php_ltrim
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_rtrim",
            echo_runtime::echo_php_rtrim
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_addslashes",
            echo_runtime::echo_php_addslashes
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_stripslashes",
            echo_runtime::echo_php_stripslashes
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_quoted_printable_encode",
            echo_runtime::echo_php_quoted_printable_encode
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_quoted_printable_decode",
            echo_runtime::echo_php_quoted_printable_decode
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_nl2br",
            echo_runtime::echo_php_nl2br
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_quotemeta",
            echo_runtime::echo_php_quotemeta
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_str_contains",
            echo_runtime::echo_php_str_contains
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_str_starts_with",
            echo_runtime::echo_php_str_starts_with
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_str_ends_with",
            echo_runtime::echo_php_str_ends_with
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_str_replace",
            echo_runtime::echo_php_str_replace
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_str_ireplace",
            echo_runtime::echo_php_str_ireplace
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strtr",
            echo_runtime::echo_php_strtr
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_str_repeat",
            echo_runtime::echo_php_str_repeat
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_str_pad",
            echo_runtime::echo_php_str_pad
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_str_split",
            echo_runtime::echo_php_str_split
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_chunk_split",
            echo_runtime::echo_php_chunk_split
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_substr",
            echo_runtime::echo_php_substr
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strpos",
            echo_runtime::echo_php_strpos
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_stripos",
            echo_runtime::echo_php_stripos
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strrpos",
            echo_runtime::echo_php_strrpos
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strripos",
            echo_runtime::echo_php_strripos
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strstr",
            echo_runtime::echo_php_strstr
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_stristr",
            echo_runtime::echo_php_stristr
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strrchr",
            echo_runtime::echo_php_strrchr
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strpbrk",
            echo_runtime::echo_php_strpbrk
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strspn",
            echo_runtime::echo_php_strspn
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strcspn",
            echo_runtime::echo_php_strcspn
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_substr_count",
            echo_runtime::echo_php_substr_count
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_substr_compare",
            echo_runtime::echo_php_substr_compare
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strcmp",
            echo_runtime::echo_php_strcmp
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strcasecmp",
            echo_runtime::echo_php_strcasecmp
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strncmp",
            echo_runtime::echo_php_strncmp
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strncasecmp",
            echo_runtime::echo_php_strncasecmp
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
    ]
}
