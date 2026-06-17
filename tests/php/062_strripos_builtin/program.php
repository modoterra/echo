<?php
// strripos() is a binary-safe, ASCII case-insensitive strrpos().
// Source: https://www.php.net/manual/en/function.strripos.php
echo strripos("abABcd", "aB") . "\n";
echo strripos("abcABC", "BC") . "\n";
echo "missing:" . strripos("abcdef", "XY") . "\n";
echo strripos("abcdef", "") . "\n";
echo strripos("1234545", 45) . "\n";
echo "nonascii:" . strripos("Ächo", "ä") . "\n";
