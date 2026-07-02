<?php
$result = system("printf 'first\nlast\n'");
echo "\n";
echo "returned:" . var_export($result, true) . "\n";
$empty_result = system("printf ''");
echo "empty:" . var_export($empty_result, true) . "\n";
echo "exists:" . function_exists("system") . "\n";
