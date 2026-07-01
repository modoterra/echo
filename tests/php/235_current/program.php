<?php
$queue = ["first" => "draft", "second" => "review"];
$empty = [];
$empty_current = current($empty);

echo "current:[" . current($queue) . "]\n";
if ($empty_current === false) {
    echo "empty:[false]\n";
}
echo "exists:[" . function_exists("current") . "]\n";
