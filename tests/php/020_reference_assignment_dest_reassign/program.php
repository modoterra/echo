<?php
// Store the original value in $a.
$a = "A\n";
// Reference assignment makes $b an alias of $a.
$b =& $a;
// Reassigning through $b changes the shared referenced value.
$b = "B\n";
// Reading through $a observes the new shared value.
echo $a;
