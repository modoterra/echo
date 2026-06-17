<?php
// ucwords() uppercases the first ASCII character after default separators.
// Source: https://www.php.net/manual/en/function.ucwords.php
echo ucwords("hello world") . "\n";
echo ucwords("hello\tworld") . "\n";
echo ucwords("hello-world") . "\n";
echo ucwords("123 abc") . "\n";
echo ucwords("ächo world") . "\n";
echo ucwords("mIXed CASE") . "\n";
