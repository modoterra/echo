<?php
$queue = ["draft", "review", "ship", "archive"];

$removed = array_splice($queue, 1, 2);
echo "removed:[" . implode(",", $removed) . "]\n";
echo "remaining:[" . implode(",", $queue) . "]\n";
echo "count:[" . count($queue) . "]\n";
echo "exists:[" . function_exists("array_splice") . "]\n";
