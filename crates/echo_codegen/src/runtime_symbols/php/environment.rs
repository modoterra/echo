pub(super) fn symbols() -> Vec<(&'static str, usize)> {
    vec![
        (
            "echo_php_define",
            echo_runtime::echo_php_define
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_constant",
            echo_runtime::echo_php_constant
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_defined",
            echo_runtime::echo_php_defined
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_microtime",
            echo_runtime::echo_php_microtime
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_sleep",
            echo_runtime::echo_php_sleep
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_usleep",
            echo_runtime::echo_php_usleep
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_gettimeofday",
            echo_runtime::echo_php_gettimeofday
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_hrtime",
            echo_runtime::echo_php_hrtime
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
            "echo_php_get_current_user",
            echo_runtime::echo_php_get_current_user as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_getmypid",
            echo_runtime::echo_php_getmypid as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_getmyuid",
            echo_runtime::echo_php_getmyuid as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_getmygid",
            echo_runtime::echo_php_getmygid as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_getmyinode",
            echo_runtime::echo_php_getmyinode as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_sys_getloadavg",
            echo_runtime::echo_php_sys_getloadavg as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_cli_get_process_title",
            echo_runtime::echo_php_cli_get_process_title
                as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_cli_set_process_title",
            echo_runtime::echo_php_cli_set_process_title
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_phpversion",
            echo_runtime::echo_php_phpversion
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_php_sapi_name",
            echo_runtime::echo_php_php_sapi_name as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_php_uname",
            echo_runtime::echo_php_php_uname
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_zend_version",
            echo_runtime::echo_php_zend_version as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_extension_loaded",
            echo_runtime::echo_php_extension_loaded
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_get_loaded_extensions",
            echo_runtime::echo_php_get_loaded_extensions
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_get_extension_funcs",
            echo_runtime::echo_php_get_extension_funcs
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_get_cfg_var",
            echo_runtime::echo_php_get_cfg_var
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ini_get",
            echo_runtime::echo_php_ini_get
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ini_get_all",
            echo_runtime::echo_php_ini_get_all
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_ini_parse_quantity",
            echo_runtime::echo_php_ini_parse_quantity
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_get_include_path",
            echo_runtime::echo_php_get_include_path as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_connection_aborted",
            echo_runtime::echo_php_connection_aborted as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_connection_status",
            echo_runtime::echo_php_connection_status as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ignore_user_abort",
            echo_runtime::echo_php_ignore_user_abort
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_headers_list",
            echo_runtime::echo_php_headers_list as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_headers_sent",
            echo_runtime::echo_php_headers_sent as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_header",
            echo_runtime::echo_php_header
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) as usize,
        ),
        (
            "echo_php_header_remove",
            echo_runtime::echo_php_header_remove as extern "C" fn(echo_runtime::EchoValue) as usize,
        ),
        (
            "echo_php_http_response_code",
            echo_runtime::echo_php_http_response_code
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ini_set",
            echo_runtime::echo_php_ini_set
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_ini_alter",
            echo_runtime::echo_php_ini_alter
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_ini_restore",
            echo_runtime::echo_php_ini_restore as extern "C" fn(echo_runtime::EchoValue) as usize,
        ),
        (
            "echo_php_php_ini_loaded_file",
            echo_runtime::echo_php_php_ini_loaded_file as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_php_ini_scanned_files",
            echo_runtime::echo_php_php_ini_scanned_files
                as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_putenv",
            echo_runtime::echo_php_putenv
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
    ]
}
