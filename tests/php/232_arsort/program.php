<?php
$statuses = ["first" => "review", "second" => "draft", "third" => "ship"];

echo "arsort:[" . arsort($statuses) . "]\n";
foreach ($statuses as $key => $value) {
    echo $key . "=" . $value . ";";
}
echo "\n";
echo "third:[" . $statuses["third"] . "]\n";
echo "exists:[" . function_exists("arsort") . "]\n";
