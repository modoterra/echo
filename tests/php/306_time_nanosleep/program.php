<?php
$zero = time_nanosleep(0, 0);
$tiny = time_nanosleep(0, 1);

echo "zero:" . ($zero === true) . "\n";
echo "tiny:" . ($tiny === true) . "\n";
echo "exists:" . function_exists("time_nanosleep") . "\n";
