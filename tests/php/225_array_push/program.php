<?php
$row = ["a" => 1];

echo "push:[" . array_push($row, "two") . "]\n";
echo "count:[" . count($row) . "]\n";
echo "a:[" . $row["a"] . "]\n";
echo "zero:[" . $row[0] . "]\n";
echo "exists:[" . function_exists("array_push") . "]\n";
