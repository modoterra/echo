<?php
$stream = fopen(__DIR__ . "/data.txt", "r");

if (!$stream) {
    echo "open-failed\n";
} else {
    $stat = fstat($stream);
    echo "array:" . is_array($stat) . "\n";
    echo "size:" . $stat["size"] . "\n";
    echo "numeric-size:" . $stat[7] . "\n";
    echo "exists:" . function_exists("fstat") . "\n";
    fclose($stream);
}
