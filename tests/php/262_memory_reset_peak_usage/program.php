<?php
$result = memory_reset_peak_usage();

echo "type:[" . gettype($result) . "]\n";
echo "exists:[" . function_exists("memory_reset_peak_usage") . "]\n";
