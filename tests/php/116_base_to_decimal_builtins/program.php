<?php
// Base-to-decimal helpers parse external text formats into decimal values.
// Sources:
// - https://www.php.net/manual/en/function.bindec.php
// - https://www.php.net/manual/en/function.hexdec.php
// - https://www.php.net/manual/en/function.octdec.php
$flags = "0b10xx11";
$color = "ffzz10";
$mode = "0789";

echo "flags:[" . bindec($flags) . "]\n";
echo "color:[" . hexdec($color) . "]\n";
echo "mode:[" . octdec($mode) . "]\n";
echo "int-coerce:[" . bindec(10) . ":" . hexdec(10) . ":" . octdec(10) . "]\n";
echo "float-coerce:[" . hexdec(10.7) . "]\n";
echo "empty:[" . bindec("") . ":" . hexdec("") . ":" . octdec("") . "]\n";
echo "large-type:[" . gettype(hexdec("FFFFFFFFFFFFFFFF")) . "]\n";
echo "exists:[" . function_exists("bindec") . function_exists("hexdec") . function_exists("octdec") . "]\n";
