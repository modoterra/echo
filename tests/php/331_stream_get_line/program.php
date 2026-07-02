<?php
$file = __DIR__ . "/lines.txt";
$written = file_put_contents($file, "alpha|beta|gamma");
$stream = fopen($file, "r");

echo "first:" . stream_get_line($stream, 20, "|") . "\n";
echo "second:" . stream_get_line($stream, 4, "|") . "\n";
echo "third:" . stream_get_line($stream, 20, "|") . "\n";

$rewound = rewind($stream);
echo "bounded:" . stream_get_line($stream, 3, "") . "\n";
echo "zero:" . stream_get_line($stream, 0, "|") . "\n";
echo "eof:" . (stream_get_line($stream, 20, "|") === false) . "\n";
echo "exists:" . function_exists("stream_get_line") . "\n";

$closed = fclose($stream);
$deleted = unlink($file);
