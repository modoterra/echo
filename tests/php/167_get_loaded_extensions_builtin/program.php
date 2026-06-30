<?php
$extensions = get_loaded_extensions();
echo is_array($extensions) . "\n";
echo count($extensions) . "\n";

$zendExtensions = get_loaded_extensions(true);
echo is_array($zendExtensions) . "\n";
echo count($zendExtensions) . "\n";
