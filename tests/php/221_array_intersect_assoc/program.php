<?php
$left = ["a" => "1", "b" => 2, "c" => "3", 4 => "same"];
$right = ["a" => "x", "b" => 2, 4 => "same"];
$intersection = array_intersect_assoc($left, $right);
echo "count:[" . count($intersection) . "]\n";
echo "has-a:[" . array_key_exists("a", $intersection) . "]\n";
echo "b:[" . $intersection["b"] . "]\n";
echo "has-c:[" . array_key_exists("c", $intersection) . "]\n";
echo "int:[" . $intersection[4] . "]\n";
echo "exists:[" . function_exists("array_intersect_assoc") . "]\n";
