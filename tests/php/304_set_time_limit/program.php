<?php
$unlimited = set_time_limit(0);
$limited = set_time_limit(1);

echo "unlimited:" . $unlimited . "\n";
echo "limited:" . $limited . "\n";
echo "exists:" . function_exists("set_time_limit") . "\n";
