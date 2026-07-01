<?php
$left = ["a" => "1", "b" => 2, "c" => "3", 4 => "same"];
$right = ["a" => "x", "b" => 2, 4 => "same"];
$diff = array_diff_assoc($left, $right);
echo "count:[" . count($diff) . "]\n";
echo "a:[" . $diff["a"] . "]\n";
echo "has-b:[" . array_key_exists("b", $diff) . "]\n";
echo "c:[" . $diff["c"] . "]\n";
echo "has-int:[" . array_key_exists(4, $diff) . "]\n";
echo "exists:[" . function_exists("array_diff_assoc") . "]\n";
