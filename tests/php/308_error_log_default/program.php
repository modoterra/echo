<?php
$logged = error_log("echo compatibility error log");

echo "logged:" . $logged . "\n";
echo "exists:" . function_exists("error_log") . "\n";
