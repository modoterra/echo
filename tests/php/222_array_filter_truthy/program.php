<?php
$row = ["a" => 1, "b" => 0, "c" => "", "d" => "ok", 5 => false, 6 => null, 7 => "0"];
$filtered = array_filter($row, null, 0);
echo "count:[" . count($filtered) . "]\n";
echo "a:[" . $filtered["a"] . "]\n";
echo "d:[" . $filtered["d"] . "]\n";
echo "has-b:[" . array_key_exists("b", $filtered) . "]\n";
echo "has-zero-string:[" . array_key_exists(7, $filtered) . "]\n";
echo "exists:[" . function_exists("array_filter") . "]\n";
