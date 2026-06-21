<?php
// implode() joins array values with a separator; join() is its alias.
// Source: https://www.php.net/manual/en/function.implode.php
// Source: https://www.php.net/manual/en/function.join.php
$columns = ["lastname", "email", "phone"];
$record = ["Doe", "d@example.com", "555-0100"];
$mixed = ["a", 2, true, false, null, 3.5];
$assoc = ["first" => "one", "two", "third" => 3];

echo "header:[" . implode(",", $columns) . "]\n";
echo "row:[" . join(",", $record) . "]\n";
echo "default:[" . implode($columns) . "]\n";
echo "mixed:[" . join("|", $mixed) . "]\n";
echo "empty:[" . implode("hello", []) . "]\n";
echo "assoc:[" . implode(",", $assoc) . "]\n";
echo "exists-implode:[" . function_exists("implode") . "]\n";
echo "exists-join:[" . function_exists("join") . "]\n";
