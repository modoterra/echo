<?php
// count() returns the number of elements in a PHP array.
// Source: https://www.php.net/manual/en/function.count.php
echo count([]) . "\n";
echo count(["alpha", "beta", 3]) . "\n";
echo "nested:" . count([[1], [2, 3], []]) . "\n";
