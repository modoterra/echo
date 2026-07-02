<?php
echo "set:" . setlocale(0, "C") . "\n";
echo "posix:" . setlocale(0, "POSIX") . "\n";
echo "bad:" . var_export(setlocale(0, "missing_LOCALE"), true) . "\n";
echo "exists:" . function_exists("setlocale") . "\n";
