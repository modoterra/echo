<?php
// Filesystem metadata helpers report local file properties.
// Sources:
// - https://www.php.net/manual/en/function.is-readable.php
// - https://www.php.net/manual/en/function.is-writable.php
// - https://www.php.net/manual/en/function.is-executable.php
// - https://www.php.net/manual/en/function.filesize.php
// - https://www.php.net/manual/en/function.realpath.php
$base = __DIR__ . "/data";
$file = $base . "/sample.txt";
$script = $base . "/run.sh";
$missing = $base . "/missing.txt";

echo "readable-file:[" . is_readable($file) . "]\n";
echo "readable-missing:[" . is_readable($missing) . "]\n";
echo "writable-file:[" . is_writable($file) . "]\n";
echo "executable-script:[" . is_executable($script) . "]\n";
echo "executable-file:[" . is_executable($file) . "]\n";
echo "size-file:[" . filesize($file) . "]\n";
echo "realpath-file:[" . basename(realpath($base . "/../data/sample.txt")) . "]\n";
echo "realpath-missing:[" . realpath($missing) . "]\n";
echo "exists:[" . function_exists("is_readable") . function_exists("realpath") . "]\n";
