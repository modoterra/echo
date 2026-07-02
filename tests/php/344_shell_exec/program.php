<?php
$output = shell_exec("printf 'echo-shell'");
echo "output:" . $output . "\n";
$empty = shell_exec("printf ''");
echo "empty:" . gettype($empty) . "\n";
echo "exists:" . function_exists("shell_exec") . "\n";
