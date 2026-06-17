<?php
// strstr() returns the part of the string from the first needle through the end.
// Source: https://www.php.net/manual/en/function.strstr.php
echo strstr("name@example.com", "@") . "\n";
echo "missing:" . strstr("abcdef", "xy") . "\n";
echo strstr("abcdef", "ab") . "\n";
echo strstr("12345", 34) . "\n";
echo strstr("abcdef", "") . "\n";
echo bin2hex(strstr("Ächo", "c")) . "\n";
