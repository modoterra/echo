<?php
$rows = [
    ["id" => 10, "name" => "Ada"],
    ["id" => 11, "name" => "Linus"],
    ["id" => 12],
];

$names = array_column($rows, "name");
echo "count:[" . count($names) . "]\n";
echo "first:[" . $names[0] . "]\n";
echo "second:[" . $names[1] . "]\n";
echo "exists:[" . function_exists("array_column") . "]\n";
