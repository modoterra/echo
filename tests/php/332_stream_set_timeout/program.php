<?php
$file = __DIR__ . "/timeout.txt";
$written = file_put_contents($file, "timeout\n");
$stream = fopen($file, "r");

$result = stream_set_timeout($stream, 2, 500000);
$meta = stream_get_meta_data($stream);

echo "result:" . $result . "\n";
echo "timed-out:" . $meta["timed_out"] . "\n";
echo "exists:" . function_exists("stream_set_timeout") . "\n";

$closed = fclose($stream);
$deleted = unlink($file);
