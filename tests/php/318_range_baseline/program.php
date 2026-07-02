<?php
echo implode(",", range(1, 5)) . "\n";
echo implode(",", range(5, 1, 2)) . "\n";
echo implode("", range("a", "e")) . "\n";
echo implode("", range("e", "a", 2)) . "\n";
echo "count:" . count(range(3, 3)) . "\n";
echo "exists:" . function_exists("range") . "\n";
