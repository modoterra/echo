<?php
$stream = fopen(__DIR__ . "/data.txt", "r");
if (!$stream) {
    echo "open-failed\n";
} else {
    echo "flush:" . fflush($stream) . "\n";
    echo "read:" . fread($stream, 3) . "\n";
    echo "exists:" . function_exists("fflush") . "\n";
    fclose($stream);
}
