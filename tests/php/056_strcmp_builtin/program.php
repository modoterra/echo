<?php
// strcmp() is binary-safe and only the sign of non-zero results is reliable.
// Source: https://www.php.net/manual/en/function.strcmp.php
echo strcmp("a", "b") . "\n";
echo strcmp("b", "a") . "\n";
echo strcmp("same", "same") . "\n";
echo strcmp("abc", "ab") . "\n";
echo strcmp("123", 123) . "\n";
