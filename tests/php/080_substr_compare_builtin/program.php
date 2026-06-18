<?php
// substr_compare() compares haystack from offset with needle, with optional length and case-insensitivity.
// Source: https://www.php.net/manual/en/function.substr-compare.php
echo substr_compare("abcde", "bc", 1, 2) . "\n";
echo substr_compare("abcde", "de", -2, 2) . "\n";
echo substr_compare("abcde", "bcg", 1, 2) . "\n";
echo substr_compare("abcde", "BC", 1, 2, true) . "\n";
echo substr_compare("abcde", "bc", 1, 3) . "\n";
echo substr_compare("abcde", "cd", 1, 2) . "\n";
echo substr_compare("abcde", "de", -2) . "\n";
echo substr_compare("abcde", "DE", -2, null, true) . "\n";
