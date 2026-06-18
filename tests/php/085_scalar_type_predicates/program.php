<?php
// Scalar type predicate builtins check the value type without coercion.
// Sources:
// - https://www.php.net/manual/en/function.is-null.php
// - https://www.php.net/manual/en/function.is-bool.php
// - https://www.php.net/manual/en/function.is-int.php
// - https://www.php.net/manual/en/function.is-float.php
// - https://www.php.net/manual/en/function.is-string.php
echo "null:[" . is_null(null) . "]\n";
echo "false:[" . is_bool(false) . "]\n";
echo "true:[" . is_bool(true) . "]\n";
echo "int:[" . is_int(42) . "]\n";
echo "integer:[" . is_integer(42) . "]\n";
echo "long:[" . is_long(42) . "]\n";
echo "float-int:[" . is_float(42) . "]\n";
echo "double-string:[" . is_double("4.2") . "]\n";
echo "string:[" . is_string("42") . "]\n";
echo "array-bool:[" . is_bool([]) . "]\n";
echo "string-int:[" . is_int("42") . "]\n";
echo "null-string:[" . is_string(null) . "]\n";
