<?php
$previous = set_error_handler("strlen");
echo "first-null:[" . is_null($previous) . "]\n";
echo "current:[" . get_error_handler() . "]\n";

$previous = set_error_handler("trim");
echo "second-prev:[" . $previous . "]\n";
echo "current2:[" . get_error_handler() . "]\n";

echo "restore1:[" . restore_error_handler() . "]\n";
echo "after-restore1:[" . get_error_handler() . "]\n";
echo "restore2:[" . restore_error_handler() . "]\n";
echo "after-restore2-null:[" . is_null(get_error_handler()) . "]\n";
