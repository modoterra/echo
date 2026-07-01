<?php
$row = ["Name" => "Ada", "COUNT" => 2, 3 => "num"];
$lower = array_change_key_case($row, 0);
$upper = array_change_key_case($row, 1);
echo "lower-name:[" . $lower["name"] . "]\n";
echo "lower-count:[" . $lower["count"] . "]\n";
echo "lower-int:[" . $lower[3] . "]\n";
echo "upper-name:[" . $upper["NAME"] . "]\n";
echo "upper-count:[" . $upper["COUNT"] . "]\n";
echo "exists:[" . function_exists("array_change_key_case") . "]\n";
