<?php
// is_nan() coerces numeric scalar values to float and detects NAN.
// Echo cannot construct NAN yet, so this fixture covers current scalar inputs.
// Source: https://www.php.net/manual/en/function.is-nan.php
echo "int:[" . is_nan(42) . "]\n";
echo "negative:[" . is_nan(-7) . "]\n";
echo "bool:[" . is_nan(true) . "]\n";
echo "null:[" . is_nan(null) . "]\n";
echo "numeric-string:[" . is_nan(" 4.2 ") . "]\n";
echo "overflow-string:[" . is_nan("1e9999") . "]\n";
