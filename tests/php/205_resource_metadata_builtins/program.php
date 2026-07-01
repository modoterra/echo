<?php
$stream = tmpfile();

echo "stream:[" . is_resource($stream) . "]\n";
echo "type:[" . get_resource_type($stream) . "]\n";
echo "id-int:[" . is_int(get_resource_id($stream)) . "]\n";
echo "id-positive:[" . (get_resource_id($stream) > 0) . "]\n";
echo "exists:[" . function_exists("get_resource_type") . function_exists("get_resource_id") . "]\n";

fclose($stream);
