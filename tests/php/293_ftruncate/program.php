<?php
$path = __DIR__ . "/truncate.txt";
$stream = fopen($path, "w+");

if (!$stream) {
    echo "open-failed\n";
} else {
    fwrite($stream, "abcdef");
    fseek($stream, 4);
    $truncated = ftruncate($stream, 3);
    echo "truncated:" . $truncated . "\n";
    echo "pointer:" . ftell($stream) . "\n";
    echo "short:" . file_get_contents($path) . "\n";
    $extended = ftruncate($stream, 6);
    echo "extended:" . $extended . "\n";
    echo "size:" . filesize($path) . "\n";
    echo "exists:" . function_exists("ftruncate") . "\n";
    fclose($stream);
    $removed = unlink($path);
}
