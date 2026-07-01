<?php
$previous = set_exception_handler("strlen");
echo "first-null:[" . is_null($previous) . "]\n";
echo "current:[" . get_exception_handler() . "]\n";

$previous = set_exception_handler("trim");
echo "second-prev:[" . $previous . "]\n";
echo "current2:[" . get_exception_handler() . "]\n";

echo "restore1:[" . restore_exception_handler() . "]\n";
echo "after-restore1:[" . get_exception_handler() . "]\n";
echo "restore2:[" . restore_exception_handler() . "]\n";
echo "after-restore2-null:[" . is_null(get_exception_handler()) . "]\n";
