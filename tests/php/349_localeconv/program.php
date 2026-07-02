<?php
$info = localeconv();
echo "decimal:" . $info["decimal_point"] . "\n";
echo "thousands:" . var_export($info["thousands_sep"], true) . "\n";
echo "grouping:" . count($info["grouping"]) . "\n";
echo "frac:" . $info["frac_digits"] . "\n";
echo "exists:" . function_exists("localeconv") . "\n";
