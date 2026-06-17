<?php
// stristr() is a binary-safe, ASCII case-insensitive strstr().
// Source: https://www.php.net/manual/en/function.stristr.php
echo stristr("USER@EXAMPLE.com", "e") . "\n";
echo "missing:" . stristr("abcdef", "XY") . "\n";
echo stristr("abcdef", "AB") . "\n";
echo stristr("12345", 34) . "\n";
echo stristr("abcdef", "") . "\n";
echo "nonascii:" . stristr("Ächo", "ä") . "\n";
