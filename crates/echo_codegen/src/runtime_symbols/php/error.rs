pub(super) fn symbols() -> Vec<(&'static str, usize)> {
    vec![
        (
            "echo_php_get_error_handler",
            echo_runtime::echo_php_get_error_handler as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_get_exception_handler",
            echo_runtime::echo_php_get_exception_handler
                as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_set_error_handler",
            echo_runtime::echo_php_set_error_handler
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_restore_error_handler",
            echo_runtime::echo_php_restore_error_handler
                as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_set_exception_handler",
            echo_runtime::echo_php_set_exception_handler
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_restore_exception_handler",
            echo_runtime::echo_php_restore_exception_handler
                as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_error_get_last",
            echo_runtime::echo_php_error_get_last as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_error_clear_last",
            echo_runtime::echo_php_error_clear_last as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
    ]
}
