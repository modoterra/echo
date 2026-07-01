<?php
$all = get_resources();
$streams = get_resources("stream");
$stream = tmpfile();
$streams_after_open = get_resources("stream");

echo "arrays:[" . is_array($all) . is_array($streams) . is_array($streams_after_open) . "]\n";
echo "counts-int:[" . is_int(count($all)) . is_int(count($streams)) . is_int(count($streams_after_open)) . "]\n";
echo "exists:[" . function_exists("get_resources") . "]\n";

fclose($stream);
