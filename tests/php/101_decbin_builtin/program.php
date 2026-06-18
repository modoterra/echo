<?php
// decbin() returns the unsigned binary string representation of an integer.
// Source: https://www.php.net/manual/en/function.decbin.php
echo "twelve:[" . decbin(12) . "]\n";
echo "twenty-six:[" . decbin(26) . "]\n";
echo "zero:[" . decbin(0) . "]\n";
echo "negative:[" . decbin(-1) . "]\n";
echo "numeric-string:[" . decbin("255") . "]\n";
echo "exists:[" . function_exists("decbin") . "]\n";
