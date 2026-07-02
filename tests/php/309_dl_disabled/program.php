<?php
$loaded = dl("missing_echo_extension.so");

echo "loaded:" . $loaded . "\n";
echo "exists:" . function_exists("dl") . "\n";
