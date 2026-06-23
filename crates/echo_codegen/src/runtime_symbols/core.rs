pub(super) fn symbols() -> Vec<(&'static str, usize)> {
    vec![
        (
            "echo_write",
            echo_runtime::echo_write as unsafe extern "C" fn(*const u8, usize) as usize,
        ),
        (
            "echo_write_value",
            echo_runtime::echo_write_value as unsafe extern "C" fn(echo_runtime::EchoValue)
                as usize,
        ),
        (
            "echo_value_string",
            echo_runtime::echo_value_string
                as unsafe extern "C" fn(*const u8, usize) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_value_add",
            echo_runtime::echo_value_add
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_sub",
            echo_runtime::echo_value_sub
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_mul",
            echo_runtime::echo_value_mul
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_div",
            echo_runtime::echo_value_div
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_mod",
            echo_runtime::echo_value_mod
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_pow",
            echo_runtime::echo_value_pow
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_unary_plus",
            echo_runtime::echo_value_unary_plus
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_value_unary_minus",
            echo_runtime::echo_value_unary_minus
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_value_bool",
            echo_runtime::echo_value_bool as extern "C" fn(echo_runtime::EchoValue) -> bool
                as usize,
        ),
        (
            "echo_value_concat",
            echo_runtime::echo_value_concat
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_identical",
            echo_runtime::echo_value_identical
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_list_new",
            echo_runtime::echo_value_list_new as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_value_list_append",
            echo_runtime::echo_value_list_append
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_array_new",
            echo_runtime::echo_value_array_new as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_value_array_append",
            echo_runtime::echo_value_array_append
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_array_set",
            echo_runtime::echo_value_array_set
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_index_get",
            echo_runtime::echo_value_index_get
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_object_new",
            echo_runtime::echo_value_object_new as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_value_object_set",
            echo_runtime::echo_value_object_set
                as unsafe extern "C" fn(
                    echo_runtime::EchoValue,
                    *const u8,
                    usize,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_object_get",
            echo_runtime::echo_value_object_get
                as unsafe extern "C" fn(
                    echo_runtime::EchoValue,
                    *const u8,
                    usize,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_exit_status",
            echo_runtime::echo_value_exit_status as extern "C" fn(echo_runtime::EchoValue) -> i32
                as usize,
        ),
        (
            "echo_task_defer",
            echo_runtime::echo_task_defer
                as extern "C" fn(
                    Option<echo_runtime::task::EchoTaskCallback>,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_task_run",
            echo_runtime::echo_task_run
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_task_join",
            echo_runtime::echo_task_join
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_task_group_new",
            echo_runtime::echo_task_group_new as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_task_group_add",
            echo_runtime::echo_task_group_add
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_task_group_run_and_join",
            echo_runtime::echo_task_group_run_and_join
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_thread_fork",
            echo_runtime::echo_thread_fork
                as extern "C" fn(
                    Option<echo_runtime::task::EchoTaskCallback>,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_thread_fork_task",
            echo_runtime::echo_thread_fork_task
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_thread_join",
            echo_runtime::echo_thread_join
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_process_spawn",
            echo_runtime::echo_process_spawn
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_process_join",
            echo_runtime::echo_process_join
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_join",
            echo_runtime::echo_join
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_require",
            echo_runtime::echo_php_require
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_require_once",
            echo_runtime::echo_php_require_once
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_task_sleep_current",
            echo_runtime::echo_task_sleep_current
                as extern "C" fn(
                    i64,
                    Option<echo_runtime::task::EchoTaskCallback>,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_time_sleep",
            echo_runtime::echo_time_sleep as extern "C" fn(i64) as usize,
        ),
        (
            "echo_call_function",
            echo_runtime::echo_call_function
                as unsafe extern "C" fn(*const u8, usize) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_reflection_register_function",
            echo_runtime::echo_reflection_register_function
                as unsafe extern "C" fn(*const u8, usize, *const u8, usize, *const u8, usize, i32)
                as usize,
        ),
    ]
}
