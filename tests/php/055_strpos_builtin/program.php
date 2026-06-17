<?php
// strpos() returns a zero-based byte position or false when not found.
// Source: https://www.php.net/manual/en/function.strpos.php
echo strpos("abcdef", "ab") . "\n";
echo strpos("abcdef", "cd") . "\n";
echo "missing:" . strpos("abcdef", "xy") . "\n";
echo strpos("12345", 34) . "\n";
echo strpos("Ächo", "c") . "\n";
