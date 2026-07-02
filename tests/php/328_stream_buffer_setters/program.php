<?php
$file = __DIR__ . "/buffer.txt";
$written = file_put_contents($file, "buffer");
$reader = fopen($file, "r");
$writer = fopen($file, "r+");

$read = stream_set_read_buffer($reader, 0);
$write = stream_set_write_buffer($writer, 0);

echo "read:" . $read . "\n";
echo "write:" . $write . "\n";
echo "ints:" . is_int($read) . is_int($write) . "\n";
echo "exists:" . function_exists("stream_set_read_buffer") . function_exists("stream_set_write_buffer") . "\n";

$closed_reader = fclose($reader);
$closed_writer = fclose($writer);
$deleted = unlink($file);
