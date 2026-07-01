<?php
echo "tcp:[" . getprotobyname("tcp") . "]\n";
echo "udp:[" . getprotobyname("udp") . "]\n";
echo "missing_type:[" . gettype(getprotobyname("definitely-not-a-protocol")) . "]\n";
echo "exists:[" . function_exists("getprotobyname") . "]\n";
