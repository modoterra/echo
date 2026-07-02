<?php
$file = __DIR__ . "/tty.txt";
$written = file_put_contents($file, "tty");
$stream = fopen($file, "r");

$result = stream_isatty($stream);
echo "isatty:" . $result . "\n";
echo "bool:" . is_bool($result) . "\n";
echo "exists:" . function_exists("stream_isatty") . "\n";

$closed = fclose($stream);
$deleted = unlink($file);
