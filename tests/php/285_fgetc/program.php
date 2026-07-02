<?php
$stream = fopen(__DIR__ . "/data.txt", "r");
echo "one:" . fgetc($stream) . "\n";
echo "two:" . fgetc($stream) . "\n";
fread($stream, 99);
echo "eof:" . fgetc($stream) . "\n";
echo "exists:" . function_exists("fgetc") . "\n";
fclose($stream);
