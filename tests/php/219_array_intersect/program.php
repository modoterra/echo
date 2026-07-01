<?php
$left = ["a" => "1", "b" => 2, "c" => "3"];
$right = ["x" => 2, "y" => "4"];
$intersection = array_intersect($left, $right);
echo "count:[" . count($intersection) . "]\n";
echo "has-a:[" . array_key_exists("a", $intersection) . "]\n";
echo "b:[" . $intersection["b"] . "]\n";
echo "has-c:[" . array_key_exists("c", $intersection) . "]\n";
echo "exists:[" . function_exists("array_intersect") . "]\n";
