<?php
// Store the original value in $a.
$a = "A\n";
// Reference assignment makes $b an alias of $a.
$b =& $a;
// Reassigning $a changes the shared referenced value.
$a = "B\n";
// Reading through $b observes the new shared value.
echo $b;
