<?php
ob_start();
$result = phpcredits(0);
$credits = ob_get_clean();

echo "result:" . $result . "\n";
echo "non-empty:" . ($credits !== "") . "\n";
echo "mentions-php:" . (strpos(strtolower($credits), "php") !== false) . "\n";
echo "exists:" . function_exists("phpcredits") . "\n";
