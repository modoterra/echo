<?php
$a = "A\n";
$b =& $a;
$b = "B\n";
echo $a;
