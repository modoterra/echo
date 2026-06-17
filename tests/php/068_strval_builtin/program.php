<?php
// strval() returns PHP's string representation of a scalar value.
// Source: https://www.php.net/manual/en/function.strval.php
echo strval("hello") . "\n";
echo "[" . strval(42) . "]\n";
echo "[" . strval(0) . "]\n";
echo "[" . strval(7) . "]\n";
echo "nonascii:" . strval("Ächo") . "\n";
