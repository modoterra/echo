pub(super) fn symbols() -> Vec<(&'static str, usize)> {
    vec![
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
            "echo_php_file_exists",
            echo_runtime::echo_php_file_exists
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_fnmatch",
            echo_runtime::echo_php_fnmatch
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_chdir",
            echo_runtime::echo_php_chdir
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_chmod",
            echo_runtime::echo_php_chmod
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_getcwd",
            echo_runtime::echo_php_getcwd as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_clearstatcache",
            echo_runtime::echo_php_clearstatcache
                as extern "C" fn(echo_runtime::EchoValue, echo_runtime::EchoValue)
                as usize,
        ),
        (
            "echo_php_realpath_cache_size",
            echo_runtime::echo_php_realpath_cache_size as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_realpath_cache_get",
            echo_runtime::echo_php_realpath_cache_get as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
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
            "echo_php_fopen",
            echo_runtime::echo_php_fopen
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_fread",
            echo_runtime::echo_php_fread
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_fclose",
            echo_runtime::echo_php_fclose
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_stream_get_contents",
            echo_runtime::echo_php_stream_get_contents
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_tmpfile",
            echo_runtime::echo_php_tmpfile as extern "C" fn() -> echo_runtime::EchoValue as usize,
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
            "echo_php_pathinfo",
            echo_runtime::echo_php_pathinfo
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
    ]
}
