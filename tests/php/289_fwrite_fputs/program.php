<?php
$path = __DIR__ . "/written.txt";
$stream = fopen($path, "w+");
if (!$stream) {
    echo "open-failed\n";
} else {
    echo "write:" . fwrite($stream, "abcdef", 3) . "\n";
    echo "puts:" . fputs($stream, "XYZ") . "\n";
    rewind($stream);
    echo "content:" . fread($stream, 32) . "\n";
    echo "fwrite-exists:" . function_exists("fwrite") . "\n";
    echo "fputs-exists:" . function_exists("fputs") . "\n";
    fclose($stream);
}
$removed = unlink($path);
