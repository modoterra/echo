<?php
$a = ob_start();
echo "hidden";
$b = ob_clean();
echo $a;
echo $b;

ob_start();
echo "flush";
$c = ob_flush();
echo $c;

ob_start();
echo "end-clean";
$d = ob_end_clean();
echo $d;

ob_start();
echo "end-flush";
$e = ob_end_flush();
echo $e;
