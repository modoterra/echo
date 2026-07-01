<?php
$row = ["a" => 1, "b" => 2, 3 => "three"];
echo "popped:[" . array_pop($row) . "]\n";
echo "count:[" . count($row) . "]\n";
echo "a:[" . $row["a"] . "]\n";
echo "has-int:[" . array_key_exists(3, $row) . "]\n";
$empty = [];
echo "empty:[" . (array_pop($empty) === null) . "]\n";
echo "exists:[" . function_exists("array_pop") . "]\n";
