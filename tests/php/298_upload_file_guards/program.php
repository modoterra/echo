<?php
$source = __DIR__ . "/candidate.txt";
$target = __DIR__ . "/moved.txt";
$written = file_put_contents($source, "payload\n");

echo "uploaded:" . is_uploaded_file($source) . "\n";
echo "moved:" . move_uploaded_file($source, $target) . "\n";
echo "source:" . file_exists($source) . "\n";
echo "target:" . file_exists($target) . "\n";
echo "exists:" . function_exists("is_uploaded_file") . function_exists("move_uploaded_file") . "\n";

$removed = unlink($source);
