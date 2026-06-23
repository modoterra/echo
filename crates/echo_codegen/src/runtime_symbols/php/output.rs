pub(super) fn symbols() -> Vec<(&'static str, usize)> {
    vec![
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
    ]
}
