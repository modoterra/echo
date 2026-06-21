<?php
// Array key/value helpers preserve insertion order, key reindexing, optional
// value filtering, and aggregate initial values.
// Sources:
// - https://www.php.net/manual/en/function.array-values.php
// - https://www.php.net/manual/en/function.array-keys.php
// - https://www.php.net/manual/en/function.array-sum.php
// - https://www.php.net/manual/en/function.array-product.php
$row = ["id" => 10, "qty" => "2", 5 => 2, "zero" => 0];
$numbers = [2, "3", 4];

echo "values:[" . implode("|", array_values($row)) . "]\n";
echo "keys:[" . implode("|", array_keys($row)) . "]\n";
echo "keys-filter:[" . implode("|", array_keys($row, 2)) . "]\n";
echo "keys-strict:[" . implode("|", array_keys($row, 2, true)) . "]\n";
echo "sum:[" . array_sum($row) . "]\n";
echo "sum-empty:[" . array_sum([]) . "]\n";
echo "product:[" . array_product($numbers) . "]\n";
echo "product-empty:[" . array_product([]) . "]\n";
echo "exists:[" . function_exists("array_values") . function_exists("array_keys") . function_exists("array_sum") . function_exists("array_product") . "]\n";
