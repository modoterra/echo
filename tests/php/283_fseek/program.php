<?php
$stream = fopen(__DIR__ . "/data.txt", "r");
echo "first:" . fread($stream, 3) . "\n";
echo "seek:" . fseek($stream, 2) . "\n";
echo "pos:" . ftell($stream) . "\n";
echo "again:" . fread($stream, 4) . "\n";
echo "exists:" . function_exists("fseek") . "\n";
fclose($stream);
