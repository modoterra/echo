<?php
$user = get_current_user();

echo "type:[" . gettype($user) . "]\n";
echo "exists:[" . function_exists("get_current_user") . "]\n";
