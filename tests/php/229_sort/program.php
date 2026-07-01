<?php
$statuses = ["review", "draft", "ship"];

echo "sort:[" . sort($statuses) . "]\n";
echo "values:[" . implode(",", $statuses) . "]\n";
echo "zero:[" . $statuses[0] . "]\n";
echo "exists:[" . function_exists("sort") . "]\n";
