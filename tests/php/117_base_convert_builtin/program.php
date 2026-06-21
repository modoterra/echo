<?php
// base_convert() rewrites external text identifiers between explicit bases.
// Source: https://www.php.net/manual/en/function.base-convert.php
$hex_id = "a37334";
$dirty_hex = "ffzz10";

echo "hex-to-bin:[" . base_convert($hex_id, 16, 2) . "]\n";
echo "dirty-hex-to-dec:[" . base_convert($dirty_hex, 16, 10) . "]\n";
echo "negative-ignored:[" . base_convert("-12", 10, 10) . "]\n";
echo "float-coerce:[" . base_convert(3.14, 10, 10) . "]\n";
echo "base36:[" . base_convert("zz", 36, 10) . "]\n";
echo "empty:[" . base_convert("", 10, 16) . "]\n";
echo "exists:[" . function_exists("base_convert") . "]\n";
