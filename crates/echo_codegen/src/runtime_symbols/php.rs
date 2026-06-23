mod array;
mod environment;
mod math;
mod output;
mod types;

pub(super) fn symbols() -> Vec<(&'static str, usize)> {
    let mut symbols = vec![
        (
            "echo_php_abs",
            echo_runtime::echo_php_abs
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_strlen",
            echo_runtime::echo_php_strlen
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
    ];
    symbols.extend(array::symbols());
    symbols.extend(environment::symbols());
    symbols.extend(math::symbols());
    symbols.extend(output::symbols());
    symbols.extend(types::symbols());
    symbols
}
