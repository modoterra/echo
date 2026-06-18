<?php
// is_array() returns true only for arrays; sizeof() is an alias of count().
// Sources:
// - https://www.php.net/manual/en/function.is-array.php
// - https://www.php.net/manual/en/function.sizeof.php
echo is_array([]) . "\n";
echo is_array(["alpha", 3]) . "\n";
echo "string:[" . is_array("not array") . "]\n";
echo "int:[" . is_array(42) . "]\n";
echo sizeof([1, 2, 3]) . "\n";
echo sizeof([]) . "\n";
