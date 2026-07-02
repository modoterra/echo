<?php
$reset = output_reset_rewrite_vars();

echo "reset:" . $reset . "\n";
echo "exists:" . function_exists("output_reset_rewrite_vars") . "\n";
