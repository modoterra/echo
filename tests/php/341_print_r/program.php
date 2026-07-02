<?php
$payload = ["id" => 42, "name" => "Ada", "active" => true, "none" => null];
$printed = print_r($payload, true);
echo $printed . "\n";
$r = print_r("Echo", false);
echo "\n";
echo "returned:" . gettype($r) . ":" . $r . "\n";
echo "exists:" . function_exists("print_r") . "\n";
