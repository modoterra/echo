<?php
// array_is_list() returns true when array keys are 0..count($array)-1.
// Source: https://www.php.net/manual/en/function.array-is-list.php
echo "empty:[" . array_is_list([]) . "]\n";
echo "list:[" . array_is_list([1, 2, 3]) . "]\n";
