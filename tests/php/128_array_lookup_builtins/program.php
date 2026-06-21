<?php
// array_key_exists() checks keys, including keys with null values.
// Source: https://www.php.net/manual/en/function.array-key-exists.php
// key_exists() is an alias of array_key_exists().
// Source: https://www.php.net/manual/en/function.key-exists.php
// array_key_first() and array_key_last() return the first/last key or null for an empty array.
// Sources:
// https://www.php.net/manual/en/function.array-key-first.php
// https://www.php.net/manual/en/function.array-key-last.php
// in_array() searches values loosely by default and strictly when requested.
// Source: https://www.php.net/manual/en/function.in-array.php
$row = ["id" => 10, "qty" => "2", 5 => null, "0" => "zero"];

echo "exists-id:[" . array_key_exists("id", $row) . "]\n";
echo "exists-null:[" . array_key_exists(5, $row) . "]\n";
echo "exists-alias:[" . key_exists("qty", $row) . "]\n";
echo "exists-missing:[" . array_key_exists("missing", $row) . "]\n";
echo "exists-bool:[" . array_key_exists(false, $row) . "]\n";
echo "first:[" . array_key_first($row) . "]\n";
echo "last:[" . array_key_last($row) . "]\n";
echo "first-empty:[" . is_null(array_key_first([])) . "]\n";
echo "last-empty:[" . is_null(array_key_last([])) . "]\n";
echo "in-loose:[" . in_array(2, $row) . "]\n";
echo "in-strict:[" . in_array(2, $row, true) . "]\n";
echo "in-string-strict:[" . in_array("2", $row, true) . "]\n";
echo "exists:[" . function_exists("array_key_exists") . function_exists("key_exists") . function_exists("array_key_first") . function_exists("array_key_last") . function_exists("in_array") . "]\n";
