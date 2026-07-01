<?php
$info = pathinfo("/srv/app/releases/current/index.php");

echo "dirname:[" . $info["dirname"] . "]\n";
echo "basename:[" . $info["basename"] . "]\n";
echo "extension:[" . $info["extension"] . "]\n";
echo "filename:[" . $info["filename"] . "]\n";
echo "exists:[" . function_exists("pathinfo") . "]\n";
