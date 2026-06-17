<?php
// strcasecmp() is binary-safe and folds only ASCII letters.
// Source: https://www.php.net/manual/en/function.strcasecmp.php
echo strcasecmp("Echo", "echo") . "\n";
echo strcasecmp("a", "B") . "\n";
echo strcasecmp("B", "a") . "\n";
echo strcasecmp("abc", "AB") . "\n";
echo strcasecmp("123", 123) . "\n";
echo strcasecmp("Ä", "ä") . "\n";
