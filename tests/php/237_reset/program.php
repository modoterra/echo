<?php
$queue = ["first" => "draft", "second" => "review"];
$empty = [];
$empty_reset = reset($empty);

echo "reset:[" . reset($queue) . "]\n";
if ($empty_reset === false) {
    echo "empty:[false]\n";
}
echo "exists:[" . function_exists("reset") . "]\n";
