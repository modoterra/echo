<?php
// gettype() returns PHP's historical type-name strings.
// Source: https://www.php.net/manual/en/function.gettype.php
echo "null:[" . gettype(null) . "]\n";
echo "false:[" . gettype(false) . "]\n";
echo "true:[" . gettype(true) . "]\n";
echo "int:[" . gettype(42) . "]\n";
echo "string:[" . gettype("abc") . "]\n";
echo "array-empty:[" . gettype([]) . "]\n";
echo "array-list:[" . gettype([1, 2]) . "]\n";
