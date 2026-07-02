<?php
$encoded = utf8_encode("Zo" . chr(235));

echo "hex:" . bin2hex($encoded) . "\n";
echo "text:" . $encoded . "\n";
echo "exists:" . function_exists("utf8_encode") . "\n";
