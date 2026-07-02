<?php
$old = umask(18);
echo "set:" . umask(null) . "\n";
$previous = umask(7);
echo "previous:" . $previous . "\n";
echo "current:" . umask(null) . "\n";
$ignored = umask($old);
echo "exists:" . function_exists("umask") . "\n";
