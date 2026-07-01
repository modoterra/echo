<?php
$row = ["a" => 1, "b" => 2, 3 => "three"];
echo "shifted:[" . array_shift($row) . "]\n";
echo "count:[" . count($row) . "]\n";
echo "b:[" . $row["b"] . "]\n";
echo "zero:[" . $row[0] . "]\n";
$empty = [];
echo "empty:[" . (array_shift($empty) === null) . "]\n";
echo "exists:[" . function_exists("array_shift") . "]\n";
