<?php
// base64_encode() encodes bytes with padded base64 and no line breaks.
// Source: https://www.php.net/manual/en/function.base64-encode.php
echo "[" . base64_encode("") . "]\n";
echo base64_encode("f") . "\n";
echo base64_encode("fo") . "\n";
echo base64_encode("foo") . "\n";
echo base64_encode("hello world") . "\n";
echo "nonascii:" . base64_encode("Ächo") . "\n";
echo "number:" . base64_encode(123) . "\n";
