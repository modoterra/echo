<?php
// array_search() returns the first key for a matching value.
// Source: https://www.php.net/manual/en/function.array-search.php
// array_count_values() counts repeated int/string values.
// Source: https://www.php.net/manual/en/function.array-count-values.php
$row = ["sku" => "A-42", 7 => "A-42", "qty" => 4, "flag" => true, "code" => "4"];
$counts = array_count_values(["new", "new", "done", 2, "2", 3]);

echo "search-loose:[" . array_search(4, $row) . "]\n";
echo "search-strict:[" . array_search(4, $row, true) . "]\n";
echo "search-first:[" . array_search("A-42", $row, true) . "]\n";
echo "search-miss:[" . array_search("missing", $row, true) . "]\n";
echo "counts-keys:[" . implode(",", array_keys($counts)) . "]\n";
echo "counts-values:[" . implode(",", array_values($counts)) . "]\n";
echo "count-new:[" . $counts["new"] . "]\n";
echo "count-two:[" . $counts[2] . "]\n";
echo "exists:[" . function_exists("array_search") . function_exists("array_count_values") . "]\n";
