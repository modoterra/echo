<?php
// dirname() returns a parent directory path from a path string.
// Source: https://www.php.net/manual/en/function.dirname.php
echo "file:[" . dirname("/etc/passwd") . "]\n";
echo "trailing:[" . dirname("/etc/") . "]\n";
echo "root:[" . dirname("/") . "]\n";
echo "dot:[" . dirname(".") . "]\n";
echo "relative:[" . dirname("foo/bar/baz") . "]\n";
echo "repeated:[" . dirname("foo//bar") . "]\n";
echo "levels:[" . dirname("/usr/local/lib", 2) . "]\n";
echo "exists:[" . function_exists("dirname") . "]\n";
