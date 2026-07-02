<?php
$path = __DIR__ . "/sync.txt";
$stream = fopen($path, "w+");

if (!$stream) {
    echo "open-failed\n";
} else {
    fwrite($stream, "abc");
    $data_synced = fdatasync($stream);
    fwrite($stream, "def");
    $all_synced = fsync($stream);
    fclose($stream);

    echo "data:" . $data_synced . "\n";
    echo "all:" . $all_synced . "\n";
    echo "contents:" . file_get_contents($path) . "\n";
    echo "exists:" . function_exists("fsync") . function_exists("fdatasync") . "\n";
    $removed = unlink($path);
}
