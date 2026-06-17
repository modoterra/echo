<?php
// substr() with omitted length returns from offset to the end.
// Source: https://www.php.net/manual/en/function.substr.php
echo substr("Echo PHP", 5) . "\n";
echo substr("abcdef", 2) . "\n";
echo "empty:" . substr("abcdef", 99) . "\n";
echo substr("abcdef", "3") . "\n";
echo bin2hex(substr("Ächo", 1)) . "\n";
