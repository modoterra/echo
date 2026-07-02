<?php
$added = output_add_rewrite_var("token", "abc");

echo "added:" . $added . "\n";
echo "exists:" . function_exists("output_add_rewrite_var") . "\n";
