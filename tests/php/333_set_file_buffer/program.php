<?php
$file = __DIR__ . "/buffer.txt";
$written = file_put_contents($file, "buffer\n");
$stream = fopen($file, "r+");

$result = set_file_buffer($stream, 0);

echo "result:" . $result . "\n";
echo "exists:" . function_exists("set_file_buffer") . "\n";

$closed = fclose($stream);
$deleted = unlink($file);
