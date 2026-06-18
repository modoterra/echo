<?php
// explode() splits a string into an array of string chunks.
// Source: https://www.php.net/manual/en/function.explode.php
echo "default-count:[" . count(explode(",", "a,b,c")) . "]\n";
echo "positive-limit-count:[" . count(explode(",", "a,b,c", 2)) . "]\n";
echo "zero-limit-count:[" . count(explode(",", "a,b,c", 0)) . "]\n";
echo "negative-limit-count:[" . count(explode(",", "a,b,c", -1)) . "]\n";
echo "missing-negative-count:[" . count(explode(",", "abc", -1)) . "]\n";
echo "edge-empty-count:[" . count(explode(",", ",a,")) . "]\n";
echo "is-list:[" . array_is_list(explode(",", "a,b")) . "]\n";
echo "string-value:[" . explode(",", "a,b") . "]\n";
echo "exists:[" . function_exists("explode") . "]\n";
