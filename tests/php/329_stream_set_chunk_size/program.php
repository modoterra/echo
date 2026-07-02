<?php
$file = __DIR__ . "/chunk.txt";
$written = file_put_contents($file, "chunk");
$stream = fopen($file, "r");

$first = stream_set_chunk_size($stream, 4096);
$second = stream_set_chunk_size($stream, 2048);

echo "first-int:" . is_int($first) . "\n";
echo "second:" . $second . "\n";
echo "exists:" . function_exists("stream_set_chunk_size") . "\n";

$closed = fclose($stream);
$deleted = unlink($file);
