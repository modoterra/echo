<?php
echo "before\n";
debug_print_backtrace();
echo "after\n";
echo "exists:[" . function_exists("debug_print_backtrace") . "]\n";
