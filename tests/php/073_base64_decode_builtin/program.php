<?php
// base64_decode() decodes bytes in non-strict mode by default.
// Source: https://www.php.net/manual/en/function.base64-decode.php
echo "[" . base64_decode("") . "]\n";
echo base64_decode("Zg==") . "\n";
echo base64_decode("Zm8=") . "\n";
echo base64_decode("Zm9v") . "\n";
echo base64_decode("aGVsbG8gd29ybGQ=") . "\n";
echo "nonascii:" . base64_decode("w4RjaG8=") . "\n";
echo "number:" . base64_decode("MTIz") . "\n";
echo "ignored:" . base64_decode("Zm 9v") . "\n";
echo "invalid:[" . base64_decode("!!!!") . "]\n";
