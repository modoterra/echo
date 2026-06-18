<?php
// is_finite() coerces numeric scalar values to float and returns false for INF.
// Source: https://www.php.net/manual/en/function.is-finite.php
echo "int:[" . is_finite(42) . "]\n";
echo "negative:[" . is_finite(-7) . "]\n";
echo "bool:[" . is_finite(true) . "]\n";
echo "null:[" . is_finite(null) . "]\n";
echo "numeric-string:[" . is_finite(" 4.2 ") . "]\n";
echo "overflow-string:[" . is_finite("1e9999") . "]\n";
