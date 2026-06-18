<?php
// is_infinite() coerces numeric scalar values to float and detects +/-INF.
// Source: https://www.php.net/manual/en/function.is-infinite.php
echo "int:[" . is_infinite(42) . "]\n";
echo "negative:[" . is_infinite(-7) . "]\n";
echo "bool:[" . is_infinite(true) . "]\n";
echo "null:[" . is_infinite(null) . "]\n";
echo "numeric-string:[" . is_infinite(" 4.2 ") . "]\n";
echo "overflow-string:[" . is_infinite("1e9999") . "]\n";
echo "negative-overflow-string:[" . is_infinite("-1e9999") . "]\n";
