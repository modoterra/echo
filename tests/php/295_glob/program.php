<?php
$matches = glob(__DIR__ . "/data/*.txt", 0);
$missing = glob(__DIR__ . "/data/*.missing", 0);

echo "count:" . count($matches) . "\n";
echo "first:" . basename($matches[0]) . "\n";
echo "second:" . basename($matches[1]) . "\n";
echo "missing:" . count($missing) . "\n";
echo "exists:" . function_exists("glob") . "\n";
