<?php
$stream = fopen(__DIR__ . "/data.txt", "r");
if (!$stream) {
    echo "open-failed\n";
} else {
    echo "first:" . fgets($stream, 4) . "\n";
    echo "pos:" . ftell($stream) . "\n";
    echo "second:" . fgets($stream, 10) . "\n";
    echo "exists:" . function_exists("fgets") . "\n";
    fclose($stream);
}

$stream = fopen(__DIR__ . "/data.txt", "r");
if ($stream) {
    echo "all:" . fgets($stream) . "\n";
    fclose($stream);
}
