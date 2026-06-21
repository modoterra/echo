<?php
// array_merge() appends numeric keys and overwrites duplicate string keys.
// Source: https://www.php.net/manual/en/function.array-merge.php
// array_replace() keeps keys while right-most arrays replace earlier values.
// Source: https://www.php.net/manual/en/function.array-replace.php
$base = ["sku" => "A-42", 7 => "old-bin", "status" => "draft"];
$override = ["status" => "active", 4 => "new-bin", "owner" => "maya"];
$extra = ["sku" => "A-43", 9 => "late"];
$merged = array_merge($base, $override, $extra);
$replaced = array_replace($base, $override, $extra);
$empty = array_merge();

echo "merge-keys:[" . implode(",", array_keys($merged)) . "]\n";
echo "merge-values:[" . implode("|", array_values($merged)) . "]\n";
echo "merge-status:[" . $merged["status"] . "]\n";
echo "replace-keys:[" . implode(",", array_keys($replaced)) . "]\n";
echo "replace-values:[" . implode("|", array_values($replaced)) . "]\n";
echo "replace-sku:[" . $replaced["sku"] . "]\n";
echo "replace-index7:[" . $replaced[7] . "]\n";
echo "empty-count:[" . count($empty) . "]\n";
echo "exists:[" . function_exists("array_merge") . function_exists("array_replace") . "]\n";
