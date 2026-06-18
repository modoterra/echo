<?php
// function_exists() checks defined functions and returns false for constructs.
// Echo currently reports supported internal PHP builtins.
// Source: https://www.php.net/manual/en/function.function-exists.php
echo "strlen:[" . function_exists("strlen") . "]\n";
echo "STRLEN:[" . function_exists("STRLEN") . "]\n";
echo "sizeof:[" . function_exists("sizeof") . "]\n";
echo "function_exists:[" . function_exists("function_exists") . "]\n";
echo "echo:[" . function_exists("echo") . "]\n";
echo "missing:[" . function_exists("definitely_missing_echo_builtin") . "]\n";
echo "empty-string:[" . function_exists("") . "]\n";
