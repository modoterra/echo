<?php
// intval() returns PHP's integer conversion of a value using base 10 by default.
// Source: https://www.php.net/manual/en/function.intval.php
echo "[" . intval("") . "]\n";
echo "[" . intval("0") . "]\n";
echo "[" . intval("42") . "]\n";
echo "[" . intval("42abc") . "]\n";
echo "[" . intval("  15") . "]\n";
echo "[" . intval("abc") . "]\n";
echo "[" . intval(42) . "]\n";
