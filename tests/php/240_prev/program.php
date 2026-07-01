<?php
$queue = ["first" => "draft", "second" => "review"];
$empty = [];

$queue_prev = prev($queue);
$empty_prev = prev($empty);

if ($queue_prev === false) {
    echo "prev:[false]\n";
}
if ($empty_prev === false) {
    echo "empty:[false]\n";
}
echo "exists:[" . function_exists("prev") . "]\n";
