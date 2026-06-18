<?php
// is_countable()/is_iterable() recognize arrays; object interfaces are deferred in Echo.
// is_scalar() checks scalar value types without coercion.
// Sources:
// - https://www.php.net/manual/en/function.is-countable.php
// - https://www.php.net/manual/en/function.is-iterable.php
// - https://www.php.net/manual/en/function.is-scalar.php
echo "countable-array:[" . is_countable([]) . "]\n";
echo "countable-int:[" . is_countable(42) . "]\n";
echo "iterable-array:[" . is_iterable([1, 2]) . "]\n";
echo "iterable-string:[" . is_iterable("abc") . "]\n";
echo "scalar-bool:[" . is_scalar(false) . "]\n";
echo "scalar-int:[" . is_scalar(42) . "]\n";
echo "scalar-string:[" . is_scalar("abc") . "]\n";
echo "scalar-null:[" . is_scalar(null) . "]\n";
echo "scalar-array:[" . is_scalar([]) . "]\n";
