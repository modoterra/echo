<?php
$stream = fopen(__DIR__ . "/data.txt", "r");
echo "first:" . fread($stream, 4) . "\n";
echo "rewind:" . rewind($stream) . "\n";
echo "pos:" . ftell($stream) . "\n";
echo "again:" . fread($stream, 4) . "\n";
echo "exists:" . function_exists("rewind") . "\n";
fclose($stream);
