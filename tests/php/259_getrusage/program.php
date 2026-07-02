<?php
$usage = getrusage();

echo "type:[" . gettype($usage) . "]\n";
echo "utime_sec_type:[" . gettype($usage["ru_utime.tv_sec"]) . "]\n";
echo "stime_usec_type:[" . gettype($usage["ru_stime.tv_usec"]) . "]\n";
echo "exists:[" . function_exists("getrusage") . "]\n";
