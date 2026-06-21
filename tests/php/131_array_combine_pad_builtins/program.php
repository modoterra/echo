<?php
// array_combine() creates an array using one array for keys and another for values.
// Source: https://www.php.net/manual/en/function.array-combine.php
// array_pad() pads an array copy to a requested length.
// Source: https://www.php.net/manual/en/function.array-pad.php
$keys = ["sku", "qty", "qty", "2"];
$values = ["A-42", 3, 4, "numeric"];
$combined = array_combine($keys, $values);

$base = ["sku" => "A-42", 7 => "seven", "qty" => 4];
$right = array_pad($base, 5, "missing");
$left = array_pad($base, -5, "missing");
$none = array_pad($base, 2, "noop");

echo "combine-keys:[" . implode(",", array_keys($combined)) . "]\n";
echo "combine-values:[" . implode(",", array_values($combined)) . "]\n";
echo "combine-qty:[" . $combined["qty"] . "]\n";
echo "combine-two:[" . $combined[2] . "]\n";
echo "right-keys:[" . implode(",", array_keys($right)) . "]\n";
echo "right-values:[" . implode(",", array_values($right)) . "]\n";
echo "left-keys:[" . implode(",", array_keys($left)) . "]\n";
echo "left-values:[" . implode(",", array_values($left)) . "]\n";
echo "none-keys:[" . implode(",", array_keys($none)) . "]\n";
echo "exists:[" . function_exists("array_combine") . function_exists("array_pad") . "]\n";
