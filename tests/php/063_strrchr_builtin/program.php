<?php
// strrchr() returns the tail of the haystack from the last occurrence.
// Source: https://www.php.net/manual/en/function.strrchr.php
echo strrchr("name@example.com", "@") . "\n";
echo strrchr("abcabc", "bc") . "\n";
echo "missing:" . strrchr("abcdef", "xy") . "\n";
echo strrchr("1234545", 45) . "\n";
echo "nonascii:" . strrchr("ÄchoÄ", "Ä") . "\n";
