<?php
// crc32() returns a 32-bit checksum as an integer on 64-bit PHP.
// Source: https://www.php.net/manual/en/function.crc32.php
// md5() and sha1() return hex digests or raw binary digests.
// Sources: https://www.php.net/manual/en/function.md5.php and https://www.php.net/manual/en/function.sha1.php
$payload = "Echo\nPHP";

echo "crc32-dec:[" . crc32($payload) . "]\n";
echo "crc32-hex:[" . dechex(crc32($payload)) . "]\n";
echo "md5-hex:[" . md5($payload) . "]\n";
echo "md5-raw-hex:[" . bin2hex(md5($payload, true)) . "]\n";
echo "md5-raw-len:[" . strlen(md5($payload, true)) . "]\n";
echo "sha1-hex:[" . sha1($payload) . "]\n";
echo "sha1-raw-hex:[" . bin2hex(sha1($payload, true)) . "]\n";
echo "sha1-raw-len:[" . strlen(sha1($payload, true)) . "]\n";
echo "empty-md5:[" . md5("") . "]\n";
echo "empty-sha1:[" . sha1("") . "]\n";
echo "exists:[" . function_exists("crc32") . function_exists("md5") . function_exists("sha1") . "]\n";
