<?php
// escapeshellarg() wraps an argument for safe shell argument passing.
// Source: https://www.php.net/manual/en/function.escapeshellarg.php
echo "plain:[" . escapeshellarg("hello") . "]\n";
echo "space:[" . escapeshellarg("hello world") . "]\n";
echo "quote:[" . escapeshellarg("can't") . "]\n";
echo "empty:[" . escapeshellarg("") . "]\n";
echo "meta:[" . escapeshellarg("path; rm -rf /") . "]\n";
echo "numeric:[" . escapeshellarg(42) . "]\n";
echo "exists:[" . function_exists("escapeshellarg") . "]\n";
