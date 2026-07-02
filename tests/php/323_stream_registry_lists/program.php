<?php
$wrappers = stream_get_wrappers();
$transports = stream_get_transports();
$filters = stream_get_filters();

echo "wrappers-array:[" . is_array($wrappers) . "]\n";
echo "wrappers-file:[" . in_array("file", $wrappers, true) . "]\n";
echo "transports-array:[" . is_array($transports) . "]\n";
echo "transports-tcp:[" . in_array("tcp", $transports, true) . "]\n";
echo "filters-array:[" . is_array($filters) . "]\n";
echo "filters-count-int:[" . is_int(count($filters)) . "]\n";
echo "exists:" . function_exists("stream_get_wrappers") . function_exists("stream_get_transports") . function_exists("stream_get_filters") . "\n";
