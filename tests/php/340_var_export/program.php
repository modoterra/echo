<?php
$payload = ["id" => 42, "name" => "Ada", "active" => true, "none" => null];
$exported = var_export($payload, true);
echo $exported . "\n";
$r = var_export("Echo's path", false);
echo "\n";
echo "returned:" . gettype($r) . "\n";
echo "exists:" . function_exists("var_export") . "\n";
