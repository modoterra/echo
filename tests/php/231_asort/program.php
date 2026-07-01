<?php
$statuses = ["first" => "review", "second" => "draft", "third" => "ship"];

echo "asort:[" . asort($statuses) . "]\n";
foreach ($statuses as $key => $value) {
    echo $key . "=" . $value . ";";
}
echo "\n";
echo "second:[" . $statuses["second"] . "]\n";
echo "exists:[" . function_exists("asort") . "]\n";
