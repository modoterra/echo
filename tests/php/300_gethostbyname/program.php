<?php
$local = gethostbyname("localhost");
$missing = gethostbyname("echo.invalid");

echo "local:" . $local . "\n";
echo "missing:" . $missing . "\n";
echo "exists:" . function_exists("gethostbyname") . "\n";
