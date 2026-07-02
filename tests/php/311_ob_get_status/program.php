<?php
ob_start();
echo "abc";
$status = ob_get_status(false);
ob_end_clean();

echo "name:" . $status["name"] . "\n";
echo "type:" . $status["type"] . "\n";
echo "level:" . $status["level"] . "\n";
echo "used:" . $status["buffer_used"] . "\n";
echo "count:" . count($status) . "\n";
echo "exists:" . function_exists("ob_get_status") . "\n";
