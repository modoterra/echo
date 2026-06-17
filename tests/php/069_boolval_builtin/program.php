<?php
// boolval() returns PHP's boolean conversion of a value.
// Source: https://www.php.net/manual/en/function.boolval.php
echo "empty:" . boolval("") . "\n";
echo "zero-string:" . boolval("0") . "\n";
echo "false-string:" . boolval("false") . "\n";
echo "text:" . boolval("hello") . "\n";
echo "zero-number:" . boolval(0) . "\n";
echo "one-number:" . boolval(1) . "\n";
