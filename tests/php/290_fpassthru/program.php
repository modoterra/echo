<?php
$stream = fopen(__DIR__ . "/data.txt", "r");
if (!$stream) {
    echo "open-failed\n";
} else {
    echo "prefix:";
    fread($stream, 2);
    echo ":";
    echo ":count=" . fpassthru($stream) . "\n";
    echo "exists:" . function_exists("fpassthru") . "\n";
    fclose($stream);
}
