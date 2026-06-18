<?php
// is_link() checks whether a local path exists and is a symbolic link.
// Source: https://www.php.net/manual/en/function.is-link.php
echo "link:[" . is_link("/proc/self/exe") . "]\n";
echo "file:[" . is_link("Cargo.toml") . "]\n";
echo "directory:[" . is_link(".") . "]\n";
echo "missing:[" . is_link("definitely_missing_echo_link") . "]\n";
echo "empty:[" . is_link("") . "]\n";
echo "exists:[" . function_exists("is_link") . "]\n";
