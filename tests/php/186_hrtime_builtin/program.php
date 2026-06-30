<?php
$parts = hrtime();

if (is_array($parts)) {
    echo "array\n";
}

echo "parts: " . count($parts) . "\n";

$number = hrtime(true);

if (is_int($number) || is_float($number)) {
    echo "number\n";
}
