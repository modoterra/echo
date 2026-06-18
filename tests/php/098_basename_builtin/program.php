<?php
// basename() returns the trailing name component of a path string.
// Source: https://www.php.net/manual/en/function.basename.php
echo "file:[" . basename("/etc/passwd") . "]\n";
echo "suffix:[" . basename("/etc/sudoers.d", ".d") . "]\n";
echo "trailing:[" . basename("/etc/") . "]\n";
echo "root:[" . basename("/") . "]\n";
echo "dot:[" . basename(".") . "]\n";
echo "exists:[" . function_exists("basename") . "]\n";
