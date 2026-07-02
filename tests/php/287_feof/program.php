<?php
$stream = fopen(__DIR__ . "/data.txt", "r");
if (!$stream) {
    echo "open-failed\n";
} else {
    if (feof($stream)) {
        echo "start:1\n";
    } else {
        echo "start:0\n";
    }
    echo "first:" . fread($stream, 3) . "\n";
    if (feof($stream)) {
        echo "after-read:1\n";
    } else {
        echo "after-read:0\n";
    }
    echo "past:" . fread($stream, 1) . "\n";
    if (feof($stream)) {
        echo "after-past:1\n";
    } else {
        echo "after-past:0\n";
    }
    rewind($stream);
    if (feof($stream)) {
        echo "after-rewind:1\n";
    } else {
        echo "after-rewind:0\n";
    }
    echo "exists:" . function_exists("feof") . "\n";
    fclose($stream);
}
