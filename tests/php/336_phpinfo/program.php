<?php
ob_start();
$result = phpinfo(0);
$info = ob_get_clean();

echo "result:" . $result . "\n";
echo "non-empty:" . ($info !== "") . "\n";
echo "mentions-php:" . (strpos(strtolower($info), "php") !== false) . "\n";
echo "exists:" . function_exists("phpinfo") . "\n";
