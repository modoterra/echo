<?php
$load = sys_getloadavg();

echo "type:[" . gettype($load) . "]\n";
echo "count:[" . count($load) . "]\n";
echo "first_type:[" . gettype($load[0]) . "]\n";
echo "exists:[" . function_exists("sys_getloadavg") . "]\n";
