<?php
$simple = str_getcsv("a,b,c");
echo "simple:" . count($simple) . ":" . implode("|", $simple) . "\n";
$quoted = str_getcsv("a,\"b,c\",d");
echo "quoted:" . count($quoted) . ":" . implode("|", $quoted) . "\n";
$empty = str_getcsv("a,,c");
echo "empty:" . count($empty) . ":" . implode("|", $empty) . "\n";
echo "exists:" . function_exists("str_getcsv") . "\n";
