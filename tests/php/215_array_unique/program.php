<?php
$row = ["a" => "1", "b" => 1, "c" => "2", "d" => "1", 9 => "3"];
$unique = array_unique($row, 2);
echo "count:[" . count($unique) . "]\n";
echo "a:[" . $unique["a"] . "]\n";
echo "has-b:[" . array_key_exists("b", $unique) . "]\n";
echo "c:[" . $unique["c"] . "]\n";
echo "int:[" . $unique[9] . "]\n";
echo "exists:[" . function_exists("array_unique") . "]\n";
