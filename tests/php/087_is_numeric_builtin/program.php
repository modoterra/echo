<?php
// is_numeric() is true for numbers and numeric strings, not for booleans,
// null, arrays, empty strings, or prefixed hex/binary-looking strings.
// Source: https://www.php.net/manual/en/function.is-numeric.php
echo "int:[" . is_numeric(42) . "]\n";
echo "string-int:[" . is_numeric("42") . "]\n";
echo "string-leading-space:[" . is_numeric(" 42") . "]\n";
echo "string-trailing-space:[" . is_numeric("42 ") . "]\n";
echo "string-negative:[" . is_numeric("-42") . "]\n";
echo "string-decimal:[" . is_numeric("4.2") . "]\n";
echo "string-exponent:[" . is_numeric("1337e0") . "]\n";
echo "string-hex:[" . is_numeric("0x539") . "]\n";
echo "string-binary:[" . is_numeric("0b101") . "]\n";
echo "string-empty:[" . is_numeric("") . "]\n";
echo "string-word:[" . is_numeric("not numeric") . "]\n";
echo "bool:[" . is_numeric(true) . "]\n";
echo "null:[" . is_numeric(null) . "]\n";
echo "array:[" . is_numeric([]) . "]\n";
