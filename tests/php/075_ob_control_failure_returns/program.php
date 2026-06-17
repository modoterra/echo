<?php
$a = ob_flush();
$b = ob_clean();
$c = ob_end_flush();
$d = ob_end_clean();

echo $a;
echo $b;
echo $c;
echo $d;
echo ob_get_level();
