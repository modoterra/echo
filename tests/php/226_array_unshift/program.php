<?php
$row = ["a" => 1];

echo "unshift:[" . array_unshift($row, "zero") . "]\n";
echo "count:[" . count($row) . "]\n";
echo "zero:[" . $row[0] . "]\n";
echo "a:[" . $row["a"] . "]\n";
echo "exists:[" . function_exists("array_unshift") . "]\n";
