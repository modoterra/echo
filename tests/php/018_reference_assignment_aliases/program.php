<?php
$a = "A\n";
$b =& $a;
$a = "B\n";
echo $b;
