<?php
$stat = lstat(".");
echo "array:" . is_array($stat) . "\n";
echo "count:" . count($stat) . "\n";
echo "has-size:" . array_key_exists("size", $stat) . "\n";
echo "has-zero:" . array_key_exists(0, $stat) . "\n";
echo "missing:" . lstat("/no/such/path") . "\n";
echo "exists:" . function_exists("lstat") . "\n";
