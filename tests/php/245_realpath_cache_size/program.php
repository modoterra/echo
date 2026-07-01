<?php
$size = realpath_cache_size();

echo "type:[" . gettype($size) . "]\n";
echo "exists:[" . function_exists("realpath_cache_size") . "]\n";
