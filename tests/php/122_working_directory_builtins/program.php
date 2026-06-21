<?php
// chdir() changes the current directory; getcwd() reads it back.
// Source: https://www.php.net/manual/en/function.chdir.php
// Source: https://www.php.net/manual/en/function.getcwd.php
$start = getcwd();

echo "start-ok:[" . is_string($start) . "]\n";
echo "enter:[" . chdir(__DIR__) . "]\n";
echo "after:[" . basename(getcwd()) . "]\n";
echo "local-file:[" . file_exists("program.php") . "]\n";
echo "restore:[" . chdir($start) . "]\n";
echo "restored-ok:[" . is_string(getcwd()) . "]\n";
echo "exists:[" . function_exists("chdir") . function_exists("getcwd") . "]\n";
