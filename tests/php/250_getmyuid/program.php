<?php
$uid = getmyuid();

echo "type:[" . gettype($uid) . "]\n";
echo "exists:[" . function_exists("getmyuid") . "]\n";
