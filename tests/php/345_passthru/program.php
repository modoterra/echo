<?php
$result = passthru("printf 'raw-pass'");
echo "\n";
echo "returned:" . gettype($result) . "\n";
echo "exists:" . function_exists("passthru") . "\n";
