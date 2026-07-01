<?php
$cache = realpath_cache_get();

echo "type:[" . gettype($cache) . "]\n";
echo "exists:[" . function_exists("realpath_cache_get") . "]\n";
