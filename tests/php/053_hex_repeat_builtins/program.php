<?php
// hex2bin() decodes hexadecimal byte pairs; str_repeat() repeats a string
// zero or more times.
// Sources:
// https://www.php.net/manual/en/function.hex2bin.php
// https://www.php.net/manual/en/function.str-repeat.php
echo hex2bin("4563686f") . "\n";
echo bin2hex(hex2bin("c384")) . "\n";
echo str_repeat("xo", 3) . "\n";
echo "empty:" . str_repeat("x", 0) . "\n";
