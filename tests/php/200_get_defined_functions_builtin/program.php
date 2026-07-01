<?php
$functions = get_defined_functions();

echo "shape:[" . array_key_exists("internal", $functions) . array_key_exists("user", $functions) . "]\n";
echo "arrays:[" . is_array($functions["internal"]) . is_array($functions["user"]) . "]\n";
echo "internal:[" . in_array("strlen", $functions["internal"], true) . in_array("function_exists", $functions["internal"], true) . in_array("get_defined_functions", $functions["internal"], true) . "]\n";
echo "user-count:[" . count($functions["user"]) . "]\n";
echo "exists:[" . function_exists("get_defined_functions") . "]\n";
