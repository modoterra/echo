<?php
$classes = get_declared_classes();
$interfaces = get_declared_interfaces();
$traits = get_declared_traits();

echo "arrays:[" . is_array($classes) . is_array($interfaces) . is_array($traits) . "]\n";
echo "counts-int:[" . is_int(count($classes)) . is_int(count($interfaces)) . is_int(count($traits)) . "]\n";
echo "exists:[" . function_exists("get_declared_classes") . function_exists("get_declared_interfaces") . function_exists("get_declared_traits") . "]\n";
