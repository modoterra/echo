<?php
// Start one buffer so ob_get_level() returns 1.
ob_start();
// Store the integer return value in a variable.
$level = ob_get_level();
// Remove the buffer before printing so the result goes straight to stdout.
ob_end_clean();
// Echo reads the previously assigned value.
echo $level, "\n";
