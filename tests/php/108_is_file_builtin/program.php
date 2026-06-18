<?php
// is_file() checks whether a local path exists and is a regular file.
// Source: https://www.php.net/manual/en/function.is-file.php
echo "file:[" . is_file("Cargo.toml") . "]\n";
echo "directory:[" . is_file(".") . "]\n";
echo "missing:[" . is_file("definitely_missing_echo_file") . "]\n";
echo "empty:[" . is_file("") . "]\n";
echo "exists:[" . function_exists("is_file") . "]\n";
