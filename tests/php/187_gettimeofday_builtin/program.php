<?php
$time = gettimeofday();

if (is_array($time)) {
    echo "array\n";
}

echo "parts: " . count($time) . "\n";

$float = gettimeofday(true);

if (is_float($float)) {
    echo "float\n";
}
