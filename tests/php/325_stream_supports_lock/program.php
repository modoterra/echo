<?php
$file = __DIR__ . "/lock.txt";
$written = file_put_contents($file, "lock");
$stream = fopen($file, "r");

$result = stream_supports_lock($stream);
echo "supports:" . $result . "\n";
echo "bool:" . is_bool($result) . "\n";
echo "exists:" . function_exists("stream_supports_lock") . "\n";

$closed = fclose($stream);
$deleted = unlink($file);
