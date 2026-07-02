<?php
$path = __DIR__ . "/key.txt";
$first = ftok($path, "E");
$second = ftok($path, "E");
echo "type:" . gettype($first) . "\n";
echo "stable:" . ($first === $second) . "\n";
echo "missing:" . ftok(__DIR__ . "/missing.txt", "E") . "\n";
echo "exists:" . function_exists("ftok") . "\n";
