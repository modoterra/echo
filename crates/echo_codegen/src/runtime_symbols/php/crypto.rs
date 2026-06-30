pub(super) fn symbols() -> Vec<(&'static str, usize)> {
    vec![
        (
            "echo_php_crypt",
            echo_runtime::echo_php_crypt
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_hash",
            echo_runtime::echo_php_hash
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_hash_algos",
            echo_runtime::echo_php_hash_algos as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_hash_copy",
            echo_runtime::echo_php_hash_copy
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_hash_equals",
            echo_runtime::echo_php_hash_equals
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_hash_file",
            echo_runtime::echo_php_hash_file
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_hash_final",
            echo_runtime::echo_php_hash_final
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_hash_hkdf",
            echo_runtime::echo_php_hash_hkdf
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_hash_hmac",
            echo_runtime::echo_php_hash_hmac
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_hash_hmac_algos",
            echo_runtime::echo_php_hash_hmac_algos as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_hash_hmac_file",
            echo_runtime::echo_php_hash_hmac_file
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_hash_init",
            echo_runtime::echo_php_hash_init
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_hash_pbkdf2",
            echo_runtime::echo_php_hash_pbkdf2
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_hash_update",
            echo_runtime::echo_php_hash_update
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_hash_update_file",
            echo_runtime::echo_php_hash_update_file
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_hash_update_stream",
            echo_runtime::echo_php_hash_update_stream
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_md5_file",
            echo_runtime::echo_php_md5_file
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_password_algos",
            echo_runtime::echo_php_password_algos as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_password_get_info",
            echo_runtime::echo_php_password_get_info
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_password_hash",
            echo_runtime::echo_php_password_hash
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_password_needs_rehash",
            echo_runtime::echo_php_password_needs_rehash
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_password_verify",
            echo_runtime::echo_php_password_verify
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_random_bytes",
            echo_runtime::echo_php_random_bytes
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_random_int",
            echo_runtime::echo_php_random_int
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_sha1_file",
            echo_runtime::echo_php_sha1_file
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
    ]
}
