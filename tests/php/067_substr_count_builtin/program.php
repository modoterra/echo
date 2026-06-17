<?php
// substr_count() counts non-overlapping occurrences of the needle.
// Source: https://www.php.net/manual/en/function.substr-count.php
echo substr_count("This is a test", "is") . "\n";
echo substr_count("aaaa", "aa") . "\n";
echo substr_count("abcdef", "xy") . "\n";
echo substr_count("1234512345", 45) . "\n";
echo "nonascii:" . substr_count("ÄchoÄ", "Ä") . "\n";
