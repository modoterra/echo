<?php
// These PHP 8 string predicate builtins are binary-safe, case-sensitive, and
// return true for an empty needle.
// Sources:
// https://www.php.net/manual/en/function.str-contains.php
// https://www.php.net/manual/en/function.str-starts-with.php
// https://www.php.net/manual/en/function.str-ends-with.php
echo str_contains("Echo PHP", "PHP") . "\n";
echo str_contains("Echo PHP", "php") . "\n";
echo str_contains("Echo PHP", "") . "\n";
echo str_starts_with("Echo PHP", "Echo") . "\n";
echo str_starts_with("Echo PHP", "PHP") . "\n";
echo str_starts_with("Echo PHP", "") . "\n";
echo str_ends_with("Echo PHP", "PHP") . "\n";
echo str_ends_with("Echo PHP", "Echo") . "\n";
echo str_ends_with("Echo PHP", "") . "\n";
echo str_contains("Ä", chr(195)) . "\n";
