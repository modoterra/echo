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
            "echo_php_php_strip_whitespace",
            echo_runtime::echo_php_php_strip_whitespace
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_disk_free_space",
            echo_runtime::echo_php_disk_free_space
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_disk_total_space",
            echo_runtime::echo_php_disk_total_space
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
            "echo_php_glob",
            echo_runtime::echo_php_glob
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_scandir",
            echo_runtime::echo_php_scandir
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
            "echo_php_umask",
            echo_runtime::echo_php_umask
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
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
            "echo_php_is_uploaded_file",
            echo_runtime::echo_php_is_uploaded_file
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
            "echo_php_lstat",
            echo_runtime::echo_php_lstat
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_stat",
            echo_runtime::echo_php_stat
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
            "echo_php_fgetc",
            echo_runtime::echo_php_fgetc
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_fgets",
            echo_runtime::echo_php_fgets
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_feof",
            echo_runtime::echo_php_feof
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_fflush",
            echo_runtime::echo_php_fflush
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_fdatasync",
            echo_runtime::echo_php_fdatasync
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_fwrite",
            echo_runtime::echo_php_fwrite
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_fpassthru",
            echo_runtime::echo_php_fpassthru
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_fstat",
            echo_runtime::echo_php_fstat
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_fsync",
            echo_runtime::echo_php_fsync
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ftruncate",
            echo_runtime::echo_php_ftruncate
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
            "echo_php_ftell",
            echo_runtime::echo_php_ftell
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_fseek",
            echo_runtime::echo_php_fseek
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_rewind",
            echo_runtime::echo_php_rewind
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
            "echo_php_stream_get_filters",
            echo_runtime::echo_php_stream_get_filters as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_stream_get_line",
            echo_runtime::echo_php_stream_get_line
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_stream_get_meta_data",
            echo_runtime::echo_php_stream_get_meta_data
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_stream_get_transports",
            echo_runtime::echo_php_stream_get_transports
                as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_stream_get_wrappers",
            echo_runtime::echo_php_stream_get_wrappers as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_stream_is_local",
            echo_runtime::echo_php_stream_is_local
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_stream_isatty",
            echo_runtime::echo_php_stream_isatty
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_stream_set_blocking",
            echo_runtime::echo_php_stream_set_blocking
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_stream_set_chunk_size",
            echo_runtime::echo_php_stream_set_chunk_size
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_stream_set_read_buffer",
            echo_runtime::echo_php_stream_set_read_buffer
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_stream_set_timeout",
            echo_runtime::echo_php_stream_set_timeout
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_stream_set_write_buffer",
            echo_runtime::echo_php_stream_set_write_buffer
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_stream_resolve_include_path",
            echo_runtime::echo_php_stream_resolve_include_path
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_stream_supports_lock",
            echo_runtime::echo_php_stream_supports_lock
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
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
            "echo_php_linkinfo",
            echo_runtime::echo_php_linkinfo
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
            "echo_php_move_uploaded_file",
            echo_runtime::echo_php_move_uploaded_file
                as extern "C" fn(
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
