<?php
// array_fill() creates repeated values with integer keys starting at the requested index.
// Source: https://www.php.net/manual/en/function.array-fill.php
// array_fill_keys() creates an array whose keys come from the input values.
// Source: https://www.php.net/manual/en/function.array-fill-keys.php
$filled = array_fill(-2, 4, "pear");
$keys = ["sku", "2", 5, true, null, "sku"];
$keyed = array_fill_keys($keys, "todo");

echo "fill-first:[" . array_key_first($filled) . "]\n";
echo "fill-last:[" . array_key_last($filled) . "]\n";
echo "fill-count:[" . count($filled) . "]\n";
echo "fill-values:[" . implode(",", array_values($filled)) . "]\n";
echo "keyed-keys:[" . implode(",", array_keys($keyed)) . "]\n";
echo "keyed-first:[" . array_key_first($keyed) . "]\n";
echo "keyed-last:[" . array_key_last($keyed) . "]\n";
echo "keyed-empty-exists:[" . array_key_exists("", $keyed) . "]\n";
echo "keyed-two:[" . $keyed[2] . "]\n";
echo "exists:[" . function_exists("array_fill") . function_exists("array_fill_keys") . "]\n";
