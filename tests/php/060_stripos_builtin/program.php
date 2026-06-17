<?php
// stripos() is a binary-safe, ASCII case-insensitive strpos().
// Source: https://www.php.net/manual/en/function.stripos.php
echo stripos("ABC", "a") . "\n";
echo stripos("xxEcho", "ECHO") . "\n";
echo "missing:" . stripos("abcdef", "XY") . "\n";
echo stripos("abcdef", "") . "\n";
echo stripos("12345", 34) . "\n";
echo "nonascii:" . stripos("Ächo", "ä") . "\n";
