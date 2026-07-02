<?php
$decoded = utf8_decode("Zoë€");

echo "hex:" . bin2hex($decoded) . "\n";
echo "exists:" . function_exists("utf8_decode") . "\n";
