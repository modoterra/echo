<?php
$statuses = ["review", "draft", "ship"];

echo "rsort:[" . rsort($statuses) . "]\n";
echo "values:[" . implode(",", $statuses) . "]\n";
echo "zero:[" . $statuses[0] . "]\n";
echo "exists:[" . function_exists("rsort") . "]\n";
