<?php
echo "first:" . strtok(";aaa;;bbb;", ";") . "\n";
echo "space:" . strtok("  first\tsecond", " \t") . "\n";
echo "missing:" . var_export(strtok(";;;", ";"), true) . "\n";
echo "exists:" . function_exists("strtok") . "\n";
