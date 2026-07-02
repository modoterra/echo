<?php
$entries = scandir(__DIR__ . "/scan_me");
echo implode(",", $entries) . "\n";
echo "count:" . count($entries) . "\n";
echo "missing:" . scandir(__DIR__ . "/missing") . "\n";
echo "exists:" . function_exists("scandir") . "\n";
