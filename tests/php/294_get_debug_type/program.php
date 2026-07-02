<?php
$path = __DIR__ . "/debug_type.txt";
$stream = fopen($path, "w+");

echo "null:" . get_debug_type(null) . "\n";
echo "bool:" . get_debug_type(false) . "\n";
echo "int:" . get_debug_type(42) . "\n";
echo "float:" . get_debug_type(1.5) . "\n";
echo "string:" . get_debug_type("abc") . "\n";
echo "array:" . get_debug_type([1, 2]) . "\n";
echo "stream:" . get_debug_type($stream) . "\n";
fclose($stream);
echo "closed:" . get_debug_type($stream) . "\n";
echo "exists:" . function_exists("get_debug_type") . "\n";
$removed = unlink($path);
