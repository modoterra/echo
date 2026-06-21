<?php
// URL encoding helpers split path-component encoding from form/query encoding.
// Sources:
// - https://www.php.net/manual/en/function.rawurlencode.php
// - https://www.php.net/manual/en/function.rawurldecode.php
// - https://www.php.net/manual/en/function.urlencode.php
// - https://www.php.net/manual/en/function.urldecode.php
$path = "sales and marketing/Miami~";
$query = "Data123!@-_ +~";

echo "raw-path:[" . rawurlencode($path) . "]\n";
echo "form-query:[" . urlencode($query) . "]\n";
echo "raw-decode:[" . rawurldecode("foo%20bar%40baz+plus%ZZ") . "]\n";
echo "form-decode:[" . urldecode("green+and+red%2Bblue%ZZ") . "]\n";
echo "roundtrip:[" . rawurldecode(rawurlencode("a/b c+~")) . "]\n";
echo "exists:[" . function_exists("rawurlencode") . function_exists("urldecode") . "]\n";
