<?php
$less = strcoll("Alpha", "Beta");
$greater = strcoll("Beta", "Alpha");
$same = strcoll("same", "same");

echo "less:" . ($less < 0) . "\n";
echo "greater:" . ($greater > 0) . "\n";
echo "same:" . ($same === 0) . "\n";
echo "case:" . (strcoll("a", "A") > 0) . "\n";
echo "exists:" . function_exists("strcoll") . "\n";
