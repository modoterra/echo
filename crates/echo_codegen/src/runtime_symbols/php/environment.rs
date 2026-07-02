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
            "echo_php_time_nanosleep",
            echo_runtime::echo_php_time_nanosleep
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_time_sleep_until",
            echo_runtime::echo_php_time_sleep_until
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_set_time_limit",
            echo_runtime::echo_php_set_time_limit
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
            "echo_php_openlog",
            echo_runtime::echo_php_openlog
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_syslog",
            echo_runtime::echo_php_syslog
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_closelog",
            echo_runtime::echo_php_closelog as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_image_type_to_extension",
            echo_runtime::echo_php_image_type_to_extension
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_image_type_to_mime_type",
            echo_runtime::echo_php_image_type_to_mime_type
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_gethostname",
            echo_runtime::echo_php_gethostname as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_gethostbyname",
            echo_runtime::echo_php_gethostbyname
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_gethostbynamel",
            echo_runtime::echo_php_gethostbynamel
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_gethostbyaddr",
            echo_runtime::echo_php_gethostbyaddr
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_getprotobyname",
            echo_runtime::echo_php_getprotobyname
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_getprotobynumber",
            echo_runtime::echo_php_getprotobynumber
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_getservbyname",
            echo_runtime::echo_php_getservbyname
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_getservbyport",
            echo_runtime::echo_php_getservbyport
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_getrusage",
            echo_runtime::echo_php_getrusage as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_memory_get_usage",
            echo_runtime::echo_php_memory_get_usage as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_memory_get_peak_usage",
            echo_runtime::echo_php_memory_get_peak_usage
                as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_memory_reset_peak_usage",
            echo_runtime::echo_php_memory_reset_peak_usage
                as extern "C" fn() -> echo_runtime::EchoValue as usize,
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
            "echo_php_getlastmod",
            echo_runtime::echo_php_getlastmod as extern "C" fn() -> echo_runtime::EchoValue
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
            "echo_php_shell_exec",
            echo_runtime::echo_php_shell_exec
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
            "echo_php_phpcredits",
            echo_runtime::echo_php_phpcredits
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_phpinfo",
            echo_runtime::echo_php_phpinfo
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_php_uname",
            echo_runtime::echo_php_php_uname
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_version_compare",
            echo_runtime::echo_php_version_compare
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_zend_version",
            echo_runtime::echo_php_zend_version as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_dl",
            echo_runtime::echo_php_dl
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
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
            "echo_php_set_include_path",
            echo_runtime::echo_php_set_include_path
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
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
            "echo_php_http_get_last_response_headers",
            echo_runtime::echo_php_http_get_last_response_headers
                as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_http_clear_last_response_headers",
            echo_runtime::echo_php_http_clear_last_response_headers as extern "C" fn() as usize,
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
            "echo_php_mail",
            echo_runtime::echo_php_mail
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_ip2long",
            echo_runtime::echo_php_ip2long
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_long2ip",
            echo_runtime::echo_php_long2ip
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_inet_pton",
            echo_runtime::echo_php_inet_pton
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_inet_ntop",
            echo_runtime::echo_php_inet_ntop
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
