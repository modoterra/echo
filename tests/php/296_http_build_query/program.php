<?php
$data = [
    "foo" => "bar baz",
    "null" => null,
    "items" => ["red apple", "blue"],
    0 => "lead",
];

echo http_build_query($data, "n_", "&", 1) . "\n";
echo http_build_query($data, "n_", "&amp;", 1) . "\n";
echo http_build_query(["space" => "a b"], "", "&", 2) . "\n";
echo "exists:" . function_exists("http_build_query") . "\n";
