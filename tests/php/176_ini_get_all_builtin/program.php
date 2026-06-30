<?php
$all = ini_get_all();
echo "all:" . count($all) . "\n";

$core = ini_get_all(null, false);
echo "core:" . count($core) . "\n";

if (ini_get_all("json") === false) {
    echo "json false\n";
}
