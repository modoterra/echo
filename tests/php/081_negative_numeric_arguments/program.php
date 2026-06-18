<?php
// PHP accepts unary-negative numeric arguments in function calls.
// Source: https://www.php.net/manual/en/function.substr-compare.php
echo substr_compare("abcde", "de", -2, 2) . "\n";
echo substr_compare("abcde", "DE", -2, null, true) . "\n";
