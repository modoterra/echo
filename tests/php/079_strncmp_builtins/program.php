<?php
// strncmp() compares at most the requested number of bytes.
// Source: https://www.php.net/manual/en/function.strncmp.php
echo strncmp("abc", "abd", 2) . "\n";
echo strncmp("abc", "abd", 3) . "\n";
echo strncmp("abc", "ab", 3) . "\n";
echo strncmp("abc", "xyz", 0) . "\n";

// strncasecmp() applies ASCII case-insensitive comparison to the prefix.
// Source: https://www.php.net/manual/en/function.strncasecmp.php
echo strncasecmp("Echo", "echo", 4) . "\n";
echo strncasecmp("abc", "ABD", 2) . "\n";
echo strncasecmp("abc", "ABD", 3) . "\n";
echo strncasecmp("Ä", "ä", 2) . "\n";
