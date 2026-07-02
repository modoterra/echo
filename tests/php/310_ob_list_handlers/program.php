<?php
ob_start();
echo "buffered";
$handlers = ob_list_handlers();
ob_end_clean();

echo "handlers:" . implode(",", $handlers) . "\n";
echo "count:" . count($handlers) . "\n";
echo "exists:" . function_exists("ob_list_handlers") . "\n";
