<?php
echo serialize(null) . "\n";
echo serialize(true) . "\n";
echo serialize(false) . "\n";
echo serialize(42) . "\n";
echo serialize("Echo") . "\n";
echo serialize(["id" => 42, "name" => "Ada"]) . "\n";
echo "exists:" . function_exists("serialize") . "\n";
