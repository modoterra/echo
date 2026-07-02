<?php
$before = error_get_last();
$cleared = error_clear_last();
$after = error_get_last();

echo "before-null:" . is_null($before) . "\n";
echo "clear-null:" . is_null($cleared) . "\n";
echo "after-null:" . is_null($after) . "\n";
echo "exists:" . function_exists("error_get_last") . function_exists("error_clear_last") . "\n";
