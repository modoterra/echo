<?php
$loopback = gethostbyaddr("127.0.0.1");
$malformed = gethostbyaddr("not-an-ip");

echo "loopback:" . $loopback . "\n";
echo "malformed:" . ($malformed === false) . "\n";
echo "exists:" . function_exists("gethostbyaddr") . "\n";
