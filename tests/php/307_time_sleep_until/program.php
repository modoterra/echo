<?php
$past = time_sleep_until(1);
$future = time_sleep_until(microtime(true) + 0.001);

echo "past:" . ($past === false) . "\n";
echo "future:" . ($future === true) . "\n";
echo "exists:" . function_exists("time_sleep_until") . "\n";
