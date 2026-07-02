<?php
$file = __DIR__ . "/blocking.txt";
$written = file_put_contents($file, "blocking");
$stream = fopen($file, "r");

$off = stream_set_blocking($stream, false);
$on = stream_set_blocking($stream, true);

echo "off:" . $off . "\n";
echo "on:" . $on . "\n";
echo "bools:" . is_bool($off) . is_bool($on) . "\n";
echo "exists:" . function_exists("stream_set_blocking") . "\n";

$closed = fclose($stream);
$deleted = unlink($file);
