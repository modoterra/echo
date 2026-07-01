<?php
$left = ["a" => "1", "b" => 2, "c" => "3"];
$right = ["x" => 2, "y" => "4"];
$diff = array_diff($left, $right);
echo "count:[" . count($diff) . "]\n";
echo "a:[" . $diff["a"] . "]\n";
echo "has-b:[" . array_key_exists("b", $diff) . "]\n";
echo "c:[" . $diff["c"] . "]\n";
echo "exists:[" . function_exists("array_diff") . "]\n";
