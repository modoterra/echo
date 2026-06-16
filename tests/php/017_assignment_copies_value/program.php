<?php
// Store the original value in $a.
$a = "A\n";
// Normal assignment copies the current value into $b.
$b = $a;
// Reassigning $a must not alter $b.
$a = "B\n";
// $b should still contain the original value.
echo $b;
