<?php
$usage = memory_get_usage();

echo "type:[" . gettype($usage) . "]\n";
echo "exists:[" . function_exists("memory_get_usage") . "]\n";
