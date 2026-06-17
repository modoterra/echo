<?php
// strspn() counts the initial span made only of bytes from the mask.
// Source: https://www.php.net/manual/en/function.strspn.php
echo strspn("42 is the answer", "0123456789") . "\n";
echo strspn("abcdef", "abc") . "\n";
echo strspn("abcdef", "xyz") . "\n";
echo strspn("12345", 12) . "\n";
echo "nonascii:" . strspn("Ächo", "Äc") . "\n";
echo "empty:" . strspn("abc", "") . "\n";
