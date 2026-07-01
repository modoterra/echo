<?php
$queue = ["first" => "draft", "second" => "review"];
$empty = [];

$queue_pos = pos($queue);
$empty_pos = pos($empty);

echo "pos:[" . $queue_pos . "]\n";
if ($empty_pos === false) {
    echo "empty:[false]\n";
}
echo "exists:[" . function_exists("pos") . "]\n";
