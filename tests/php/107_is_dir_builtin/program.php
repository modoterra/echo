<?php
// is_dir() checks whether a local path exists and is a directory.
// Source: https://www.php.net/manual/en/function.is-dir.php
echo "directory:[" . is_dir(".") . "]\n";
echo "file:[" . is_dir("Cargo.toml") . "]\n";
echo "missing:[" . is_dir("definitely_missing_echo_directory") . "]\n";
echo "empty:[" . is_dir("") . "]\n";
echo "exists:[" . function_exists("is_dir") . "]\n";
