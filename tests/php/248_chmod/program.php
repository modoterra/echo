<?php
$path = sys_get_temp_dir() . "/echo_chmod_" . getmypid();
$written = file_put_contents($path, "x");

$ok = chmod($path, 0600);

if ($ok) {
    echo "chmod:[true]\n";
}
echo "exists:[" . function_exists("chmod") . "]\n";

$removed = unlink($path);
