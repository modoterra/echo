<?php
$result = usleep(0);

if ($result === null) {
    echo "usleep:[null]\n";
}
echo "exists:[" . function_exists("usleep") . "]\n";
