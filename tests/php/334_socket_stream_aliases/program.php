<?php
$file = __DIR__ . "/aliases.txt";
$written = file_put_contents($file, "aliases\n");
$stream = fopen($file, "r");

$status = socket_get_status($stream);
$blocking = socket_set_blocking($stream, false);
$timeout = socket_set_timeout($stream, 2, 0);

echo "status:" . is_array($status) . "\n";
echo "wrapper:" . $status["wrapper_type"] . "\n";
echo "blocking:" . $blocking . "\n";
echo "timeout:" . $timeout . "\n";
echo "exists:" . function_exists("socket_get_status") . function_exists("socket_set_blocking") . function_exists("socket_set_timeout") . "\n";

$closed = fclose($stream);
$deleted = unlink($file);
