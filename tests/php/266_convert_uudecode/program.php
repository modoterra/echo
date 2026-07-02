<?php
$encoded = convert_uuencode("Hi");

echo "decoded:[" . convert_uudecode($encoded) . "]\n";
echo "bad_type:[" . gettype(convert_uudecode("not valid")) . "]\n";
echo "exists:[" . function_exists("convert_uudecode") . "]\n";
