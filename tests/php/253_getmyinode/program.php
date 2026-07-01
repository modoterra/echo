<?php
$inode = getmyinode();

echo "type:[" . gettype($inode) . "]\n";
echo "exists:[" . function_exists("getmyinode") . "]\n";
