<?php
$queue = ["first" => "draft", "second" => "review"];
$empty = [];
$empty_key = key($empty);

echo "key:[" . key($queue) . "]\n";
if ($empty_key === null) {
    echo "empty:[null]\n";
}
echo "exists:[" . function_exists("key") . "]\n";
