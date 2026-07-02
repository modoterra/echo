<?php
$result = exec("printf 'first\nlast\n'");
echo "returned:" . var_export($result, true) . "\n";
$empty_result = exec("printf ''");
echo "empty:" . var_export($empty_result, true) . "\n";
echo "exists:" . function_exists("exec") . "\n";
