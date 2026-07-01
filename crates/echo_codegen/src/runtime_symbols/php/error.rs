pub(super) fn symbols() -> Vec<(&'static str, usize)> {
    vec![(
        "echo_php_get_error_handler",
        echo_runtime::echo_php_get_error_handler as extern "C" fn() -> echo_runtime::EchoValue
            as usize,
    )]
}
