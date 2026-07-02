<?php
$target = __DIR__ . "/target.txt";
$missing = __DIR__ . "/missing.txt";
$written = file_put_contents($target, "target\n");

echo "present-int:" . is_int(linkinfo($target)) . "\n";
echo "missing:" . linkinfo($missing) . "\n";
echo "exists:" . function_exists("linkinfo") . "\n";

$removed = unlink($target);
