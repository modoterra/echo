<?php
$left = ["a" => 1, "b" => 2, 3 => "three"];
$right = ["b" => 20, 3 => "other", "c" => 4];
$intersection = array_intersect_key($left, $right);
echo "count:[" . count($intersection) . "]\n";
echo "has-a:[" . array_key_exists("a", $intersection) . "]\n";
echo "b:[" . $intersection["b"] . "]\n";
echo "int:[" . $intersection[3] . "]\n";
echo "exists:[" . function_exists("array_intersect_key") . "]\n";
