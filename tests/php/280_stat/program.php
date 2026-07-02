<?php
$stat = stat(".");
echo "array:" . is_array($stat) . "\n";
echo "count:" . count($stat) . "\n";
echo "has-size:" . array_key_exists("size", $stat) . "\n";
echo "has-zero:" . array_key_exists(0, $stat) . "\n";
echo "missing:" . stat("/no/such/path") . "\n";
echo "exists:" . function_exists("stat") . "\n";
