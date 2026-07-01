<?php
$gid = getmygid();

echo "type:[" . gettype($gid) . "]\n";
echo "exists:[" . function_exists("getmygid") . "]\n";
