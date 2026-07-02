<?php
$file = __DIR__ . "/local.txt";
$written = file_put_contents($file, "local");
$stream = fopen($file, "r");

echo "path:" . stream_is_local($file) . "\n";
echo "file-url:" . stream_is_local("file://" . $file) . "\n";
echo "http-url:" . stream_is_local("http://example.com") . "\n";
echo "stream:" . stream_is_local($stream) . "\n";
echo "exists:" . function_exists("stream_is_local") . "\n";

$closed = fclose($stream);
$deleted = unlink($file);
