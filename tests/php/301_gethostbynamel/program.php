<?php
$local = gethostbynamel("localhost");
$missing = gethostbynamel("echo.invalid");

echo "local-array:" . is_array($local) . "\n";
echo "local-has-loopback:" . in_array("127.0.0.1", $local, true) . "\n";
echo "missing:" . ($missing === false) . "\n";
echo "exists:" . function_exists("gethostbynamel") . "\n";
