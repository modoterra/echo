<?php
$left = ["a" => 1, "b" => 2, 3 => "three"];
$right = ["b" => 20, 3 => "other", "c" => 4];
$diff = array_diff_key($left, $right);
echo "count:[" . count($diff) . "]\n";
echo "a:[" . $diff["a"] . "]\n";
echo "has-b:[" . array_key_exists("b", $diff) . "]\n";
echo "has-int:[" . array_key_exists(3, $diff) . "]\n";
echo "exists:[" . function_exists("array_diff_key") . "]\n";
