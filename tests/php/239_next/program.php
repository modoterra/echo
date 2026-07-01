<?php
$queue = ["first" => "draft", "second" => "review"];
$single = ["only"];
$single_next = next($single);

echo "next:[" . next($queue) . "]\n";
if ($single_next === false) {
    echo "single:[false]\n";
}
echo "exists:[" . function_exists("next") . "]\n";
