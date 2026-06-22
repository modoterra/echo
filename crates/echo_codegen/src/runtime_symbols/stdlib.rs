pub(super) fn symbols() -> Vec<(&'static str, usize)> {
    vec![
        (
            "echo_std_assert_ok",
            echo_runtime::echo_std_assert_ok
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_assert_equals",
            echo_runtime::echo_std_assert_equals
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_std_http_response_text",
            echo_runtime::echo_std_http_response_text
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_http_read_request",
            echo_runtime::echo_std_http_read_request
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_net_listen",
            echo_runtime::echo_std_net_listen
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_net_connect",
            echo_runtime::echo_std_net_connect
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_net_accept",
            echo_runtime::echo_std_net_accept
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_net_read",
            echo_runtime::echo_std_net_read
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_std_net_write",
            echo_runtime::echo_std_net_write
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_std_net_close",
            echo_runtime::echo_std_net_close
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_reflect_exists",
            echo_runtime::echo_std_reflect_exists
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_reflect_params",
            echo_runtime::echo_std_reflect_params
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_reflect_return_type",
            echo_runtime::echo_std_reflect_return_type
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_reflect_type_of",
            echo_runtime::echo_std_reflect_type_of
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
    ]
}
