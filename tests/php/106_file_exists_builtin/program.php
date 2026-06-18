<?php
// file_exists() checks whether a local file or directory exists.
// Source: https://www.php.net/manual/en/function.file-exists.php
echo "file:[" . file_exists("Cargo.toml") . "]\n";
echo "directory:[" . file_exists(".") . "]\n";
echo "missing:[" . file_exists("definitely_missing_echo_file") . "]\n";
echo "empty:[" . file_exists("") . "]\n";
echo "exists:[" . function_exists("file_exists") . "]\n";
