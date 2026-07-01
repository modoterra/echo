<?php
$statuses = ["b" => "review", "a" => "draft", "c" => "ship"];

echo "krsort:[" . krsort($statuses) . "]\n";
foreach ($statuses as $key => $value) {
    echo $key . "=" . $value . ";";
}
echo "\n";
echo "c:[" . $statuses["c"] . "]\n";
echo "exists:[" . function_exists("krsort") . "]\n";
