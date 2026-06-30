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
