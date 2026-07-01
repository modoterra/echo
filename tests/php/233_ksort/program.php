<?php
$statuses = ["b" => "review", "a" => "draft", "c" => "ship"];

echo "ksort:[" . ksort($statuses) . "]\n";
foreach ($statuses as $key => $value) {
    echo $key . "=" . $value . ";";
}
echo "\n";
echo "a:[" . $statuses["a"] . "]\n";
echo "exists:[" . function_exists("ksort") . "]\n";
