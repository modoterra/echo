<?php
$queue = ["first" => "draft", "second" => "review"];
$empty = [];
$empty_end = end($empty);

echo "end:[" . end($queue) . "]\n";
if ($empty_end === false) {
    echo "empty:[false]\n";
}
echo "exists:[" . function_exists("end") . "]\n";
