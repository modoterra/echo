<?php
// Store the original value in $a.
$a = "A\n";
// Normal assignment copies $a into $b without aliasing.
$b = $a;
// Reassigning the destination must not alter the source.
$b = "B\n";
// $a should still contain the original value.
echo $a;
