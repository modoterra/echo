pub(super) fn symbols() -> Vec<(&'static str, usize)> {
    vec![
        (
            "echo_php_function_exists",
            echo_runtime::echo_php_function_exists
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_error_reporting",
            echo_runtime::echo_php_error_reporting
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_gc_collect_cycles",
            echo_runtime::echo_php_gc_collect_cycles as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_gc_disable",
            echo_runtime::echo_php_gc_disable as extern "C" fn() as usize,
        ),
        (
            "echo_php_gc_enable",
            echo_runtime::echo_php_gc_enable as extern "C" fn() as usize,
        ),
        (
            "echo_php_gc_enabled",
            echo_runtime::echo_php_gc_enabled as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_gc_mem_caches",
            echo_runtime::echo_php_gc_mem_caches as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_gc_status",
            echo_runtime::echo_php_gc_status as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_class_exists",
            echo_runtime::echo_php_class_exists
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_interface_exists",
            echo_runtime::echo_php_interface_exists
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_trait_exists",
            echo_runtime::echo_php_trait_exists
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_enum_exists",
            echo_runtime::echo_php_enum_exists
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_get_defined_functions",
            echo_runtime::echo_php_get_defined_functions
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_get_declared_classes",
            echo_runtime::echo_php_get_declared_classes
                as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_get_declared_interfaces",
            echo_runtime::echo_php_get_declared_interfaces
                as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_get_declared_traits",
            echo_runtime::echo_php_get_declared_traits as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_gettype",
            echo_runtime::echo_php_gettype
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_array_is_list",
            echo_runtime::echo_php_array_is_list
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_array",
            echo_runtime::echo_php_is_array
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_countable",
            echo_runtime::echo_php_is_countable
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_iterable",
            echo_runtime::echo_php_is_iterable
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_numeric",
            echo_runtime::echo_php_is_numeric
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_null",
            echo_runtime::echo_php_is_null
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_bool",
            echo_runtime::echo_php_is_bool
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_callable",
            echo_runtime::echo_php_is_callable
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_int",
            echo_runtime::echo_php_is_int
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_float",
            echo_runtime::echo_php_is_float
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_finite",
            echo_runtime::echo_php_is_finite
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_infinite",
            echo_runtime::echo_php_is_infinite
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_nan",
            echo_runtime::echo_php_is_nan
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_object",
            echo_runtime::echo_php_is_object
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_resource",
            echo_runtime::echo_php_is_resource
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_string",
            echo_runtime::echo_php_is_string
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_scalar",
            echo_runtime::echo_php_is_scalar
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_strval",
            echo_runtime::echo_php_strval
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_boolval",
            echo_runtime::echo_php_boolval
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_intval",
            echo_runtime::echo_php_intval
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_floatval",
            echo_runtime::echo_php_floatval
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
    ]
}
