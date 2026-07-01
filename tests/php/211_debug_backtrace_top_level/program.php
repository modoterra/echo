<?php
$trace = debug_backtrace();
echo "is-array:[" . is_array($trace) . "]\n";
echo "count:[" . count($trace) . "]\n";
echo "exists:[" . function_exists("debug_backtrace") . "]\n";
