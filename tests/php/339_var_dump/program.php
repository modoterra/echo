<?php
$r = var_dump(null);
$r = var_dump(true);
$r = var_dump(false);
$r = var_dump(42);
$r = var_dump(3.5);
$r = var_dump("Echo");
$r = var_dump(["id" => 42, "name" => "Ada"]);
echo "exists:" . function_exists("var_dump") . "\n";
