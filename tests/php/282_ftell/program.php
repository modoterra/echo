<?php
$stream = fopen(__DIR__ . "/data.txt", "r");
echo "start:" . ftell($stream) . "\n";
echo "read:" . fread($stream, 4) . "\n";
echo "after:" . ftell($stream) . "\n";
fclose($stream);
echo "exists:" . function_exists("ftell") . "\n";
