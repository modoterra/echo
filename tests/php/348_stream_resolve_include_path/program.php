<?php
$resolved = stream_resolve_include_path(__DIR__ . "/target.php");
echo "resolved:" . basename($resolved) . "\n";
$missing = stream_resolve_include_path(__DIR__ . "/missing.php");
echo "missing:" . var_export($missing, true) . "\n";
echo "exists:" . function_exists("stream_resolve_include_path") . "\n";
