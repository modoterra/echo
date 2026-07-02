<?php
echo "used:[" . count_chars("banana", 3) . "]\n";
echo "array_type:[" . gettype(count_chars("banana", 1)) . "]\n";
echo "exists:[" . function_exists("count_chars") . "]\n";
