<?php
$free = disk_free_space(__DIR__);
$total = disk_total_space(__DIR__);
$alias = diskfreespace(__DIR__);
$missing = disk_free_space(__DIR__ . "/missing-directory");

echo "free-float:[" . is_float($free) . "]\n";
echo "total-float:[" . is_float($total) . "]\n";
echo "alias-float:[" . is_float($alias) . "]\n";
echo "ordered:[" . ($free <= $total) . "]\n";
echo "missing:[" . ($missing === false) . "]\n";
