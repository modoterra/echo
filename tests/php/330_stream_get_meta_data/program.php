<?php
$file = __DIR__ . "/meta.txt";
$written = file_put_contents($file, "meta\n");
$stream = fopen($file, "r");
$initial = stream_get_meta_data($stream);
$line = fgets($stream);
$after = stream_get_meta_data($stream);

echo "array:" . is_array($initial) . "\n";
echo "wrapper:" . $initial["wrapper_type"] . "\n";
echo "stream:" . $initial["stream_type"] . "\n";
echo "mode:" . $initial["mode"] . "\n";
echo "uri-match:" . ($initial["uri"] === $file) . "\n";
echo "seekable:" . $initial["seekable"] . "\n";
echo "blocked:" . $initial["blocked"] . "\n";
echo "timed-out:" . $initial["timed_out"] . "\n";
echo "unread-int:" . is_int($initial["unread_bytes"]) . "\n";
echo "eof-bool:" . is_bool($after["eof"]) . "\n";
echo "exists:" . function_exists("stream_get_meta_data") . "\n";

$closed = fclose($stream);
$deleted = unlink($file);
