<?php
// strcspn() counts the initial span made only of bytes outside the mask.
// Source: https://www.php.net/manual/en/function.strcspn.php
echo strcspn("abcd", "x") . "\n";
echo strcspn("abcd", "d") . "\n";
echo strcspn("abcd", "bd") . "\n";
echo strcspn("12345", 34) . "\n";
echo "nonascii:" . strcspn("Ächo", "c") . "\n";
echo "empty:" . strcspn("abc", "") . "\n";
